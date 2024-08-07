//! embassy hello world
//!
//! This is an example of running the embassy executor with multiple tasks
//! concurrently.

//% CHIPS: esp32 esp32c2 esp32c3 esp32c6 esp32h2 esp32s2 esp32s3
//% FEATURES: embassy esp-hal-embassy/integrated-timers

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    clock::ClockControl,
    gpio::{Event, GpioPin, Input, Io, Pull},
    mcpwm::{operator::PwmPinConfig, timer::PwmWorkingMode, McPwm, PeripheralClockConfig},
    peripherals::Peripherals,
    prelude::*,
    rng::Rng,
    system::SystemControl,
    timer::{timg::TimerGroup, ErasedTimer, OneShotTimer, PeriodicTimer},
    uart::{self, config::AtCmdConfig},
    
};
use esp_println::println;
use esp_wifi::{initialize, EspWifiInitFor};

/// Module containing interface types for communicating with controller (via esp-now) and the computer (via serial)
mod esda_interface;

/// Module containing espnow transmit task
mod esda_wireless;

/// Module containing code to handle inputs from joysticks, buttons etc
mod esda_controls;

// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[embassy_executor::task]
async fn run() {
    loop {
        esp_println::println!("Hello world from embassy using esp-hal-async!");
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

#[main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    println!("Beginning Asterius Firmware Initialisation...");
    println!("Initialising Runtime...");
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks, None);
    let timer0 = OneShotTimer::new(timg0.timer0.into());
    let timers = [timer0];
    let timers = mk_static!([OneShotTimer<ErasedTimer>; 1], timers);
    esp_hal_embassy::init(&clocks, timers);

    println!("Starting esp-now Initialisation");
    let timer = PeriodicTimer::new(
        esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1, &clocks, None)
            .timer1
            .into(),
    );

    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    let wifi = peripherals.WIFI;
    let mut esp_now = esp_wifi::esp_now::EspNow::new(&init, wifi).unwrap();
    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    

    // Initialise analog read pins
    let read_y_in: GpioPin<35> = io.pins.gpio35; // y-axis
    let read_x_in: GpioPin<34> = io.pins.gpio34; // x-axis

    let mut adc1_config = AdcConfig::new();
    let mut adc1_pin = adc1_config.enable_pin(read_y_in, Attenuation::Attenuation11dB);
    let mut adc1 = Adc::new(peripherals.ADC1, adc1_config);

    let mut adc2_config = AdcConfig::new();
    let mut adc2_pin = adc2_config.enable_pin(read_x_in, Attenuation::Attenuation11dB);
    let mut adc2: Adc<esp_hal::peripherals::ADC2> = Adc::new(peripherals.ADC2, adc2_config);

    // starts wireless_transmitter 
    spawner.spawn(esda_wireless::wireless_transmitter(esp_now));


    // starts controls for reading analog values
    // spawner.spawn(esda_controls::joystick_x_listener());

    // Occupy the main thread to avoid tripping the watchdog
    loop {
        Timer::after(Duration::from_millis(5_000)).await;
    }
}