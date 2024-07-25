use core::sync::atomic::AtomicU32;

use embassy_executor::task;
use embassy_sync::{blocking_mutex::{raw::CriticalSectionRawMutex, NoopMutex}, mutex::Mutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_hal::{
    analog::adc::{Adc, AdcConfig, AdcPin, Attenuation}, clock::ClockControl, delay::Delay, gpio::{GpioPin, Io}, i2s::RegisterAccess, peripherals::{self, Peripherals, ADC1, ADC2}, prelude::*, rng::Rng, system::SystemControl, timer::PeriodicTimer
};
use embassy_sync::mutex::MutexGuard;
use crate::{peripheral_extensions::AdcExtension, esda_interface::EsdaControllerStruct};
use esp_hal::gpio::InputPin;

// protect access to EsdaControllerStruct
static CONTROLLER_STATE: Mutex<CriticalSectionRawMutex, EsdaControllerStruct> = Mutex::new(EsdaControllerStruct {
    x_value_pack: 0,
    y_value_pack: 0,
    button_state: 0,
    tog_switch_val: 0,
});

// define a static signal to notify wireless transmission task
pub static CONTROLLER_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[task]
pub async fn update_controller_state(
    mut adc1: Adc<'static, impl AdcExtension>),
    // mut pin_x: AdcPin<impl embedded_hal_02::adc::Channel<ADCI, ID = u8>, )>
{
    let mut pin_value_y: u16 = nb::block(adc1.read_oneshot(&mut adc1_pin)).unwrap();


    loop {
        // replace with actual reading
        
        let new_button_state: u8 = 1;
        let new_tog_switch_val: u8 = 1;


        // lock mutex (CONTROLLER_STATE) and update
        {
            let mut state: MutexGuard<CriticalSectionRawMutex, EsdaControllerStruct> = CONTROLLER_STATE.lock().await;
            let mut changed = false;

            if state.button_state != new_button_state {
                state.set_button_state(new_button_state);
                changed = true;
            }
            if state.tog_switch_val != new_tog_switch_val {
                state.set_button_state(new_tog_switch_val);
                changed = true;
            }
            // signal that struct fields have changed
            if changed {
                CONTROLLER_SIGNAL.signal(());
            }
        }

        // wait for 1 sec
        Timer::after(Duration::from_secs(1)).await;        
    }
}


pub async fn update_controller<P>(
    mut x_pin: AdcPin<P, ADC1>,
    mut y_pin: AdcPin<P, ADC2>,
){
    loop{

    }
}

// for wireless_transmission task
pub async fn get_controller_state() -> EsdaControllerStruct {
    // create a copy of the data instead of a reference to the actual data
    CONTROLLER_STATE.lock().await.clone()
}
