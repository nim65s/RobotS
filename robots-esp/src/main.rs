//! embassy_hello_world + serial_interrupts

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::{cell::RefCell, fmt::Write};
use critical_section::Mutex;
use embassy_executor::Executor;
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
use static_cell::StaticCell;

#[embassy_executor::task]
async fn run1() {
    loop {
        critical_section::with(|cs| {
            let mut serial0 = SERIAL0.borrow_ref_mut(cs);
            let serial0 = serial0.as_mut().unwrap();
            writeln!(serial0, "run1 serial0\r").ok();
        });
        critical_section::with(|cs| {
            let mut serial1 = SERIAL1.borrow_ref_mut(cs);
            let serial1 = serial1.as_mut().unwrap();
            writeln!(serial1, "run1 serial1\r").ok();
        });
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

#[embassy_executor::task]
async fn run2() {
    loop {
        critical_section::with(|cs| {
            let mut serial0 = SERIAL0.borrow_ref_mut(cs);
            let serial0 = serial0.as_mut().unwrap();
            writeln!(serial0, "run2 serial0\r").ok();
        });
        Timer::after(Duration::from_millis(5_000)).await;
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();
static SERIAL0: Mutex<RefCell<Option<Uart<UART0>>>> = Mutex::new(RefCell::new(None));
static SERIAL1: Mutex<RefCell<Option<Uart<UART1>>>> = Mutex::new(RefCell::new(None));

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

    let serial0 = Uart::new(peripherals.UART0);
    let mut serial1 = Uart::new_with_config(
        peripherals.UART1,
        Some(Config::default()),
        Some(pins1),
        &clocks,
    );

    serial1.set_at_cmd(AtCmdConfig::new(None, None, None, b'a', None));
    serial1.listen_at_cmd();

    embassy::init(&clocks, timer_group0.timer0);

    critical_section::with(|cs| {
        SERIAL0.borrow_ref_mut(cs).replace(serial0);
        SERIAL1.borrow_ref_mut(cs).replace(serial1);
    });

    interrupt::enable(
        peripherals::Interrupt::UART1,
        interrupt::Priority::Priority1,
    )
    .unwrap();
    interrupt::set_kind(
        Cpu::ProCpu,
        interrupt::CpuInterrupt::Interrupt1, // Interrupt 1 handles priority one interrupts
        interrupt::InterruptKind::Edge,
    );

    unsafe {
        riscv::interrupt::enable();
    }

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(run1()).ok();
        spawner.spawn(run2()).ok();
    });
}

#[interrupt]
fn UART1() {
    let mut cnt = 0;
    critical_section::with(|cs| {
        let mut serial1 = SERIAL1.borrow_ref_mut(cs);
        let serial1 = serial1.as_mut().unwrap();

        while let nb::Result::Ok(_c) = serial1.read() {
            cnt += 1;
        }
        serial1.reset_at_cmd_interrupt();
    });
    critical_section::with(|cs| {
        let mut serial0 = SERIAL0.borrow_ref_mut(cs);
        let serial0 = serial0.as_mut().unwrap();
        writeln!(serial0, "Read {} bytes\r", cnt,).ok();
    });
}
