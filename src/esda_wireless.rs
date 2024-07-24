
use embassy_executor::task;

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_println::{dbg, print, println};
use esp_wifi::esp_now::{self, EspNow, PeerInfo, BROADCAST_ADDRESS};
use smoltcp::wire::{DhcpMessageType, Icmpv4Message};
use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation}, clock::ClockControl, delay::Delay, gpio::{GpioPin, Io}, peripherals::{self, Peripherals}, prelude::*, rng::Rng, system::{Peripheral, SystemControl}, timer::PeriodicTimer
};


#[task]
pub async fn wireless_transmitter(mut esp_now: EspNow<'static>, ) {
    loop {
        let peripherals: Peripherals = Peripherals::take();
        let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

        // Initialise analog read pins
        let read_y_in: GpioPin<35> = io.pins.gpio35; // y-axis
        let read_x_in: GpioPin<34> = io.pins.gpio34; // x-axis
        // Either loop with a timer and send all the data or add channels (not signals) to the parameters which are fired by other tasks when they detect changes to the controls
        // I like the second option better :)
    }
}
