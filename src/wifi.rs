use crate::CONFIG;

use esp_idf_hal::sys::EspError;
use esp_idf_svc::wifi::{AsyncWifi, ClientConfiguration, EspWifi};

use log::info;

//this code is all very inspired by https://github.com/jasta/esp32-tokio-demo/blob/main/src/main.rs
pub struct WifiLoop<'a> {
    pub wifi: AsyncWifi<EspWifi<'a>>,
}
impl<'a> WifiLoop<'a> {
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
        let wifi = &mut self.wifi;

        loop {
            info!("in the connection_loop");
            wifi.wifi_wait(|wifi| wifi.is_up(), None).await.unwrap();
            info!("trying to connect");
            match wifi.connect().await {
                Ok(_) => (),
                Err(_err) => {
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
