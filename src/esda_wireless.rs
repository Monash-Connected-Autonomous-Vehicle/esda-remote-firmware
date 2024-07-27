use core::f32::consts::E;

use embassy_executor::task;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_println::{dbg, print, println};
use esp_wifi::esp_now::{self, EspNow, PeerInfo, BROADCAST_ADDRESS};
use smoltcp::wire::{DhcpMessageType, Icmpv4Message};

// use crate::{esda_controls::get_controller_state, esda_interface::EsdaControllerStruct};
use crate::esda_controls::{CONTROLLER_SIGNAL, get_controller_state, CONTROLLER_STATE_SIGNAL};


#[task]
pub async fn wireless_transmitter(
    mut esp_now: EspNow<'static>,
    controller_transmit_signal: &'static Signal<NoopRawMutex, (f32, f32)>, // Might need to change this
) {
    loop {
        // Either loop with a timer and send all the data or add channels (not signals) to the parameters which are fired by other tasks when they detect changes to the controls
        // I like the second option better :)
        
        // Wait for CONTROLLER_SIGNAL
        CONTROLLER_SIGNAL.wait().await;
        println!("Waiting for CONTROLLER_SIGNAL");
        let controller_data_1 = CONTROLLER_STATE_SIGNAL.wait().await;
        let data_bytes_1 = controller_data_1.to_bytes();


        println!("CONtroller 1 data_bytes_1: {:?}", data_bytes_1);

        // broadcast (send to all esp32) struct through esp now
        match esp_now.send(&BROADCAST_ADDRESS, &data_bytes_1) {
            Ok(_) => println!("Data sent successfully: {:?}", data_bytes_1),
            Err(e) => println!("Failed to send data: {:?}", e),
        }

        // // wait for 1 sec before sending again
        // Timer::after(Duration::from_secs(1)).await;
    }
}
