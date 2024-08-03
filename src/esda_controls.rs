use core::{pin, sync::atomic::AtomicU32};

use embassy_executor::task;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::MutexGuard;
use embassy_sync::{
    blocking_mutex::{raw::CriticalSectionRawMutex, raw::NoopRawMutex, NoopMutex},
    mutex::Mutex,
    signal::Signal,
};
use embassy_time::{Duration, Timer};
use esp_hal::gpio;
use esp_hal::{
    analog::adc::{Adc, AdcConfig, AdcPin, Attenuation},
    clock::ClockControl,
    delay::Delay,
    gpio::{Event, Input, Level, GpioPin, InputPin, Io, Pull},
    i2s::RegisterAccess,
    peripherals::{self, Peripherals, ADC1, ADC2},
    prelude::*,
    rng::Rng,
    system::SystemControl,
    timer::PeriodicTimer,
};
use esp_println::println;

use crate::esda_interface::EsdaControllerStruct;

// protect access to EsdaControllerStruct
static CONTROLLER_STATE: Mutex<CriticalSectionRawMutex, EsdaControllerStruct> =
    Mutex::new(EsdaControllerStruct {
        x_value_pack: 0.0,
        y_value_pack: 0.0,
        button_state: false,
        tog_switch_val: false,
    });

// define a static signal to notify wireless transmission task
// pub static CONTROLLER_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

// Define a signal to carry the updated controller state
// pub static CONTROLLER_STATE_SIGNAL: Signal<CriticalSectionRawMutex, EsdaControllerStruct> = Signal::new();

const X_PIN: u8 = 35;
const Y_PIN: u8 = 34;

#[task]
pub async fn update_controller_state(
    mut adc_x: Adc<'static, esp_hal::peripherals::ADC1>,
    mut adc_pin_x: AdcPin<GpioPin<X_PIN>, esp_hal::peripherals::ADC1>,
    // mut adc_config_x: AdcConfig<esp_hal::peripherals::ADC1>,
    mut adc_y: Adc<'static, esp_hal::peripherals::ADC2>,
    mut adc_pin_y: AdcPin<GpioPin<Y_PIN>, esp_hal::peripherals::ADC2>,
    controller_state_channel: &'static Channel<NoopRawMutex, EsdaControllerStruct, 2>, // Might need to change this
                                                                                       // mut adc_config_y: AdcConfig<esp_hal::peripherals::ADC2>,
                                                                                       // mut gpio: Gpio<'static>,
    mut digital_pin_estop: Input<'static, GpioPin<10>>,
    mut digital_pin_auto: Input<'static, GpioPin<32>>,
) {
    let mut cntr = 0;
    loop {
        // replace with actual reading

        let mut new_button_state: bool = false; // E-Stop button starts off as false. After 3 presses, it becomes true
        let mut new_tog_switch_val: bool = false;

        // lock mutex (CONTROLLER_STATE) and update
        {
            // Constantly reads the analog values of a joystick
            let mut adc_value_x: u16 = nb::block!(adc_x.read_oneshot(&mut adc_pin_x)).unwrap();
            // let mut pin_value_x: f32 = adc_value_x as f32;
            let mut adc_value_y: u16 = nb::block!(adc_y.read_oneshot(&mut adc_pin_y)).unwrap();
            // let mut pin_value_y: f32 = adc_value_y as f32;
            let mut state: MutexGuard<CriticalSectionRawMutex, EsdaControllerStruct> =
                CONTROLLER_STATE.lock().await;
            let mut changed = false;

            let mut adc_value_x_8_bit = adc_value_x as f32; // Truncates if necessary
            let mut adc_value_y_8_bit = adc_value_y as f32; // Truncates if necessary

            println!(
                "y-value reading = {}, x-value reading = {}\n",
                adc_value_y_8_bit, adc_value_x_8_bit
            );

            // Signalling for the controller


            let auto_pressed = digital_pin_auto.is_low();
            println!("Autonomous bool: {}", auto_pressed);
            
            if auto_pressed {
                cntr = cntr + 1;
            }
            println!("Counter value estop: {}\n", cntr);

            if state.y_value_pack != adc_value_y_8_bit {
                state.y_value_pack = adc_value_y_8_bit;
                changed = true;
            }

            if state.x_value_pack != adc_value_x_8_bit {
                state.x_value_pack = adc_value_x_8_bit;
                changed = true;
            }

            if cntr == 3 { // This is for autonomous mode. 
                state.set_button_state(new_button_state);
                new_button_state = !new_button_state;
                cntr = 0;
                changed = true;
            }
            if state.tog_switch_val != new_tog_switch_val {
                state.set_button_state(new_tog_switch_val);
                changed = true;
            }
            // use channel to send struct fields when they have changed
            if changed {
                // CONTROLLER_SIGNAL.signal(());
                // CONTROLLER_STATE_SIGNAL.signal(state.clone());
                controller_state_channel.send(*state).await;
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
