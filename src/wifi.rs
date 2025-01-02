use crate::CONFIG;

use esp_idf_hal::sys::EspError;
use esp_idf_svc::{
    log::EspLogger,
    wifi::{AsyncWifi, ClientConfiguration, EspWifi},
};
use std::time::Duration;

use log::info;

async fn start_wifi_loop(wifi: AsyncWifi<EspWifi<'_>>) {
    let mut wifi_loop = wifi_loop { wifi };
    wifi_loop.do_connect_loop();
}

//this code is all very inspired by https://github.com/jasta/esp32-tokio-demo/blob/main/src/main.rs
pub struct wifi_loop<'a> {
    pub wifi: AsyncWifi<EspWifi<'a>>,
}
impl<'a> wifi_loop<'a> {
    pub async fn configure(&mut self) -> Result<(), EspError> {
        self.wifi
            .set_configuration(&esp_idf_svc::wifi::Configuration::Client(ClientConfiguration {
                ssid: CONFIG.wifi_ssid.try_into().unwrap(),
                password: CONFIG.password.try_into().unwrap(),
                ..Default::default()
            }))?;

        self.wifi.start().await
    }

    pub async fn do_connect_loop(&mut self) {
        let mut wifi = &mut self.wifi;

        loop {
            info!("in the connection_loop");
            wifi.wifi_wait(|wifi| wifi.is_up(), None).await.unwrap();
            info!("trying to connect");
            match wifi.connect().await {
                Ok(_) => (),
                Err(err) => {
                    // info!("error occured while connecting retrying in 10 sec");
                    // smol::Timer::after(Duration::from_secs_f32(10.0)).await;

                    continue;
                }
            }
            info!("waiting for connection");
            wifi.wait_netif_up().await.unwrap()
        }
    }
}
