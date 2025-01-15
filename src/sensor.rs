use esp_idf_hal::{
    self,
    adc::oneshot::{config, AdcChannelDriver, AdcDriver},
};

// pub fn setup_sensor(channl: smol::channel::Sender<u16>) {
pub fn setup_sensor(_pin: i32) {
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
        log::info!("value from temp_sensor: {}", adc.read_raw(&mut adc_pin).unwrap());
        // channl.send(adc.read_raw(&mut adc_pin).unwrap());
        std::thread::sleep(core::time::Duration::from_secs(1));
    }
}
