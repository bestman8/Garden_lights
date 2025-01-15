use crate::sensor;
use chrono::Datelike;
use chrono::Timelike;
use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::task::block_on;
use esp_idf_svc::nvs;
use log::error;
use log::{info, log, warn};

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct Relays {
    relay_1: Relay,
    relay_2: Relay,
    relay_3: Relay,
    relay_4: Relay,
    status_led: Relay,
}

impl Relays {
    pub fn init(nvs: &esp_idf_svc::nvs::EspNvs<nvs::NvsDefault>) -> Relays {
        Relays {
            relay_1: Relay::init(RelayNumber::Relay1, nvs),
            relay_2: Relay::init(RelayNumber::Relay2, nvs),
            relay_3: Relay::init(RelayNumber::Relay3, nvs),
            relay_4: Relay::init(RelayNumber::Relay4, nvs),
            status_led: Relay::init(RelayNumber::StatusLed, nvs),
        }
    }

    async fn update(&mut self, reciever: &smol::channel::Receiver<Relay>) {
        use RelayNumber::*;
        loop {
            info!("from update");
            if let Ok(obtained_relay) = reciever.recv().await {
                match obtained_relay.number {
                    Relay1 => self.relay_1 = obtained_relay,
                    Relay2 => self.relay_2 = obtained_relay,
                    Relay3 => self.relay_3 = obtained_relay,
                    Relay4 => self.relay_4 = obtained_relay,
                    StatusLed => self.status_led = obtained_relay,
                    _ => {
                        warn!("this shouldn't exist")
                    }
                }
            }
        }
    }

    fn do_stuff(data: &Relay) -> bool {
        info!("inside do stuff");
        use core::time::Duration;
        use std::thread::sleep;
        // todo!();
        // fn return_pin_for_relay
        let start_data = *data; //we clone here on purpose because we're compareing it later

        let pin: AnyOutputPin = unsafe { AnyOutputPin::new(data.get_pin_i32()) };
        let mut pindriver = PinDriver::output(pin).unwrap();
        let mut started = false;
        loop {
            if start_data != *data {
                info!("data changed");
                return true;
            }
            sleep(Duration::from_secs(10)); //longer makes way more sense for the long term

            let time_zone = chrono_tz::Europe::Amsterdam;
            let time_now = chrono::Utc::now().with_timezone(&time_zone);
            let current_time = TimeOfDay {
                hour: time_now.hour().try_into().unwrap(),
                minute: time_now.minute().try_into().unwrap(),
                second: time_now.second().try_into().unwrap(),
            };
            let current_month = time_now.month();
            if !data.operating_months.is_current_month(current_month) {
                continue;
            }

            let day = time_now.day();
            if !data.days_off_the_week.is_current_day(day) {
                continue;
            }
            if let Some(exclude_times) = data.exclude_times {
                if !exclude_times.on_or_off(current_time) {
                    continue;
                }
            }

            match data.condition.on_or_off(current_time) {
                true => {
                    let _ = pindriver.set_high();
                }
                false => {
                    let _ = pindriver.set_low();
                }
            }
        }
    }

    async fn run(&self, reciever: smol::channel::Receiver<Relay>, nvs: &esp_idf_svc::nvs::EspNvs<nvs::NvsDefault>) {
        println!("inside the run fn");
        println!("inside the run fn");

        // std::

        // let arc_mutex: std::sync::Arc<std::sync::Mutex<Relays>> = std::sync::Arc::new(std::sync::Mutex::new(self));
        std::thread::spawn(|| sensor::setup_sensor(1));

        let shared_data = std::sync::Arc::new(std::sync::RwLock::new(*self));
        macro_rules! create_threads {
            ($($relay:ident)+) => {
                $(
                    let rw_reader_clone_relay = shared_data.clone();
                    std::thread::spawn(move || {
                        loop{
                            let data = rw_reader_clone_relay.read().unwrap().$relay;

                            let _ = Relays::do_stuff(&data);
                        }
                    });
                )*
            };
        }
        create_threads!(relay_1 relay_2 relay_3 relay_4 status_led);
        // create_threads!(status_led);

        std::thread::sleep(core::time::Duration::from_secs(10));
        let rw_writer_clone = shared_data.clone();
        std::thread::spawn(move || {
            let mut data = rw_writer_clone.write().unwrap();
            esp_idf_hal::task::block_on(Relays::update(&mut *data, &reciever));
        });

        loop {
            info!("BEFORE NEVER");
            smol::Timer::never().await;
            log::error!("AFTER NEVER");
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq)]
pub struct Relay {
    number: RelayNumber,
    condition: Condition,
    days_off_the_week: DaysOffTheWeek,
    operating_months: Month,
    exclude_times: Option<Times>,
}
impl Relay {
    fn new(number: RelayNumber) -> Relay {
        // use time::{Month, Weekday};?

        info!("relay created");
        Relay {
            condition: Condition::Time(None),
            number: number,
            days_off_the_week: DaysOffTheWeek::all(),
            operating_months: Month::all(),
            exclude_times: None,
        }
    }

    fn init(number: RelayNumber, nvs: &esp_idf_svc::nvs::EspNvs<nvs::NvsDefault>) -> Relay {
        let name = match number {
            RelayNumber::Relay1 => "Relay1",
            RelayNumber::Relay2 => "Relay2",
            RelayNumber::Relay3 => "Relay3",
            RelayNumber::Relay4 => "Relay4",
            RelayNumber::StatusLed => "StatusLed",
        };
        match nvs.get_raw(name, &mut [0; 512]) {
            Ok(bytes) => {
                match postcard::from_bytes(bytes.unwrap_or(&[0, 0])) {
                    Ok(relay) => {
                        return relay;
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
        // return Relay::new(number);
    }

    fn get_pin_i32(&self) -> i32 {
        match self.number {
            RelayNumber::Relay1 => 21,
            RelayNumber::Relay2 => 19,
            RelayNumber::Relay3 => 18,
            RelayNumber::Relay4 => 5,
            RelayNumber::StatusLed => 25,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq)]
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

    fn is_current_month(&self, current_month: u32) -> bool {
        match current_month {
            1 => self.jan,
            2 => self.feb,
            3 => self.mar,
            4 => self.apr,
            5 => self.may,
            6 => self.jun,
            7 => self.jul,
            8 => self.aug,
            9 => self.sep,
            10 => self.oct,
            11 => self.nov,
            12 => self.dec,
            _ => false,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq)]
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

    fn is_current_day(&self, other_day: u32) -> bool {
        match other_day {
            1 => self.mon,
            2 => self.tue,
            3 => self.wed,
            4 => self.thu,
            5 => self.fri,
            6 => self.sat,
            7 => self.sun,
            _ => false,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq)]
struct TimeOfDay {
    hour: u8,
    minute: u8,
    second: u8,
}
impl TimeOfDay {
    fn correct_difference(&self, time1: TimeOfDay) -> u32 {
        let secs_self = self.to_sec();
        let mut sec_time_other = time1.to_sec();
        if secs_self > sec_time_other {
            sec_time_other += 86400;
        }
        sec_time_other - secs_self
    }

    fn to_sec(&self) -> u32 {
        u32::from(self.hour) * 3600 + u32::from(self.minute) * 60 + u32::from(self.second)
    }

    fn from_sec(secs: u32) -> TimeOfDay {
        TimeOfDay {
            hour: (secs / 3600).try_into().unwrap(),
            minute: ((secs % 3600) / 60).try_into().unwrap(),
            second: (secs % 60).try_into().unwrap(),
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq)]
enum RelayNumber {
    Relay1,
    Relay2,
    Relay3,
    Relay4,
    StatusLed,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq)]
struct LightAmount {
    greater_or_less: bool,
    value: u32,
}
impl LightAmount {
    fn on_or_off(&self, current_time: TimeOfDay) -> bool {
        todo!()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq)]
///with time it just means that it is on between those times
///
/// with light amount it just means that it is on when the light amount > or <
/// that depending on what is set in the only struct where this is supposed to be used
///
/// light amount limited is exaclty the same but only limited to those times
enum Condition {
    Time(Option<Option<Times>>),
    LightAmount(LightAmount),
    LightAmountTimeLimited(LightAmount, Times),
}
impl Condition {
    fn on_or_off(&self, current_time: TimeOfDay) -> bool {
        // *started = false;
        match self {
            Condition::Time(option_times) => {
                if let Some(times) = option_times {
                    if let Some(on_or_off) = times {
                        on_or_off.on_or_off(current_time)
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            Condition::LightAmount(light_amount) => light_amount.on_or_off(current_time),
            Condition::LightAmountTimeLimited(light_amount, times) => light_amount.on_or_off(current_time) && times.on_or_off(current_time),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq)]
struct Times {
    start_time: TimeOfDay,
    end_time: TimeOfDay,
}
impl Times {
    fn on_or_off(&self, current_time: TimeOfDay) -> bool {
        let mut secs_current_time = current_time.to_sec();
        let secs_time_start = self.start_time.to_sec();
        let mut secs_end_time = self.end_time.to_sec();
        if secs_time_start > secs_end_time && secs_current_time < secs_end_time {
            secs_current_time += 86400;
        }
        if secs_time_start > secs_end_time {
            secs_end_time += 86400
        }
        secs_time_start < secs_current_time && secs_current_time < secs_end_time
    }
}

pub fn relay_controller_func(
    nvs: esp_idf_svc::nvs::EspNvs<nvs::NvsDefault>,
    pins: esp_idf_hal::gpio::Pins,
    reciever: smol::channel::Receiver<Relay>,
) {
    let mut relays = Relays::init(&nvs);
    info!("left the Relays::init");

    block_on(relays.run(reciever, &nvs));
    // relays.update(reciever).await;
}
