use core::{pin, sync::atomic::AtomicU32};

use embassy_executor::task;
use embassy_sync::{blocking_mutex::{raw::CriticalSectionRawMutex, NoopMutex, raw::NoopRawMutex}, mutex::Mutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_hal::{
    analog::adc::{Adc, AdcConfig, AdcPin, Attenuation}, clock::ClockControl, delay::Delay, gpio::{GpioPin, Io, InputPin, Pull}, i2s::RegisterAccess, peripherals::{self, Peripherals, ADC1, ADC2}, prelude::*, rng::Rng, system::SystemControl, timer::PeriodicTimer
};
use embassy_sync::mutex::MutexGuard;
use esp_println::println;
use crate::{peripheral_extensions::AdcExtension, esda_interface::EsdaControllerStruct};
use esp_hal::gpio;

// protect access to EsdaControllerStruct
static CONTROLLER_STATE: Mutex<CriticalSectionRawMutex, EsdaControllerStruct> = Mutex::new(EsdaControllerStruct {
    x_value_pack: 0,
    y_value_pack: 0,
    button_state: 0,
    tog_switch_val: 0,
});

// define a static signal to notify wireless transmission task
pub static CONTROLLER_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

// Define a signal to carry the updated controller state
pub static CONTROLLER_STATE_SIGNAL: Signal<CriticalSectionRawMutex, EsdaControllerStruct> = Signal::new();


const X_PIN: u8 = 35;
const Y_PIN: u8 = 34;

#[embassy_executor::task]
async fn run(button_state: &'static mut u8) {
    loop {
        // Print the current state of the button
        esp_println::println!("Button state: {}", button_state);
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

#[task]
pub async fn update_controller_state(
    mut adc_x: Adc<'static, esp_hal::peripherals::ADC1>,
    mut adc_pin_x: AdcPin<GpioPin<X_PIN>, esp_hal::peripherals::ADC1>,
    mut adc_y: Adc<'static, esp_hal::peripherals::ADC2>,
    mut adc_pin_y: AdcPin<GpioPin<Y_PIN>, esp_hal::peripherals::ADC2>,
    mut button_pin: gpio::Gpio23,
    mut estop_pin: gpio::Gpio5,
)   
{
    loop {
        // replace with actual reading
        
        
        // the actual messages button and toggle that will change and are sent via ESDA messages
        let mut new_button_state: u8 = 0;
        let mut new_tog_switch_val: u8 = 0;
        
        // button_pin.is_set_low();

        // lock mutex (CONTROLLER_STATE) and update
        {   

            

            let button_pressed = button_pin.is_low();

            


            // Constantly reads the 
            let mut adc_value_x: u16 = nb::block!(adc_x.read_oneshot(&mut adc_pin_x)).unwrap();
            // let mut pin_value_x: f32 = adc_value_x as f32;
            let mut adc_value_y: u16 = nb::block!(adc_y.read_oneshot(&mut adc_pin_y)).unwrap();
            // let mut pin_value_y: f32 = adc_value_y as f32;
            let mut state: MutexGuard<CriticalSectionRawMutex, EsdaControllerStruct> = CONTROLLER_STATE.lock().await;
            let mut changed = false;

            let mut adc_value_x_8_bit: u8 = adc_value_x as u8; // Truncates if necessary
            let mut adc_value_y_8_bit: u8 = adc_value_y as u8; // Truncates if necessary

            println!("y-value reading = {}, x-value reading = {}", adc_value_y_8_bit, adc_value_x_8_bit);

            // Signalling for the controller
            
            if state.y_value_pack != adc_value_y_8_bit {
                state.y_value_pack = adc_value_y_8_bit;
                changed = true;
            }
            
            if state.x_value_pack != adc_value_x_8_bit {
                state.x_value_pack = adc_value_x_8_bit;
                changed = true;
            }
            if button_pressed {
                new_button_state = new_button_state.wrapping_add(1);
                state.set_button_state(new_button_state);
                changed = true;
                esp_println::println!("Button pressed! Counter: {}\n", new_button_state);
            }
            // if state.button_state != new_button_state {
            //     state.set_button_state(new_button_state);
            //     changed = true;
            // }
            if state.tog_switch_val != new_tog_switch_val {
                state.set_button_state(new_tog_switch_val);
                changed = true;
            }
            // signal that struct fields have changed
            if changed {
                CONTROLLER_SIGNAL.signal(());
                CONTROLLER_STATE_SIGNAL.signal(state.clone());
                // changed = false;
            } else {
                println!("No change being detected");
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
