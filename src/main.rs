mod mqtt;
mod relay;
mod wifi;

use esp_idf_hal::{gpio::*, peripherals::Peripherals};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    log::EspLogger,
    nvs::*,
    timer::EspTimerService,
    wifi::{AsyncWifi, EspWifi},
};
use futures_lite::{FutureExt, StreamExt};
use smol::future::poll_fn;
use std::time::Duration;

use log::info;

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
    EspLogger::initialize_default();

    let config = esp_idf_svc::sys::esp_vfs_eventfd_config_t { max_fds: 5 };
    esp_idf_svc::sys::esp! { unsafe { esp_idf_svc::sys::esp_vfs_eventfd_register(&config) } }.unwrap();

    let peripherals = Peripherals::take().unwrap();
    let modem = peripherals.modem;
    let pins: Pins = peripherals.pins;
    let timer = EspTimerService::new().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();

    let nvs = EspDefaultNvsPartition::take().unwrap();
    // let storage = nvs.clone();
    let _wifi: AsyncWifi<EspWifi<'_>> = AsyncWifi::wrap(EspWifi::new(modem, sys_loop.clone(), Some(nvs)).unwrap(), sys_loop, timer).unwrap();
    // let test_storage = "test";
    // let mut nsv_ds = EspNvs::new(storage, "test", true).unwrap();
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

    // println!(
    //     "{:?}",
    //     postcard::from_bytes::<StructToBeStored>(nsv_ds.get_raw("test1", &mut [0; 100]).unwrap().unwrap()).unwrap()
    // );
    // println!("{}", nvs_ds.get_i16());
    // let _wifi: esp_idf_svc::wifi::EspWifi<'_> = wifi::wifi_create(&sys_loop, &nvs, modem).unwrap();
    // // let (mut mqtt_client, mut mqtt_connection) = handled_mqtt_create(0);

    let app_config = CONFIG;
    println!("wifi ssid: {}, wifi password: {}", app_config.wifi_ssid, app_config.password);

    // let ex = smol::LocalExecutor::new();
    let _sntp = esp_idf_svc::sntp::EspSntp::new_default();
    futures_lite::future::block_on(run(pins, _wifi));
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct StructToBeStored<'a> {
    some_bytes: &'a [u8],
    a_str: &'a str,
    a_number: i16,
}
async fn run(pins: Pins, wifi: AsyncWifi<EspWifi<'_>>) {
    let ex = smol::LocalExecutor::new();

    let futures = vec![
        led_blink_async_2(0, pins.gpio25.into(), 0.5),
        // led_blink_async_2(1, pins.gpio21.into(), 5.0),
        // led_blink_async_2(3, pins.gpio18.into(), 10.0),
        // led_blink_async_2(4, pins.gpio5.into(), 15f32),
        // led_blink_async_2(2, pins.gpio19.into(), 7.0),
    ];
    // let futurefds = async_wifi_task(wifi);
    let mut handles = vec![];
    ex.spawn_many(futures, &mut handles);

    // ex.spawn_many(vec![futurefds], &mut handles);
    ex.spawn(async_wifi_task(wifi)).detach();
    // let test = futures_lite::future::zip(futures, async_wifi_task(wifi));

    // // wifi_loop.configure().await.unwrap();
    // let test2 = async move {
    //     async_wifi_task(wifi).await;
    // };

    ex.spawn(async {
        loop {
            println!("current time: {:?}", std::time::SystemTime::now());
            // let time_format = time::format_description::parse("[hour]:[minute]:[second]").unwrap();
            // let current_time = time::OffsetDateTime::now_utc().format(&time_format).unwrap();
            smol::Timer::after(core::time::Duration::from_secs_f32(30.0)).await;
        }
    })
    .detach();
    // ex.spawn_many(vec![test1], &mut handles);

    // ex.run(test1);
    // ex.run(test2);
    ex.run(async move { futures_lite::stream::iter(handles).then(|f| f).collect::<Vec<_>>().await })
        .await;
}

async fn async_wifi_task(wifi: AsyncWifi<EspWifi<'_>>) {
    let mut wifi_loop: wifi::wifi_loop<'_> = wifi::wifi_loop { wifi };
    wifi_loop.configure().await.unwrap();
    wifi_loop.do_connect_loop().await;
}

async fn led_blink_async_2(relay_number: u8, pin: esp_idf_hal::gpio::AnyOutputPin, duration_in_sec: f32) {
    use core::time::Duration;
    let mut led = PinDriver::output(pin).unwrap();

    let mut timer = EspTimerService::new().unwrap().timer_async().unwrap();
    loop {
        println!("current_time utc: {:?}", std::time::SystemTime::now());
        led.set_high().unwrap();
        timer.after(Duration::from_secs_f32(duration_in_sec)).await.unwrap();
        led.set_low().unwrap();
        timer.after(Duration::from_secs_f32(duration_in_sec)).await.unwrap()
    }
}
