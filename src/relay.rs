use chrono::Weekday;
use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_svc::nvs;
use log::{error, log, warn};
use time::{convert::Week, Time};

use crate::relay;
// #[derive(serde::Serialize, serde::Deserialize)]
pub struct Relays {
    relay_1: RelayWithPin,
    relay_2: RelayWithPin,
    relay_3: RelayWithPin,
    relay_4: RelayWithPin,
    status_led: RelayWithPin,
}

impl Relays {
    async fn update(&mut self, reciever: smol::channel::Receiver<Relay>) {
        use RelayNumber::*;
        if let Ok(obtained_relay) = reciever.recv().await {
            match obtained_relay.number {
                Relay1 => self.relay_1.custom_from(obtained_relay),
                Relay2 => self.relay_2.custom_from(obtained_relay),
                Relay3 => self.relay_3.custom_from(obtained_relay),
                Relay4 => self.relay_4.custom_from(obtained_relay),
                StatusLed => self.status_led.custom_from(obtained_relay),
                _ => {
                    warn!("this shouldn't exist")
                }
            }
        }
    }

    pub fn init(
        relay_1_pin: AnyOutputPin,
        relay_2_pin: AnyOutputPin,
        relay_3_pin: AnyOutputPin,
        relay_4_pin: AnyOutputPin,
        status_led_pin: AnyOutputPin,
        nvs: &esp_idf_svc::nvs::EspNvs<nvs::NvsDefault>,
    ) -> Relays {
        let relay_1 = Relay::init(RelayNumber::Relay1, nvs);
        let relay_2 = Relay::init(RelayNumber::Relay2, nvs);
        let relay_3 = Relay::init(RelayNumber::Relay3, nvs);
        let relay_4 = Relay::init(RelayNumber::Relay4, nvs);
        let status_led = Relay::init(RelayNumber::StatusLed, nvs);

        Relays {
            relay_1: relay_1.to_relay_with_pin(relay_1_pin),
            relay_2: relay_2.to_relay_with_pin(relay_2_pin),
            relay_3: relay_3.to_relay_with_pin(relay_3_pin),
            relay_4: relay_4.to_relay_with_pin(relay_4_pin),
            status_led: status_led.to_relay_with_pin(status_led_pin),
        }
    }
}

impl RelayWithPin {
    fn custom_from(&mut self, relay: Relay) {
        self.condition = relay.condition;
        self.days_off_the_week = relay.days_off_the_week;
        self.operating_months = relay.operating_months;
        self.exclude_times = relay.exclude_times;
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Relay {
    number: RelayNumber,
    condition: Condition,
    days_off_the_week: DaysOffTheWeek,
    operating_months: Month,
    exclude_times: Option<[TimeOfDay; 2]>,
}
impl Relay {
    fn new(number: RelayNumber) -> Relay {
        // use time::{Month, Weekday};?
        Relay {
            number: number,
            condition: Condition::Time(None),
            days_off_the_week: DaysOffTheWeek::all(),
            operating_months: Month::all(),
            exclude_times: None,
        }
    }

    pub fn init(number: RelayNumber, nvs: &esp_idf_svc::nvs::EspNvs<nvs::NvsDefault>) -> Relay {
        let name = match number {
            RelayNumber::Relay1 => "Relay1",
            RelayNumber::Relay2 => "Relay2",
            RelayNumber::Relay3 => "Relay3",
            RelayNumber::Relay4 => "Relay4",
            RelayNumber::StatusLed => "StatusLed",
        };
        match nvs.get_raw(name, &mut [0; 512]) {
            Ok(bytes) => {
                match postcard::from_bytes(bytes.unwrap()) {
                    Ok(fdks) => {
                        return fdks;
                    }
                    Err(_) => {
                        return Relay::new(number);
                    }
                };
            }
            Err(_) => {
                return Relay::new(number);
            }
        };
    }

    fn get_from_storage() {}

    fn set_to_storage() {}

    fn to_relay_with_pin(self, pin: esp_idf_hal::gpio::AnyOutputPin) -> RelayWithPin {
        RelayWithPin {
            relay: pin,
            condition: self.condition,
            days_off_the_week: self.days_off_the_week,
            operating_months: self.operating_months,
            exclude_times: self.exclude_times,
        }
    }

    fn from_storage() {
        todo!()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
struct Month {
    jan: bool,
    feb: bool,
    mar: bool,
    apr: bool,
    may: bool,
    jun: bool,
    jul: bool,
    aug: bool,
    sep: bool,
    oct: bool,
    nov: bool,
    dec: bool,
}

impl Month {
    fn all() -> Self {
        Month {
            jan: true,
            feb: true,
            mar: true,
            apr: true,
            may: true,
            jun: true,
            jul: true,
            aug: true,
            sep: true,
            oct: true,
            nov: true,
            dec: true,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
struct DaysOffTheWeek {
    mon: bool,
    tue: bool,
    wed: bool,
    thu: bool,
    fri: bool,
    sat: bool,
    sun: bool,
}

impl DaysOffTheWeek {
    fn all() -> Self {
        DaysOffTheWeek {
            mon: true,
            tue: true,
            wed: true,
            thu: true,
            fri: true,
            sat: true,
            sun: true,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
struct TimeOfDay {
    hour: u32,
    minute: u32,
    second: u32,
}
struct RelayWithPin {
    relay: AnyOutputPin,
    condition: Condition,
    days_off_the_week: DaysOffTheWeek,
    operating_months: Month,
    exclude_times: Option<[TimeOfDay; 2]>,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum RelayNumber {
    Relay1,
    Relay2,
    Relay3,
    Relay4,
    StatusLed,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]

struct LightAmount {
    greater_or_less: bool,
    value: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
///with time it just means that it is on between those times
///
/// with light amount it just means that it is on when the light amount > or <
/// that depending on what is set in the only struct where this is supposed to be used
///
/// light amount limited is exaclty the same but only limited to those times
enum Condition {
    Time(Option<[TimeOfDay; 2]>),
    LightAmount(LightAmount),
    LightAmountTimeLimited(u32, Option<[TimeOfDay; 2]>),
}

pub async fn relay_controller_func(
    nvs: esp_idf_svc::nvs::EspNvs<nvs::NvsDefault>,
    pins: esp_idf_hal::gpio::Pins,
    reciever: smol::channel::Receiver<Relay>,
) {
    let mut relays = Relays::init(
        pins.gpio21.into(),
        pins.gpio19.into(),
        pins.gpio18.into(),
        pins.gpio2.into(),
        pins.gpio25.into(),
        &nvs,
    );
    relays.update(reciever).await;
}
