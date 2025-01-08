use esp_idf_svc::{mqtt::client::*, sys::EspError};

use crate::{relay, CONFIG};
use core::time::Duration;
use log::*;
pub fn run(client: &mut EspMqttClient<'_>, connection: &mut EspMqttConnection, sender: smol::channel::Sender<relay::Relay>) -> Result<(), EspError> {
    let topic = &CONFIG.mqtt_topic;

    std::thread::scope(|s| {
        info!("About to start the MQTT client");

        // Need to immediately start pumping the connection for messages, or else subscribe() and publish() below will not work
        // Note that when using the alternative constructor - `EspMqttClient::new_cb` - you don't need to
        // spawn a new thread, as the messages will be pumped with a backpressure into the callback you provide.
        // Yet, you still need to efficiently process each message in the callback without blocking for too long.
        //
        // Note also that if you go to http://tools.emqx.io/ and then connect and send a message to topic
        // "esp-mqtt-demo", the client configured here should receive it.
        // let mut connected = false;
        std::thread::Builder::new()
            .stack_size(6000)
            .spawn_scoped(s, move || {
                info!("MQTT Listening for messages");
                while let Ok(event) = connection.next() {
                    info!("[Queue] Event: {}", event.payload());
                    if let Ok(relay_struct) = postcard::from_bytes(event.payload().to_string().as_bytes()) {
                        if smol::block_on(sender.send(relay_struct)).is_ok() {
                            info!("send to channel")
                        };
                    } else {
                        error!("cannot parse struct");
                    }
                }

                info!("Connection closed");
            })
            .unwrap();

        loop {
            // if !connected {
            if let Err(e) = client.subscribe(topic, QoS::AtMostOnce) {
                error!(
                    "Failed to subscribe to topic \"{topic}\": {e}, retrying..., actual esp_error number {:?}",
                    e
                );
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }
            // else {
            //     info!("Subscribed to topic \"{topic}\"");
            //     connected = true;
            // }
            // }

            // Just to give a chance of our connection to get even the first published message
            std::thread::sleep(Duration::from_secs(1));
            // connection.next()

            let payload = "Hello from esp-mqtt-demo!";

            loop {
                client.enqueue(topic, QoS::AtLeastOnce, true, payload.as_bytes())?;
                // client.enqueue(topic, QoS::AtMostOnce, false, payload.as_bytes())?;

                info!("Published \"{payload}\" to topic \"{topic}\"");

                let sleep_secs = 10;

                info!("Now sleeping for {sleep_secs}s...");
                std::thread::sleep(Duration::from_secs(sleep_secs));
            }
        }
    })
}

pub fn mqtt_create(url: &str, client_id: &str) -> Result<(EspMqttClient<'static>, EspMqttConnection), EspError> {
    let (mqtt_client, mqtt_conn) = EspMqttClient::new(
        url,
        &MqttClientConfiguration {
            client_id: Some(client_id),
            disable_clean_session: false,

            // protocol_version: esp_idf_svc::mqtt::client::MqttProtocolVersion::,
            ..Default::default()
        },
    )?;

    Ok((mqtt_client, mqtt_conn))
}
