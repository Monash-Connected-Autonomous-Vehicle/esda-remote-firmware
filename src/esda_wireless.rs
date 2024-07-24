use core::f32::consts::E;

use embassy_executor::task;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_println::{dbg, print, println};
use esp_wifi::esp_now::{self, EspNow, PeerInfo, BROADCAST_ADDRESS};
use smoltcp::wire::{DhcpMessageType, Icmpv4Message};

// use crate::{esda_controls::get_controller_state, esda_interface::EsdaControllerStruct};
use crate::esda_controls::{get_controller_state, CONTROLLER_SIGNAL};


#[task]
pub async fn wireless_transmitter(mut esp_now: EspNow<'static>) {
    loop {
        // Either loop with a timer and send all the data or add channels (not signals) to the parameters which are fired by other tasks when they detect changes to the controls
        // I like the second option better :)
        
        // wait for signal 
        CONTROLLER_SIGNAL.wait().await;

        // latest controller data
        let controller_data = get_controller_state().await;

        let data_bytes = controller_data.to_bytes(); // convert to bytes array

        // broadcast (send to all esp32) struct through esp now
        match esp_now.send(&BROADCAST_ADDRESS, &data_bytes) {
            Ok(_) => println!("Data sent successfully: {:?}", data_bytes),
            Err(e) => println!("Failed to send data: {:?}", e),
        }

        // // wait for 1 sec before sending again
        // Timer::after(Duration::from_secs(1)).await;
    }
}
