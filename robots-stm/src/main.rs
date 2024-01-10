//! <https://github.com/embassy-rs/embassy/blob/main/examples/stm32f1/src/bin/usb_serial.rs>

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::used_underscore_binding)]

use defmt::{error, info, panic, Format};
use embassy_executor::Spawner;
use embassy_futures::{
    join::join,
    select::{select, Either},
};
use embassy_stm32::{
    bind_interrupts,
    exti::ExtiInput,
    gpio::{Input, Level, Output, Pull, Speed},
    peripherals::{PA0, PB2, USB},
    time::Hertz,
    usb::{Driver, InterruptHandler},
    Config,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_time::Timer;
use embassy_usb::{
    class::cdc_acm::{CdcAcmClass, State},
    driver::EndpointError,
    Builder,
};
use robots_lib::{Cmd, Vec, CMD_MAX_SIZE};
use static_cell::make_static;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB_LP_CAN1_RX0 => InterruptHandler<USB>;
});

type CmdSignal = Signal<NoopRawMutex, Cmd>;
type LedSignal = Signal<NoopRawMutex, bool>;
type UsbDriver = Driver<'static, USB>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");

    let mut config = Config::default();
    config.rcc.hse = Some(Hertz(8_000_000));
    config.rcc.sys_ck = Some(Hertz(48_000_000));
    config.rcc.pclk1 = Some(Hertz(24_000_000));
    let mut p = embassy_stm32::init(config);

    {
        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        // This forced reset is needed only for development, without it host
        // will not reset your device when you upload new firmware.
        let _dp = Output::new(&mut p.PA12, Level::Low, Speed::Low);
        Timer::after_millis(10).await;
    }

    let led = Output::new(p.PB2, Level::High, Speed::Low);
    let btn = ExtiInput::new(Input::new(p.PA0, Pull::Down), p.EXTI0);
    let usb_driver = Driver::new(p.USB, Irqs, p.PA12, p.PA11);

    let recv_sig = make_static!(Signal::new());
    let send_sig = make_static!(Signal::new());
    let led_sig = make_static!(Signal::new());

    info!("Go !");
    if let Err(e) = spawner.spawn(usb_task(usb_driver, recv_sig, send_sig)) {
        error!("usb_task error: {:?}", e);
    }
    if let Err(e) = spawner.spawn(ping_task(send_sig)) {
        error!("ping_task error: {:?}", e);
    }
    if let Err(e) = spawner.spawn(dispatch_task(recv_sig, send_sig, led_sig)) {
        error!("ping_task error: {:?}", e);
    }
    if let Err(e) = spawner.spawn(led_task(led, led_sig)) {
        error!("led_task error: {:?}", e);
    }
    if let Err(e) = spawner.spawn(btn_task(btn, send_sig)) {
        error!("btn_task error: {:?}", e);
    }
}

#[embassy_executor::task]
async fn ping_task(send_sig: &'static CmdSignal) {
    loop {
        Timer::after_millis(3_000).await;
        send_sig.signal(Cmd::Ping);
    }
}

#[embassy_executor::task]
async fn usb_task(driver: UsbDriver, recv_sig: &'static CmdSignal, send_sig: &'static CmdSignal) {
    let config = embassy_usb::Config::new(0xc0de, 0xcafe);
    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 7];
    let mut state = State::new();
    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);
    let mut usb = builder.build();
    let usb_fut = usb.run();

    // Do stuff with the class!
    let cdc_fut = async {
        let mut buf = [0; CMD_MAX_SIZE + 2];
        loop {
            class.wait_connection().await;
            info!("Connected");
            loop {
                match select(class.read_packet(&mut buf), send_sig.wait()).await {
                    Either::First(Err(e)) => {
                        error!("usb read {:?}", e);
                        break;
                    }
                    Either::First(Ok(len)) => match Vec::from_slice(&buf[..len]) {
                        Err(()) => error!("Vec::from_slice {:?} {}", buf, len),
                        Ok(mut v) => match Cmd::from_vec(&mut v) {
                            Err(e) => error!("Cmd::from_vec {:?}", e),
                            Ok(cmd) => recv_sig.signal(cmd),
                        },
                    },
                    Either::Second(cmd) => match cmd.to_vec() {
                        Err(e) => error!("cmd.to_vec {:?}", e),
                        Ok(data) => {
                            info!("Sending {:?}", cmd);
                            class
                                .write_packet(&data)
                                .await
                                .unwrap_or_else(|e| error!("usb write {:?}", e));
                        }
                    },
                }
            }
            info!("Disconnected");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, cdc_fut).await;
}

#[derive(Format)]
struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Self {},
        }
    }
}

#[embassy_executor::task]
async fn dispatch_task(
    recv_sig: &'static CmdSignal,
    send_sig: &'static CmdSignal,
    led_sig: &'static LedSignal,
) {
    loop {
        match recv_sig.wait().await {
            Cmd::Ping => send_sig.signal(Cmd::Pong),
            Cmd::Led(state) => led_sig.signal(state),
            cmd => info!("Received {:?}", cmd),
        }
    }
}

#[embassy_executor::task]
async fn btn_task(mut btn: ExtiInput<'static, PA0>, send_sig: &'static CmdSignal) {
    loop {
        btn.wait_for_rising_edge().await;
        send_sig.signal(Cmd::Button);
        info!("button");
    }
}

#[embassy_executor::task]
async fn led_task(mut led: Output<'static, PB2>, led_sig: &'static LedSignal) {
    loop {
        let level = led_sig.wait().await;
        led.set_level(level.into());
        info!("set led level {:?}", level);
    }
}
