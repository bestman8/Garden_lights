// use esp_idf_hal::{
//     self,
//     adc::oneshot::{config, AdcChannelDriver, AdcDriver},
// };

// // pub fn setup_sensor(channl: smol::channel::Sender<u16>) {
// pub fn setup_sensor<'a>(
//     _pin: i32,
// ) -> (
//     AdcDriver<'a, esp_idf_hal::adc::ADC1>,
//     AdcChannelDriver<'a, esp_idf_hal::gpio::Gpio34, 'a AdcDriver<'a, esp_idf_hal::adc::ADC1>>,
// ) {
//     let thing = unsafe { esp_idf_hal::adc::ADC1::new() };
//     let adc = AdcDriver::new(thing).unwrap();
//     let mut adc_pin = AdcChannelDriver::new(
//         &adc,
//         unsafe { esp_idf_hal::gpio::Gpio34::new() },
//         &config::AdcChannelConfig {
//             attenuation: esp_idf_hal::adc::attenuation::adc_atten_t_ADC_ATTEN_DB_11,
//             ..Default::default()
//         },
//     )
//     .unwrap();
//     return (adc, adc_pin);

//     // return_sensor_data(adc, adc_pin)
// }

// fn return_sensor_data<'a>(
//     adc: AdcDriver<'a, esp_idf_hal::adc::ADC1>,
//     adc_pin: &mut AdcChannelDriver<'a, esp_idf_hal::gpio::Gpio34, &AdcDriver<'a, esp_idf_hal::adc::ADC1>>,
// ) -> u16 {
//     adc.read_raw(adc_pin).unwrap()
//     // loop {
//     // log::info!("value from temp_sensor: {}", adc.read_raw(&mut adc_pin).unwrap());
//     // channl.send(adc.read_raw(&mut adc_pin).unwrap());
//     // std::thread::sleep(core::time::Duration::from_secs(1));
//     // }
// }
