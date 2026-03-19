//! Example application for the system-flash bootloader.
//!
//! - Blink task: toggles LED on PD4 every second with defmt status
//! - Main loop: listens on USART1 (TX=PD5, RX=PD6) for tinyboot Probe commands,
//!   reboots into bootloader on receipt

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

mod transport;

use ch32_hal as hal;
use defmt_rtt as _;
use panic_halt as _;

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::gpio::{AnyPin, Level, Output};
use hal::usart::{self, Uart};
use hal::Peri;
use tinyboot_ch32_app::traits::BootClient;
use tinyboot_ch32_app::{frame::Frame, payload_size};

hal::bind_interrupts!(struct Irqs {
    USART1 => hal::usart::InterruptHandler<hal::peripherals::USART1>;
});

const FRAME_SIZE: usize = 64;

#[embassy_executor::task]
async fn blink(pin: Peri<'static, AnyPin>) {
    let mut led = Output::new(pin, Level::Low, Default::default());
    loop {
        led.set_high();
        defmt::info!("LED on");
        Timer::after_millis(1000).await;
        led.set_low();
        defmt::info!("LED off");
        Timer::after_millis(1000).await;
    }
}

#[embassy_executor::main(entry = "qingke_rt::entry")]
async fn main(spawner: Spawner) -> ! {
    let mut config = hal::Config::default();
    config.rcc = hal::rcc::Config::SYSCLK_FREQ_48MHZ_HSI;
    let p = hal::init(config);

    defmt::info!("Confirming boot...");
    tinyboot_ch32_app::BootClient::default().confirm();
    defmt::info!("Boot confirmed.");

    spawner.spawn(blink(p.PD4.into())).unwrap();

    // USART1 async (Remap 0: RX=PD6, TX=PD5)
    let mut uart_config = usart::Config::default();
    uart_config.baudrate = 115200;
    let uart = Uart::new::<0>(
        p.USART1, p.PD6, p.PD5, Irqs, p.DMA1_CH4, p.DMA1_CH5, uart_config,
    )
    .unwrap();
    let (mut tx, mut rx) = uart.split();
    let mut rx = transport::AsyncRx(rx);
    let mut tx = transport::AsyncTx(tx);
    let mut frame = Frame::<{ payload_size(FRAME_SIZE) }>::default();
    let info = tinyboot_ch32_app::AppInfo {
        capacity: 16 * 1024,
        payload_size: payload_size(FRAME_SIZE) as u16,
        erase_size: 1024,
        version: 1,
    };

    defmt::info!("App started. Listening on USART1...");

    loop {
        tinyboot_ch32_app::poll_cmd_async(&mut rx, &mut tx, &mut frame, &info).await;
    }
}
