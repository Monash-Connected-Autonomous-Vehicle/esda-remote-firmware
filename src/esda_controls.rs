use embassy_executor::task;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation}, clock::ClockControl, delay::Delay, gpio::{GpioPin, Io}, peripherals::{self, Peripherals}, prelude::*, rng::Rng, system::SystemControl, timer::PeriodicTimer
};

use crate::esda_interface::EsdaControllerStruct;

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
pub async fn update_controller_state() {
    
    

    loop {
        // replace with actual reading
        
        let new_button_state: u8 = 1;
        let new_tog_switch_val: u8 = 1;


        // lock mutex (CONTROLLER_STATE) and update
        {
            let mut state: embassy_sync::mutex::MutexGuard<CriticalSectionRawMutex, EsdaControllerStruct> = CONTROLLER_STATE.lock().await;
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

// for wireless_transmission task
pub async fn get_controller_state() -> EsdaControllerStruct {
    // create a copy of the data instead of a reference to the actual data
    CONTROLLER_STATE.lock().await.clone()
}