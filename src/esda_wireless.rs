use core::f32::consts::E;

use embassy_executor::task;
use embassy_sync::channel::Channel;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_println::{dbg, print, println};
use esp_wifi::esp_now::{self, EspNow, PeerInfo, BROADCAST_ADDRESS};
use smoltcp::wire::{DhcpMessageType, Icmpv4Message};

use crate::esda_interface::EsdaControllerStruct;
use crate::esda_interface::{self, ESDAMessage};

// use crate::{esda_controls::get_controller_state, esda_interface::EsdaControllerStruct};
// use crate::{esda_controls::{get_controller_state, CONTROLLER_SIGNAL, CONTROLLER_STATE_SIGNAL}, esda_interface::EsdaControllerStruct};

#[task]
pub async fn wireless_transmitter(
    mut esp_now: EspNow<'static>,
    controller_transmit_signal: &'static Signal<NoopRawMutex, (f32, f32)>, // Might need to change this
    controller_state_channel: &'static Channel<NoopRawMutex, EsdaControllerStruct, 2>,
) {
    loop {
        // // Either loop with a timer and send all the data or add channels (not signals) to the parameters which are fired by other tasks when they detect changes to the controls
        // // I like the second option better :)

        // // Wait for info to come from channel
        // controller_state_channel.wait().await;
        // println!("Waiting for CONTROLLER_SIGNAL");
        // let controller_data_1 = CONTROLLER_STATE_SIGNAL.wait().await;
        // let data_bytes_1 = controller_data_1.to_bytes();

        // println!("CONtroller 1 data_bytes_1: {:?}", data_bytes_1);

        // // broadcast (send to all esp32) struct through esp now
        // match esp_now.send(&BROADCAST_ADDRESS, &data_bytes_1) {
        //     Ok(_) => println!("Data sent successfully: {:?}", data_bytes_1),
        //     Err(e) => println!("Failed to send data: {:?}", e),
        // }

        // // // wait for 1 sec before sending again
        // Timer::after(Duration::from_secs(1)).await;

        // Wait for a message from the channel and destructure it to get the inner states
        let EsdaControllerStruct {
            x_value_pack,
            y_value_pack,
            button_state,
            tog_switch_val,
        } = controller_state_channel.receive().await;

        let steer_message = ESDAMessage {
            id: esda_interface::ESDAMessageID::SteerAmount,
            data: map_analog_value(x_value_pack),
        }
        .to_le_bytes();

        let throttle_message_left = ESDAMessage {
            id: esda_interface::ESDAMessageID::SetTargetVelLeft,
            data: map_analog_value(y_value_pack),
        }
        .to_le_bytes();

        let throttle_message_right = ESDAMessage {
            id: esda_interface::ESDAMessageID::SetTargetVelRight,
            // Copy the data from the left
            data: map_analog_value(y_value_pack),
        }
        .to_le_bytes();

        // // TODO?: Change to estop_state later on
        // let mut estop_state: Option<ESDAMessage> = if button_state {
        //     Some(ESDAMessage {
        //         id: esda_interface::ESDAMessageID::SetTargetVelRight,
        //         // Copy the data from the left
        //         ..throttle_message_left
        //     })
        // } else {
        //     None
        // };

        // TODO?: Change to estop_state later on 
        if button_state {
            let estop_message = ESDAMessage {
                id: esda_interface::ESDAMessageID::ESTOP,
                // Copy the data from the left
                data: 1.0,
            }
            .to_le_bytes();
            match esp_now.send(&BROADCAST_ADDRESS, &estop_message) {
                Ok(_) => println!("ESTOP message sent successfully: {:?}", &estop_message),
                Err(e) => println!(
                    "Failed to send ESTOP message, tell Sam he was too confident: {:?}", e
                ),
            }
        }

        // Make our send buffer
        let mut send_buffer: [u8; 24] = [0; 24];
        // Copy the velocities into it
        send_buffer[0..=7].copy_from_slice(&steer_message[0..=7]);
        println!("Sending steering message: {:b}", u64::from_le_bytes(steer_message));
        send_buffer[8..=15].copy_from_slice(&throttle_message_left[0..=7]);
        println!("Sending throttle message left: {:b}", u64::from_le_bytes(throttle_message_left));
        send_buffer[16..=23].copy_from_slice(&throttle_message_right[0..=7]);
        println!("Sending throttle message right: {:b}", u64::from_le_bytes(throttle_message_right));

        match esp_now.send(&BROADCAST_ADDRESS, &steer_message) {
            Ok(_) => println!("Steer Data sent successfully: {:?}", &steer_message),
            Err(e) => println!("Failed to send steer data: {:?}", e),
        }
        match esp_now.send(&BROADCAST_ADDRESS, &throttle_message_left) {
            Ok(_) => println!("Left throttle Data sent successfully: {:?}", &throttle_message_left),
            Err(e) => println!("Failed to send left throttle data: {:?}", e),
        }
        match esp_now.send(&BROADCAST_ADDRESS, &throttle_message_right) {
            Ok(_) => println!("right throttle Data sent successfully: {:?}", &throttle_message_right),
            Err(e) => println!("Failed to send right throttle data: {:?}", e),
        }
    }
}

// maps analog values to values for ESC
fn map_analog_value(value: f32) -> f32 {
    let min_analog: f32 = 0.0;
    let max_analog: f32 = 255.0;
    let min_mapped: f32 = 150.0;
    let max_mapped: f32 = 1670.0;
    let deadzone_lb: f32 = 1500.0;
    let deadzone_ub: f32 = 1550.0;

    let mapped_value =
        min_mapped + (value - min_analog) * (max_mapped - min_mapped) / (max_analog - min_analog);
    if mapped_value >= deadzone_lb && mapped_value <= deadzone_ub {
        return (deadzone_lb + deadzone_ub) / 2.0;
    }
    mapped_value
}
