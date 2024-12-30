// #![feature(future_join)]

mod mqtt;
mod wifi;
use anyhow::Ok;
// use embedded_svc::mqtt::client::asynch::Client;
use esp_idf_hal::delay::{self, Delay};
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::task::block_on;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::fs;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::timer::{EspAsyncTimer, EspTimerService};
use futures_lite::StreamExt;
use smol::Async;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    password: &'static str,

    #[default("")]
    mqtt_url: &'static str,
    #[default("")]
    mqtt_client_id: &'static str,
}

// use tokio::{sleep, spawn, Duration};
fn main() {
    esp_idf_hal::sys::link_patches();
    let peripherals = Peripherals::take().unwrap();
    let modem = peripherals.modem;
    let pins: Pins = peripherals.pins;

    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    // let _wifi: esp_idf_svc::wifi::EspWifi<'_> = wifi::wifi_create(&sys_loop, &nvs, modem).unwrap();
    // // let (mut mqtt_client, mut mqtt_connection) = handled_mqtt_create(0);

    let app_config = CONFIG;
    println!(
        "wifi ssid: {}, wifi password: {}",
        app_config.wifi_ssid, app_config.password
    );

    block_on(run(pins))
}

async fn run(pins: Pins) {
    let ex = smol::LocalExecutor::new();
    let futures = vec![
        led_blink_async_2(pins.gpio25.into(), 0.5),
        led_blink_async_2(pins.gpio21.into(), 5.0),
        led_blink_async_2(pins.gpio5.into(), 7.0),
        led_blink_async_2(pins.gpio18.into(), 10.0),
        led_blink_async_2(pins.gpio19.into(), 15f32),
    ];
    let mut handles = vec![];
    ex.spawn_many(futures, &mut handles);
    ex.run(async move {
        futures_lite::stream::iter(handles)
            .then(|f| f)
            .collect::<Vec<_>>()
            .await
    })
    .await;
}

async fn led_blink_async_2(pin: esp_idf_hal::gpio::AnyOutputPin, duration_in_sec: f32) {
    use core::time::Duration;
    let mut test = PinDriver::output(pin).unwrap();

    let mut timer = EspTimerService::new().unwrap().timer_async().unwrap();
    loop {
        test.set_high().unwrap();
        timer
            .after(Duration::from_secs_f32(duration_in_sec))
            .await
            .unwrap();
        test.set_low().unwrap();
        timer
            .after(Duration::from_secs_f32(duration_in_sec))
            .await
            .unwrap()
    }
}
