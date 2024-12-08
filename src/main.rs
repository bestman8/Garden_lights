#![feature(future_join)]

use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::task::block_on;
use esp_idf_svc::timer::{EspAsyncTimer, EspTimerService};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    password: &'static str,
}

// use tokio::{sleep, spawn, Duration};
fn main() {
    esp_idf_hal::sys::link_patches();
    let peripherals = Peripherals::take().unwrap();
    let app_config = CONFIG;
    println!(
        "wifi ssid: {}, wifi password: {}",
        app_config.wifi_ssid, app_config.password
    );

    block_on(run(peripherals))
}

async fn run(peripherals: Peripherals) {
    let mut led_1: PinDriver<'_, Gpio25, Output> =
        PinDriver::output(peripherals.pins.gpio25).unwrap();

    let mut led_2 = PinDriver::output(peripherals.pins.gpio21).unwrap();
    let mut led_3 = PinDriver::output(peripherals.pins.gpio5).unwrap();
    let mut led_4 = PinDriver::output(peripherals.pins.gpio18).unwrap();
    let mut led_5 = PinDriver::output(peripherals.pins.gpio19).unwrap();
    let task_1 = led_blink_async(&mut led_1, 0.5);
    let task_2 = led_blink_async(&mut led_2, 5.0);
    let task_3 = led_blink_async(&mut led_3, 7.0);
    let task_4 = led_blink_async(&mut led_4, 10.0);
    let task_5 = led_blink_async(&mut led_5, 15.0);

    std::future::join!(task_1, task_2, task_3, task_4, task_5).await;
}
// esp_idf_hal::task::CriticalSection::

async fn led_blink_async<T: esp_idf_hal::gpio::Pin>(
    led: &mut PinDriver<'_, T, Output>,
    duration_in_seconds: f32,
) {
    let mut timer: EspAsyncTimer = EspTimerService::new().unwrap().timer_async().unwrap();

    loop {
        use core::time::Duration;

        led.set_high();
        timer
            .after(Duration::from_secs_f32(duration_in_seconds))
            .await
            .unwrap();
        led.set_low();
        timer
            .after(Duration::from_secs_f32(duration_in_seconds))
            .await
            .unwrap();
    }
}
