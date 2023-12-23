//! https://github.com/esp-rs/esp-hal/blob/main/esp32c3-hal/examples/embassy_serial.rs

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use embedded_hal_async::digital::Wait;
use embedded_io_async::Write;
use esp32c3_hal::{
    clock::ClockControl,
    embassy,
    gpio::{GpioPin, Input, PullUp},
    interrupt,
    peripherals::{Interrupt, Peripherals, UART0, UART1},
    prelude::*,
    rmt::{Channel0, Rmt},
    uart::{
        config::{AtCmdConfig, Config},
        TxRxPins, UartRx, UartTx,
    },
    Uart, IO,
};
use esp_backtrace as _;
use esp_hal_smartled::{smartLedAdapter, SmartLedsAdapter};
use heapless::String;
use robots_lib::{Cmd, Error, Vec, CMD_MAX_SIZE};
use smart_leds::{
    brightness, gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
};
use static_cell::make_static;

type CmdSignal = Signal<NoopRawMutex, Cmd>;
type HueSignal = Signal<NoopRawMutex, u8>;
type MonSignal = Signal<NoopRawMutex, String<100>>;
type TX0 = UartTx<'static, UART0>;
type RX0 = UartRx<'static, UART0>;
type TX1 = UartTx<'static, UART1>;
type Led = SmartLedsAdapter<Channel0<0>, 0, 25>;
type Btn = GpioPin<Input<PullUp>, 9>;

fn monitor_err(mon_sig: &'static MonSignal, e: impl Into<Error>) {
    use core::fmt::Write;
    let mut out = String::new();
    write!(&mut out, "{:?}", e.into()).unwrap();
    mon_sig.signal(out);
}

#[embassy_executor::task]
async fn tx_task(mut tx: TX0, cmd_sig: &'static CmdSignal, mon_sig: &'static MonSignal) {
    loop {
        let cmd = cmd_sig.wait().await;
        tx.write_all(cmd.to_vec().unwrap().as_slice())
            .await
            .unwrap_or_else(|e| monitor_err(mon_sig, e));
        mon_sig.signal(String::from("tx end"));
    }
}

#[embassy_executor::task]
async fn rx_task(
    mut rx: RX0,
    cmd_sig: &'static CmdSignal,
    hue_sig: &'static HueSignal,
    mon_sig: &'static MonSignal,
) {
    let mut rbuf: [u8; CMD_MAX_SIZE] = [0u8; CMD_MAX_SIZE];

    loop {
        match embedded_io_async::Read::read(&mut rx, &mut rbuf).await {
            Err(e) => monitor_err(mon_sig, e),
            Ok(len) => {
                let mut v = Vec::from_slice(&rbuf[..len]).unwrap();
                match Cmd::from_vec(&mut v) {
                    Err(e) => monitor_err(mon_sig, e),
                    Ok(Cmd::Ping) => cmd_sig.signal(Cmd::Pong),
                    Ok(Cmd::Hue(h)) => hue_sig.signal(h),
                    Ok(_) => {}
                };
            }
        }
        mon_sig.signal(String::from("rx end"));
    }
}

#[embassy_executor::task]
async fn led_task(mut led: Led, hue_sig: &'static HueSignal, mon_sig: &'static MonSignal) {
    let mut color = Hsv {
        hue: 128,
        sat: 255,
        val: 255,
    };
    loop {
        color.hue = hue_sig.wait().await;
        let data = [hsv2rgb(color)];
        led.write(brightness(gamma(data.iter().cloned()), 10))
            .unwrap_or_else(|e| monitor_err(mon_sig, e));
        mon_sig.signal(String::from("led end"));
    }
}

#[embassy_executor::task]
async fn btn_task(mut btn: Btn, cmd_sig: &'static CmdSignal, mon_sig: &'static MonSignal) {
    loop {
        btn.wait_for_falling_edge().await.unwrap();
        cmd_sig.signal(Cmd::Button);
        mon_sig.signal(String::from("btn end"));
    }
}

#[embassy_executor::task]
async fn monitor_task(mut tx: TX1, mon_sig: &'static MonSignal) {
    let txt = String::<20>::from("\r\n\nstart monitor\r\n");
    tx.write_all(txt.into_bytes().as_slice()).await.unwrap();
    loop {
        let txt = mon_sig.wait().await;
        tx.write_all(txt.into_bytes().as_slice()).await.unwrap();
        tx.write_all(&[b'\r', b'\n']).await.unwrap();
    }
}

#[embassy_executor::task]
async fn ping_task(cmd_sig: &'static CmdSignal, mon_sig: &'static MonSignal) {
    loop {
        Timer::after(Duration::from_millis(3_000)).await;
        cmd_sig.signal(Cmd::Ping);
        mon_sig.signal(String::from("ping end"));
    }
}

#[main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let rmt = Rmt::new(peripherals.RMT, 80u32.MHz(), &clocks).unwrap();
    let led = <smartLedAdapter!(0, 1)>::new(rmt.channel0, io.pins.gpio8);

    let btn = io.pins.gpio9.into_pull_up_input();

    let timer_group0 = esp32c3_hal::timer::TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0.timer0);

    let mut serial0 = Uart::new(peripherals.UART0, &clocks);
    serial0.set_at_cmd(AtCmdConfig::new(Some(0), Some(0), None, 0, Some(1)));
    serial0
        .set_rx_fifo_full_threshold(CMD_MAX_SIZE as u16)
        .unwrap();
    let (tx0, rx0) = serial0.split();

    let pins1 = TxRxPins::new_tx_rx(
        io.pins.gpio0.into_push_pull_output(),
        io.pins.gpio1.into_floating_input(),
    );
    let serial1 = Uart::new_with_config(peripherals.UART1, Config::default(), Some(pins1), &clocks);
    let (tx1, _) = serial1.split();

    interrupt::enable(Interrupt::UART0, interrupt::Priority::Priority1).unwrap();
    interrupt::enable(Interrupt::GPIO, interrupt::Priority::Priority2).unwrap();

    let cmd_sig = make_static!(Signal::new());
    let hue_sig = make_static!(Signal::new());
    let mon_sig = make_static!(Signal::new());

    spawner.spawn(rx_task(rx0, cmd_sig, hue_sig, mon_sig)).ok();
    spawner.spawn(tx_task(tx0, cmd_sig, mon_sig)).ok();
    spawner.spawn(led_task(led, hue_sig, mon_sig)).ok();
    spawner.spawn(btn_task(btn, cmd_sig, mon_sig)).ok();
    spawner.spawn(ping_task(cmd_sig, mon_sig)).ok();
    spawner.spawn(monitor_task(tx1, mon_sig)).ok();
}
