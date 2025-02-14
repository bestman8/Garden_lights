use esp_idf_svc::{mqtt::client::*, sys::EspError};

use crate::relay::Relay;
use crate::{relay, CONFIG};
use core::time::Duration;
use esp_idf_hal::{
    self,
    adc::oneshot::{config, AdcChannelDriver, AdcDriver},
};

use log::*;
pub fn run(
    client: &mut EspMqttClient<'_>,
    connection: &mut EspMqttConnection,
    // sender: smol::channel::Sender<relay::Relay>,
    sender_relay_1: crossbeam::channel::Sender<relay::Relay>,
    sender_relay_2: crossbeam::channel::Sender<relay::Relay>,
    sender_relay_3: crossbeam::channel::Sender<relay::Relay>,
    sender_relay_4: crossbeam::channel::Sender<relay::Relay>,
    sender_status_led: crossbeam::channel::Sender<relay::Relay>,
) -> Result<(), EspError> {
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

                    println!("payload_str :{}", payload_str);

                    let data = payload_str.split("data: Ok(\"").nth(1).unwrap_or_default().chars().collect::<Vec<_>>();
                    let data = data
                        .chunks(2)
                        .map_while(|chars| {
                            let (num_1, num_2) = (chars[0].to_digit(16)? as u8, chars[1].to_digit(16)? as u8);
                            Some((num_1 << 4) | num_2)
                        })
                        .collect::<Vec<_>>();

                    println!("{:?}", data);

                    if let Ok(relay_struct) = postcard::from_bytes::<Relay>(&data) {
                        println!("\n{:?}\n", relay_struct);
                        match relay_struct.number {
                            relay::RelayNumber::Relay1 => sender_relay_1.send(relay_struct).unwrap(),
                            relay::RelayNumber::Relay2 => sender_relay_2.send(relay_struct).unwrap(),
                            relay::RelayNumber::Relay3 => sender_relay_3.send(relay_struct).unwrap(),
                            relay::RelayNumber::Relay4 => sender_relay_4.send(relay_struct).unwrap(),
                            relay::RelayNumber::StatusLed => sender_status_led.send(relay_struct).unwrap(),
                        };

                        info!("as been sent to channel")
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

            std::thread::sleep(Duration::from_secs(1));

            let thing = unsafe { esp_idf_hal::adc::ADC1::new() };
            let adc = AdcDriver::new(thing).unwrap();
            let mut adc_pin = AdcChannelDriver::new(
                &adc,
                unsafe { esp_idf_hal::gpio::Gpio34::new() },
                &config::AdcChannelConfig {
                    attenuation: esp_idf_hal::adc::attenuation::adc_atten_t_ADC_ATTEN_DB_11,
                    ..Default::default()
                },
            )
            .unwrap();

            loop {
                let sleep_secs = 40;
                client.enqueue(topic, QoS::AtLeastOnce, true, adc.read_raw(&mut adc_pin).unwrap().to_string().as_bytes())?;

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
            ..Default::default()
        },
    )?;

    Ok((mqtt_client, mqtt_conn))
}
