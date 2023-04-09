//! embassy_hello_world + serial_interrupts

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::{cell::RefCell, fmt::Write};
use critical_section::Mutex;
use embassy_executor::Executor;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use esp32c3_hal::{
    clock::ClockControl,
    embassy, interrupt,
    peripherals::{self, Peripherals, UART0, UART1},
    prelude::*,
    riscv,
    timer::TimerGroup,
    uart::{config::AtCmdConfig, config::Config, TxRxPins},
    Cpu, Rtc, Uart, IO,
};
use esp_backtrace as _;
use robots_lib::{Cmd, Vec};
use static_cell::StaticCell;

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
async fn run1() {
    loop {
        monitor!("run1");
        send_cmd(Cmd::Ping);
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

#[embassy_executor::task]
async fn run2() {
    loop {
        let cmd = CMD.wait().await;
        monitor!("run2: {cmd:?}");
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();
static SERIAL0: Mutex<RefCell<Option<Uart<UART0>>>> = Mutex::new(RefCell::new(None));
static SERIAL1: Mutex<RefCell<Option<Uart<UART1>>>> = Mutex::new(RefCell::new(None));
static CMD: Signal<CriticalSectionRawMutex, Cmd> = Signal::new();

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    // Disable watchdog timers
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let pins1 = TxRxPins::new_tx_rx(
        io.pins.gpio0.into_push_pull_output(),
        io.pins.gpio1.into_floating_input(),
    );

    let mut serial0 = Uart::new(peripherals.UART0);
    let serial1 = Uart::new_with_config(
        peripherals.UART1,
        Some(Config::default()),
        Some(pins1),
        &clocks,
    );

    serial0.set_at_cmd(AtCmdConfig::new(Some(0), Some(0), None, 0, Some(1)));
    serial0.listen_at_cmd();

    critical_section::with(|cs| {
        SERIAL0.borrow_ref_mut(cs).replace(serial0);
        SERIAL1.borrow_ref_mut(cs).replace(serial1);
    });

    interrupt::enable(
        peripherals::Interrupt::UART0,
        interrupt::Priority::Priority1,
    )
    .unwrap();

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
        spawner.spawn(run1()).ok();
        spawner.spawn(run2()).ok();
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
    CMD.signal(Cmd::from_vec(&mut vec).unwrap());
}
