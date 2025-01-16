use esp_idf_svc::{mqtt::client::*, sys::EspError};

use crate::{relay, CONFIG};
use core::time::Duration;
use esp_idf_svc::mqtt::client::*;
use esp_idf_svc::timer::{EspAsyncTimer, EspTaskTimerService, EspTimerService};

use log::*;
pub fn async_mqtt_create(url: &str, client_id: &str) -> Result<(EspAsyncMqttClient, EspAsyncMqttConnection), EspError> {
    let (mqtt_client, mqtt_conn) = EspAsyncMqttClient::new(
        url,
        &MqttClientConfiguration {
            client_id: Some(client_id),
            ..Default::default()
        },
    )?;

    Ok((mqtt_client, mqtt_conn))
}

pub async fn async_run(client: &mut EspAsyncMqttClient, connection: &mut EspAsyncMqttConnection, sender: smol::channel::Sender<relay::Relay>) {
    let ex = smol::LocalExecutor::new();

    ex.spawn(async {
        info!("mqtt async working i guess");
        while let Ok(event) = connection.next().await {
            info!("[Queue] Event: {}", event.payload());
            let payload_str = format!("{}", event.payload());
            let data_str = if let Some(data_str) = payload_str.split("data: ").nth(1) {
                if let Some(data) = data_str.split(", details:").next() {
                    data
                } else {
                    continue;
                }
            } else {
                continue;
            };
            println!("payload_str :{}", payload_str);
            println!("data_str :{}", data_str);
            let data: Vec<u8> = data_str
                .trim_matches('[')
                .trim_matches(']')
                .split(", ")
                .map(|x| x.parse::<u8>().unwrap_or(0))
                .collect();
            println!("{:?}", data);

            // let test: Vec<u8> = event.payload().into();
            // info!("{:?}", event.payload().to_string().as_mut_vec());
            if let Ok(relay_struct) = postcard::from_bytes(&data) {
                if esp_idf_hal::task::block_on(sender.send(relay_struct)).is_ok() {
                    info!("sent to channel")
                };
            } else {
                error!("cannot parse struct");
            }
        }
    })
    .detach();

    ex.spawn(async {
        // Using `pin!` is optional, but it optimizes the memory size of the Futures
        loop {
            if let Err(e) = client.subscribe(CONFIG.mqtt_topic, QoS::AtMostOnce).await {
                error!("Failed to subscribe to topic \"{}\": {}, retrying...", CONFIG.mqtt_topic, e);

                // Re-try in 0.5s
                smol::Timer::after(Duration::from_millis(500)).await;

                continue;
            }

            info!("Subscribed to topic \"{}\"", CONFIG.mqtt_topic);

            // Just to give a chance of our connection to get even the first published message
            smol::Timer::after(Duration::from_millis(500)).await;

            let payload = "Hello from esp-mqtt-demo!";

            loop {
                let _ = client.publish(CONFIG.mqtt_topic, QoS::AtMostOnce, false, payload.as_bytes()).await;

                info!("Published \"{payload}\" to topic \"{}\"", CONFIG.mqtt_topic);

                let sleep_secs = 2;

                info!("Now sleeping for {sleep_secs}s...");
                smol::Timer::after(Duration::from_secs(sleep_secs)).await;
            }
        }
    })
    .detach();
    smol::Timer::never().await;
    todo!()
}

pub fn run(client: &mut EspMqttClient<'_>, connection: &mut EspMqttConnection, sender: smol::channel::Sender<relay::Relay>) -> Result<(), EspError> {
    info!("About to start the MQTT client");
    let topic = &CONFIG.mqtt_topic;

    std::thread::scope(|s| {
        info!("About to start the MQTT client");

        std::thread::Builder::new()
            .stack_size(6000)
            .spawn_scoped(s, move || {
                info!("MQTT Listening for messages");
                while let Ok(event) = connection.next() {
                    info!("[Queue] Event: {}", event.payload());
                    let payload_str = format!("{}", event.payload());
                    let data_str = if let Some(data_str) = payload_str.split("data: ").nth(1) {
                        if let Some(data) = data_str.split(", details:").next() {
                            data
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    };
                    println!("payload_str :{}", payload_str);
                    println!("data_str :{}", data_str);
                    let data: Vec<u8> = data_str
                        .trim_matches('[')
                        .trim_matches(']')
                        .split(", ")
                        .map(|x| x.parse::<u8>().unwrap_or(0))
                        .collect();
                    println!("{:?}", data);

                    // let test: Vec<u8> = event.payload().into();
                    // info!("{:?}", event.payload().to_string().as_mut_vec());
                    if let Ok(relay_struct) = postcard::from_bytes(&data) {
                        if esp_idf_hal::task::block_on(sender.send(relay_struct)).is_ok() {
                            info!("sent to channel")
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

                let sleep_secs = 2;

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
