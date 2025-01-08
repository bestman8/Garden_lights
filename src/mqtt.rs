use esp_idf_hal::sys::esp_netif_is_netif_up;
use esp_idf_svc::{mqtt::client::*, sys::EspError};

// pub fn mqtt_create(url: &str, client_id: &str) -> Result<MqttStruct, EspError> {
//     let (mqtt_client, mqtt_conn) = EspAsyncMqttClient::new(
//         url,
//         &MqttClientConfiguration {
//             client_id: Some(client_id),
//             ..Default::default()
//         },
//     )?;

//     Ok(MqttStruct {
//         client: mqtt_client,
//         connection: mqtt_conn,
//     })
// }

// pub struct MqttStruct {
//     client: EspAsyncMqttClient,
//     connection: EspAsyncMqttConnection,
// }
// impl MqttStruct {
//     pub async fn run(&mut self, topic: &str) {
//         let ex = smol::LocalExecutor::new();

//         println!("mqtt started");
//         ex.spawn(async {
//             loop {
//                 while let Ok(event) = self.connection.next().await {
//                     println!("got: {}", event.payload());
//                 }
//             }
//         })
//         .detach();

//         self.client
//             .publish(topic, QoS::AtMostOnce, false, "does this work from the esp32".as_bytes())
//             .await
//             .unwrap();
//         ex.spawn(async {
//             loop {
//                 // if esp_idf_svc::netif::EspNetif::
//                 if let Err(_e) = self.client.subscribe(topic, QoS::AtLeastOnce).await {
//                     println!("retrying in 10 sec");
//                     std::thread::sleep(core::time::Duration::from_secs(10)).await;
//                 }
//             }
//         })
//         .detach();
//     }
// }
use core::time::Duration;
use log::*;
use std::thread;

pub fn run(client: &mut EspMqttClient<'_>, connection: &mut EspMqttConnection) -> Result<(), EspError> {
    let topic = "program";

    std::thread::scope(|s| {
        info!("About to start the MQTT client");

        // Need to immediately start pumping the connection for messages, or else subscribe() and publish() below will not work
        // Note that when using the alternative constructor - `EspMqttClient::new_cb` - you don't need to
        // spawn a new thread, as the messages will be pumped with a backpressure into the callback you provide.
        // Yet, you still need to efficiently process each message in the callback without blocking for too long.
        //
        // Note also that if you go to http://tools.emqx.io/ and then connect and send a message to topic
        // "esp-mqtt-demo", the client configured here should receive it.
        let mut connected = false;
        std::thread::Builder::new()
            .stack_size(6000)
            .spawn_scoped(s, move || {
                info!("MQTT Listening for messages");

                while let Ok(event) = connection.next() {
                    info!("[Queue] Event: {}", event.payload());
                }

                info!("Connection closed");
            })
            .unwrap();

        loop {
            if !connected {
                if let Err(e) = client.subscribe(topic, QoS::AtMostOnce) {
                    error!(
                        "Failed to subscribe to topic \"{topic}\": {e}, retrying..., actual esp_error number {:?}",
                        e
                    );
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                } else {
                    info!("Subscribed to topic \"{topic}\"");
                    connected = true;
                    // println!("fsdfs");
                }
            }

            // Just to give a chance of our connection to get even the first published message
            std::thread::sleep(Duration::from_secs(1));
            // connection.next()

            let payload = "Hello from esp-mqtt-demo!";

            // loop {
            //     client.enqueue(topic, QoS::AtMostOnce, false, payload.as_bytes())?;

            //     info!("Published \"{payload}\" to topic \"{topic}\"");

            //     let sleep_secs = 2;

            //     info!("Now sleeping for {sleep_secs}s...");
            //     std::thread::sleep(Duration::from_secs(sleep_secs));
            // }
        }
    })
}

pub fn mqtt_create(url: &str, client_id: &str) -> Result<(EspMqttClient<'static>, EspMqttConnection), EspError> {
    let (mqtt_client, mqtt_conn) = EspMqttClient::new(
        url,
        &MqttClientConfiguration {
            client_id: Some(client_id),
            ..Default::default()
        },
    )?;

    Ok((mqtt_client, mqtt_conn))
}
