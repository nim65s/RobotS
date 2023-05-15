//! embassy_hello_world + serial_interrupts

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::{cell::RefCell, fmt::Write};

use critical_section::Mutex;
use embassy_executor::Executor;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp32c3_hal::{
    clock::ClockControl,
    embassy,
    gpio::{self, Event, Gpio9, Input, PullDown},
    interrupt,
    peripherals::{Interrupt, Peripherals, UART0, UART1},
    prelude::*,
    pulse_control::{Channel0, ClockSource},
    riscv,
    timer::TimerGroup,
    uart::{
        config::{AtCmdConfig, Config},
        TxRxPins,
    },
    Cpu, PulseControl, Rtc, Uart, IO,
};
use esp_backtrace as _;
use esp_hal_smartled::{smartLedAdapter, SmartLedsAdapter};
use robots_lib::{Cmd, Vec};
use smart_leds::{
    brightness, gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
};
use static_cell::StaticCell;

type Gpio8 = gpio::GpioPin<
    gpio::Unknown,
    gpio::Bank0GpioRegisterAccess,
    gpio::SingleCoreInteruptStatusRegisterAccessBank0,
    gpio::InputOutputPinType,
    gpio::Gpio8Signals,
    8,
>;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();
static SERIAL0: Mutex<RefCell<Option<Uart<UART0>>>> = Mutex::new(RefCell::new(None));
static SERIAL1: Mutex<RefCell<Option<Uart<UART1>>>> = Mutex::new(RefCell::new(None));
static RECV_CMD: Signal<CriticalSectionRawMutex, Cmd> = Signal::new();
static HUE: Signal<CriticalSectionRawMutex, u8> = Signal::new();
static BUTTON: Mutex<RefCell<Option<Gpio9<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));

macro_rules! monitor {
    ($input:tt) => {
        critical_section::with(|cs| {
            let mut serial = SERIAL1.borrow_ref_mut(cs);
            let serial = serial.as_mut().unwrap();
            write!(serial, $input).ok();
            writeln!(serial, "\r").ok();
        });
    };
}

fn send_cmd(cmd: Cmd) {
    critical_section::with(|cs| {
        let mut serial = SERIAL0.borrow_ref_mut(cs);
        let serial = serial.as_mut().unwrap();

        for c in cmd.to_vec().unwrap().iter() {
            serial.write(*c).unwrap();
        }
    });
}

#[embassy_executor::task]
async fn three_secs() {
    loop {
        monitor!("three_secs");
        send_cmd(Cmd::Ping);
        Timer::after(Duration::from_millis(3_000)).await;
    }
}

#[embassy_executor::task]
async fn recv_cmd() {
    loop {
        let cmd = RECV_CMD.wait().await;
        monitor!("recv_cmd: {cmd:?}");
        match cmd {
            Cmd::Ping => send_cmd(Cmd::Pong),
            Cmd::Hue(hue) => HUE.signal(hue),
            _ => {}
        }
    }
}

#[embassy_executor::task]
async fn led_task(channel0: Channel0, gpio8: Gpio8) {
    let mut led = <smartLedAdapter!(1)>::new(channel0, gpio8);
    let mut data;
    let mut color = Hsv {
        hue: 0,
        sat: 255,
        val: 255,
    };

    loop {
        color.hue = HUE.wait().await;
        data = [hsv2rgb(color)];
        led.write(brightness(gamma(data.iter().cloned()), 10))
            .unwrap();
    }
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;

    // Disable watchdog timers
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let pulse = PulseControl::new(
        peripherals.RMT,
        &mut system.peripheral_clock_control,
        ClockSource::APB,
        0,
        0,
        0,
    )
    .unwrap();

    let mut serial0 = Uart::new(peripherals.UART0, &mut system.peripheral_clock_control);
    serial0.set_at_cmd(AtCmdConfig::new(Some(0), Some(0), None, 0, Some(1)));
    serial0.listen_at_cmd();

    let pins1 = TxRxPins::new_tx_rx(
        io.pins.gpio0.into_push_pull_output(),
        io.pins.gpio1.into_floating_input(),
    );
    let serial1 = Uart::new_with_config(
        peripherals.UART1,
        Some(Config::default()),
        Some(pins1),
        &clocks,
        &mut system.peripheral_clock_control,
    );

    let mut button = io.pins.gpio9.into_pull_down_input();
    button.listen(Event::FallingEdge);

    critical_section::with(|cs| {
        SERIAL0.borrow_ref_mut(cs).replace(serial0);
        SERIAL1.borrow_ref_mut(cs).replace(serial1);
        BUTTON.borrow_ref_mut(cs).replace(button)
    });

    interrupt::enable(Interrupt::UART0, interrupt::Priority::Priority1).unwrap();
    interrupt::enable(Interrupt::GPIO, interrupt::Priority::Priority2).unwrap();

    embassy::init(&clocks, timer_group0.timer0);

    interrupt::set_kind(
        Cpu::ProCpu,
        interrupt::CpuInterrupt::Interrupt1, // Interrupt 1 handles priority one interrupts
        interrupt::InterruptKind::Edge,
    );

    unsafe {
        riscv::interrupt::enable();
    }

    monitor!("Hello");

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(three_secs()).ok();
        spawner.spawn(recv_cmd()).ok();
        spawner.spawn(led_task(pulse.channel0, io.pins.gpio8)).ok();
    });
}

#[interrupt]
fn UART0() {
    let mut vec = Vec::new();
    critical_section::with(|cs| {
        let mut serial = SERIAL0.borrow_ref_mut(cs);
        let serial = serial.as_mut().unwrap();

        while let nb::Result::Ok(c) = serial.read() {
            vec.push(c).unwrap();
        }

        serial.reset_at_cmd_interrupt();
    });
    RECV_CMD.signal(Cmd::from_vec(&mut vec).unwrap());
}

#[interrupt]
fn GPIO() {
    critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
    monitor!("button");
    send_cmd(Cmd::Button);
}
