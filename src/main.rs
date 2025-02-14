#![feature(get_mut_unchecked)]
mod mqtt;
mod relay;
mod sensor;
mod wifi;

use esp_idf_hal::{peripherals::Peripherals, task::block_on};
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

fn main() {
    esp_idf_hal::sys::link_patches();
    EspLogger::initialize_default();

    let config = esp_idf_svc::sys::esp_vfs_eventfd_config_t { max_fds: 15 };
    esp_idf_svc::sys::esp! { unsafe { esp_idf_svc::sys::esp_vfs_eventfd_register(&config) } }.unwrap();

    let peripherals = Peripherals::take().unwrap();
    let modem = peripherals.modem;
    let timer = EspTimerService::new().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();

    let nvs = EspDefaultNvsPartition::take().unwrap();
    let nvs_clone: EspNvsPartition<NvsDefault> = nvs.clone();
    let _wifi: AsyncWifi<EspWifi<'_>> = AsyncWifi::wrap(EspWifi::new(modem, sys_loop.clone(), Some(nvs)).unwrap(), sys_loop, timer).unwrap();

    println!("something from the stack");

    let _sntp = esp_idf_svc::sntp::EspSntp::new_default();
    block_on(run(nvs_clone, _wifi));
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct StructToBeStored<'a> {
    some_bytes: &'a [u8],
    a_str: &'a str,
    a_number: i16,
}
async fn run(nvs: esp_idf_svc::nvs::EspNvsPartition<NvsDefault>, wifi: AsyncWifi<EspWifi<'static>>) {
    let (
        (sender_relay_1, reciever_relay_1),
        (sender_relay_2, reciever_relay_2),
        (sender_relay_3, reciever_relay_3),
        (sender_relay_4, reciever_relay_4),
        (sender_status_led, reciever_status_led),
    ) = (
        crossbeam::channel::bounded(5),
        crossbeam::channel::bounded(5),
        crossbeam::channel::bounded(5),
        crossbeam::channel::bounded(5),
        crossbeam::channel::bounded(5),
    );
    let wifi_task = std::thread::spawn(|| esp_idf_hal::task::block_on(async_wifi_task(wifi)));
    let (mut client, mut conn) = mqtt::mqtt_create(CONFIG.mqtt_url, CONFIG.mqtt_client_id).unwrap();
    let mqtt_task = std::thread::spawn(move || {
        mqtt::run(
            &mut client,
            &mut conn,
            // sender,
            sender_relay_1,
            sender_relay_2,
            sender_relay_3,
            sender_relay_4,
            sender_status_led,
        )
        .unwrap()
    });
    // std::thread::sleep(Duration::from_secs(60));
    let relay_task = std::thread::spawn(|| {
        relay::relay_controller_func(
            nvs,
            (
                reciever_relay_1,
                reciever_relay_2,
                reciever_relay_3,
                reciever_relay_4,
                reciever_status_led,
            ),
        )
    });

    mqtt_task.join().unwrap();
    wifi_task.join().unwrap();
    relay_task.join().unwrap();
    panic!("this shouldn't exist restarting");
}

async fn async_wifi_task(wifi: AsyncWifi<EspWifi<'_>>) {
    let mut wifi_loop: wifi::WifiLoop<'_> = wifi::WifiLoop { wifi };
    wifi_loop.configure().await.unwrap();
    wifi_loop.do_connect_loop().await;
}
