// #![feature(future_join)]

mod mqtt;
mod relay;
mod wifi;

// use embedded_svc::mqtt::client::asynch::Client;
use esp_idf_hal::{gpio::*, peripherals::Peripherals, task::block_on};
use esp_idf_svc::{eventloop::EspSystemEventLoop, fs, nvs::*, timer::EspTimerService};
use futures_lite::StreamExt;
use std::io::Write;

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

    let sys_loop: esp_idf_svc::eventloop::EspEventLoop<esp_idf_svc::eventloop::System> = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    // let nsv_testpartition = EspCustomNvsPartition::take("my_data").unwrap();

    // let test_storage = "test";
    let mut nsv_ds = EspNvs::new(nvs, "test", true).unwrap();
    // {
    //     let key_raw_struct_data = StructToBeStored {
    //         some_bytes: &[1, 2, 3, 4],
    //         a_str: "this is from storage",
    //         a_number: 42,
    //     };
    //     use postcard::to_vec;

    //     nsv_ds
    //         .set_raw("test1", &to_vec::<StructToBeStored, 100>(&key_raw_struct_data).unwrap())
    //         .unwrap();
    // }

    println!("something from the stack");
    println!(
        "{:?}",
        postcard::from_bytes::<StructToBeStored>(nsv_ds.get_raw("test1", &mut [0; 100]).unwrap().unwrap()).unwrap()
    );
    // println!("{}", nvs_ds.get_i16());
    // let _wifi: esp_idf_svc::wifi::EspWifi<'_> = wifi::wifi_create(&sys_loop, &nvs, modem).unwrap();
    // // let (mut mqtt_client, mut mqtt_connection) = handled_mqtt_create(0);

    let app_config = CONFIG;
    println!("wifi ssid: {}, wifi password: {}", app_config.wifi_ssid, app_config.password);

    // block_on(run(pins))
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct StructToBeStored<'a> {
    some_bytes: &'a [u8],
    a_str: &'a str,
    a_number: i16,
}
async fn run(pins: Pins) {
    let ex = smol::LocalExecutor::new();
    let futures = vec![
        led_blink_async_2(0, pins.gpio25.into(), 0.5),
        led_blink_async_2(1, pins.gpio21.into(), 5.0),
        led_blink_async_2(2, pins.gpio19.into(), 7.0),
        led_blink_async_2(3, pins.gpio18.into(), 10.0),
        led_blink_async_2(4, pins.gpio5.into(), 15f32),
    ];
    let mut handles = vec![];
    ex.spawn_many(futures, &mut handles);
    ex.run(async move { futures_lite::stream::iter(handles).then(|f| f).collect::<Vec<_>>().await })
        .await;
}

async fn led_blink_async_2(relay_number: u8, pin: esp_idf_hal::gpio::AnyOutputPin, duration_in_sec: f32) {
    use core::time::Duration;
    let mut test = PinDriver::output(pin).unwrap();

    let mut timer = EspTimerService::new().unwrap().timer_async().unwrap();
    loop {
        test.set_high().unwrap();
        timer.after(Duration::from_secs_f32(duration_in_sec)).await.unwrap();
        test.set_low().unwrap();
        timer.after(Duration::from_secs_f32(duration_in_sec)).await.unwrap()
    }
}
