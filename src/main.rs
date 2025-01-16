#![feature(get_mut_unchecked)]
mod mqtt;
mod relay;
mod sensor;
mod wifi;

use std::time::Duration;

use esp_idf_hal::{gpio::*, peripherals::Peripherals, task::thread};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    log::EspLogger,
    nvs::*,
    timer::EspTimerService,
    wifi::{AsyncWifi, EspWifi},
};

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
    #[default("")]
    mqtt_topic: &'static str,
}
// struct SharedData {
//     pins: esp_idf_hal::gpio::Pins,
//     ledc: esp_idf_hal::ledc::LEDC,
// }

// use tokio::{sleep, spawn, Duration};
fn main() {
    esp_idf_hal::sys::link_patches();
    EspLogger::initialize_default();

    let config = esp_idf_svc::sys::esp_vfs_eventfd_config_t { max_fds: 15 };
    esp_idf_svc::sys::esp! { unsafe { esp_idf_svc::sys::esp_vfs_eventfd_register(&config) } }.unwrap();

    let peripherals = Peripherals::take().unwrap();
    let modem = peripherals.modem;
    let pins = peripherals.pins;
    let timer = EspTimerService::new().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    // let shared_data = shared_data {
    //     pins: peripherals.pins,
    //     ledc: peripherals.ledc,
    // };

    let nvs = EspDefaultNvsPartition::take().unwrap();
    let nvs_clone = nvs.clone();
    let _wifi: AsyncWifi<EspWifi<'_>> = AsyncWifi::wrap(EspWifi::new(modem, sys_loop.clone(), Some(nvs)).unwrap(), sys_loop, timer).unwrap();
    // let test_storage = "test";
    // let mut nsv_ds: EspNvs<NvsDefault> = EspNvs::new(nvs_clone, "test", true).unwrap();
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
    let usedstorage = esp_idf_svc::nvs::EspNvs::new(nvs_clone, "relays", true).unwrap();
    futures_lite::future::block_on(run(usedstorage, pins, _wifi));
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct StructToBeStored<'a> {
    some_bytes: &'a [u8],
    a_str: &'a str,
    a_number: i16,
}
async fn run(nvs: esp_idf_svc::nvs::EspNvs<esp_idf_svc::nvs::NvsDefault>, pins: Pins, wifi: AsyncWifi<EspWifi<'static>>) {
    let ex = smol::LocalExecutor::new();
    let (sender, reciever) = smol::channel::unbounded();
    std::thread::spawn(|| esp_idf_hal::task::block_on(async_wifi_task(wifi)));
    let (mut client, mut conn) = mqtt::mqtt_create(CONFIG.mqtt_url, CONFIG.mqtt_client_id).unwrap();
    std::thread::spawn(move || mqtt::run(&mut client, &mut conn, sender).unwrap());
    std::thread::sleep(Duration::from_secs(60));
    std::thread::spawn(|| relay::relay_controller_func(nvs, pins, reciever));
    // std::thread::spawn(|| loop {
    //     let time_format = time::format_description::parse("[hour]:[minute]:[second]").unwrap();
    //     let current_time = time::OffsetDateTime::now_utc().format(&time_format).unwrap();
    //     println!("current time: {:?}", current_time);
    //     std::thread::sleep(core::time::Duration::from_secs_f32(30.0));
    // });

    // ex.spawn(relay::relay_controller_func(nvs, pins, reciever)).detach();
    // std::thread::spawn(|| relay::relay_controller_func(nvs, pins, reciever));

    // ex.spawn(async_wifi_task(wifi)).detach();
    // std::thread::spawn(|| {
    //     let (mut client, mut conn) = mqtt::async_mqtt_create(CONFIG.mqtt_url, CONFIG.mqtt_client_id).unwrap();

    //     esp_idf_hal::task::block_on(mqtt::async_run(&mut client, &mut conn, sender));
    // });
    smol::Timer::never().await;
    println!("THIS HAPPENS AFTER NEVER");
}

async fn async_wifi_task(wifi: AsyncWifi<EspWifi<'_>>) {
    let mut wifi_loop: wifi::WifiLoop<'_> = wifi::WifiLoop { wifi };
    wifi_loop.configure().await.unwrap();
    wifi_loop.do_connect_loop().await;
}

async fn _led_blink_async_2(_relay_number: u8, pin: esp_idf_hal::gpio::AnyOutputPin, duration_in_sec: f32) {
    use core::time::Duration;
    let mut led = PinDriver::output(pin).unwrap();

    let mut timer = EspTimerService::new().unwrap().timer_async().unwrap();
    loop {
        // println!("current_time utc: {:?}", std::time::SystemTime::now());
        led.set_high().unwrap();
        timer.after(Duration::from_secs_f32(duration_in_sec)).await.unwrap();
        led.set_low().unwrap();
        timer.after(Duration::from_secs_f32(duration_in_sec)).await.unwrap()
    }
}
