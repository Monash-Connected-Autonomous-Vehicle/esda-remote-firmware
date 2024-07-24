use embassy_executor::task;
use esp_hal::{analog::adc::Adc, peripherals::ADC1};

#[task]
pub async fn joystick_x_listener(mut pin: Adc<'static, ADC1<'static>>) {
    loop {
        let x_value: u16 = pin.read().await.unwrap();
        //pin.
        
        // Either loop with a timer and send all the data or add channels (not signals) to the parameters which are fired by other tasks when they detect changes to the controls
        // I like the second option better :)
    }
} 

pub async fn joystick_y_listener(mut pin:Adc){


}