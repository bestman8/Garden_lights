// use anyhow::Ok;
use chrono::{Datelike, Timelike};
use crossbeam::channel::Receiver;
use esp_idf_hal::gpio::{AnyOutputPin, PinDriver};
use esp_idf_svc::nvs;
use log::{error, info};
use std::result::Result::Ok;
use std::thread::{sleep, spawn};
use std::time::Duration;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Relays {
    relay_1: Relay,
    relay_2: Relay,
    relay_3: Relay,
    relay_4: Relay,
    status_led: Relay,
}

impl Relays {
    pub fn init(nvs: nvs::EspNvsPartition<nvs::NvsDefault>) -> Relays {
        Relays {
            relay_1: Relay::init(RelayNumber::Relay1, nvs.clone()),
            relay_2: Relay::init(RelayNumber::Relay2, nvs.clone()),
            relay_3: Relay::init(RelayNumber::Relay3, nvs.clone()),
            relay_4: Relay::init(RelayNumber::Relay4, nvs.clone()),
            status_led: Relay::init(RelayNumber::StatusLed, nvs.clone()),
        }
    }

    fn run(mut self, channels: Channels, nvs: nvs::EspNvsPartition<nvs::NvsDefault>) {
        println!("inside the run fn");

        let (relay_1_nvs, relay_2_nvs, relay_3_nvs, relay_4_nvs, status_nvs) = (nvs.clone(), nvs.clone(), nvs.clone(), nvs.clone(), nvs.clone());
        let (reciever_relay_1, reciever_relay_2, reciever_relay_3, reciever_relay_4, reciever_status_led) = channels;

        spawn(move || {
            // let shared_data_relay_1_clone = shared_data_relay_1_clone.clone();
            // Relay::do_stuff(self.relay_1, RelayNumber::Relay1, reciever_relay_1, relay_1_nvs);
            self.relay_1.do_stuff_expr(reciever_relay_1, relay_1_nvs);
        });
        spawn(move || {
            self.relay_2.do_stuff_expr(reciever_relay_2, relay_2_nvs);
            // Relay::do_stuff(self.relay_2, RelayNumber::Relay2, reciever_relay_2, relay_2_nvs);
        });

        spawn(move || {
            self.relay_3.do_stuff_expr(reciever_relay_3, relay_3_nvs);
        });

        spawn(move || {
            self.relay_4.do_stuff_expr(reciever_relay_4, relay_4_nvs);
        });

        spawn(move || {
            self.status_led.do_stuff_expr(reciever_status_led, status_nvs);
        });

        // Spawn a thread for reading and printing the state
        spawn(move || {
            // let read_only = read_only.lock().unwrap();
            println!("in the read function before looper");
            loop {
                // println!("{:?}", *read_only);
                let time_zone = chrono_tz::Europe::Amsterdam;
                let time_now = chrono::Utc::now().with_timezone(&time_zone);
                println!("{}", time_now.format("%Y-%m-%d %H:%M:%S"));
                sleep(Duration::from_secs(40));
            }
        });

        sleep(Duration::from_secs(u64::MAX));

        // esp_idf_hal::task::block_on(smol::Timer::never());
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Relay {
    pub number: RelayNumber,
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
            number,
            days_off_the_week: DaysOffTheWeek::all(),
            operating_months: Month::all(),
            exclude_times: None,
        }
    }

    fn init(number: RelayNumber, nvs: nvs::EspNvsPartition<nvs::NvsDefault>) -> Relay {
        let name = number.get_name();
        let mut buf = [0; 128];

        nvs::EspNvs::new(nvs, number.get_name(), true)
            .and_then(|nvs| nvs.get_raw(name, &mut buf))
            .unwrap_or_default()
            .map(postcard::from_bytes::<Relay>)
            .unwrap_or_else(|| Ok(Relay::new(number.clone())))
            .unwrap_or_else(|_| Relay::new(number.clone()))
    }

    fn _get_pin_i32(&self) -> i32 {
        match self.number {
            RelayNumber::Relay1 => 21,
            RelayNumber::Relay2 => 19,
            RelayNumber::Relay3 => 18,
            RelayNumber::Relay4 => 5,
            RelayNumber::StatusLed => 25,
        }
    }

    fn do_stuff_expr(&mut self, reciever_relay: Receiver<Relay>, nvs: nvs::EspNvsPartition<nvs::NvsDefault>) {
        let relay_number = self.number.clone();
        let mut nsv_ds: nvs::EspNvs<nvs::NvsDefault> = nvs::EspNvs::new(nvs, relay_number.get_name(), true).unwrap();

        self.get_from_storage(&relay_number, &nsv_ds);

        info!("inside do stuff");
        let mut pindriver = PinDriver::output(unsafe { AnyOutputPin::new(relay_number.get_pin_i32()) }).unwrap();
        loop {
            std::thread::sleep(Duration::from_secs(20));
            let _ = reciever_relay.try_recv().inspect(|new_relay| {
                self.set_and_save_relay(&mut nsv_ds, new_relay);
            });

            self.on_or_off(&mut pindriver);
        }
    }

    fn on_or_off(&mut self, pindriver: &mut PinDriver<'_, AnyOutputPin, esp_idf_hal::gpio::Output>) {
        let (current_time, current_month, current_day) = TimeOfDay::now();
        // let current_month = time_now.month();
        if !self.operating_months.is_current_month(current_month)
            || !self.days_off_the_week.is_current_day(current_day)
            || self.exclude_times.as_ref().is_some_and(|times| !times.on_or_off(&current_time))
        {
            let _ = pindriver.set_low();
        } else {
            let _ = self
                .condition
                .on_or_off(&current_time)
                .then(|| pindriver.set_high())
                .or_else(|| Some(pindriver.set_low()));
        }
    }

    fn set_and_save_relay(&mut self, nsv_ds: &mut nvs::EspNvs<nvs::NvsDefault>, new_relay: &Relay) {
        info!("RECIEVED THE STRUCT {:?}", &new_relay);
        let _ = nsv_ds
            .set_raw(new_relay.number.get_name(), &postcard::to_vec::<Relay, 128>(new_relay).unwrap())
            .inspect_err(|&err| {
                error!("failed_storing_the_struct {} ", err);
            });
        *self = new_relay.clone();
    }

    fn get_from_storage(&mut self, relay_number: &RelayNumber, nsv_ds: &nvs::EspNvs<nvs::NvsDefault>) {
        let key_raw_struct_data: &mut [u8] = &mut [0; 128];

        if let Ok(thing) = nsv_ds.get_raw(relay_number.get_name(), key_raw_struct_data) {
            *self = postcard::from_bytes::<Relay>(thing.unwrap_or(&[0, 0])).unwrap_or(self.clone());
        };
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Month {
    months: u16,
}
impl Month {
    pub fn all() -> Self {
        Month { months: 0b00001111_11111111 }
    }

    pub fn is_current_month(&self, other_month: u32) -> bool {
        let other_month = Month::months_to_mask(other_month);
        (other_month & self.months) != 0
    }

    ///where january is 1
    pub fn months_to_mask(month: u32) -> u16 {
        let month: u16 = month.try_into().unwrap_or(0);
        match month {
            1 => 0b00000000_00000001,  //jan
            2 => 0b00000000_00000010,  //feb
            3 => 0b00000000_00000100,  //mar
            4 => 0b00000000_00001000,  //apr
            5 => 0b00000000_00010000,  //may
            6 => 0b00000000_00100000,  //jun
            7 => 0b00000000_01000000,  //jul
            8 => 0b00000000_10000000,  //aug
            9 => 0b00000001_00000000,  //sept
            10 => 0b00000010_00000000, //okt
            11 => 0b00000100_00000000, //nov
            12 => 0b00001000_00000000, //dec
            // 0 => 0b01000000, //sunday
            _ => {
                println!("day: {} doesn't exist", month);
                0b00000000
            } //unknown day
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct DaysOffTheWeek {
    pub days: u8,
}
impl DaysOffTheWeek {
    pub fn all() -> Self {
        DaysOffTheWeek { days: u8::MAX }
    }

    pub fn _days_to_struct(days: Vec<u8>) -> DaysOffTheWeek {
        let mut mask = 0;
        for day in days {
            mask |= DaysOffTheWeek::day_to_mask(day.into());
        }
        DaysOffTheWeek { days: mask }
    }

    pub fn day_to_mask(day: u32) -> u8 {
        let day: u8 = day.try_into().unwrap_or(0);
        match day {
            1 => 0b00000001, //monday
            2 => 0b00000010, //tuesday
            3 => 0b00000100, //wednesday
            4 => 0b00001000, //thursday
            5 => 0b00010000, //friday
            6 => 0b00100000, //saturday
            0 => 0b01000000, //sunday
            _ => {
                println!("day: {} doesn't exist", day);
                0b00000000
            } //unknown day
        }
    }

    pub fn is_current_day(&self, other_day: u32) -> bool {
        // let other_day: u8 = other_day.try_into().unwrap_or(0);
        let other_day = DaysOffTheWeek::day_to_mask(other_day);
        (other_day & self.days) != 0
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
struct TimeOfDay {
    hour: u8,
    minute: u8,
    second: u8,
}
impl TimeOfDay {
    fn to_sec(&self) -> u32 {
        u32::from(self.hour) * 3600 + u32::from(self.minute) * 60 + u32::from(self.second)
    }

    fn now() -> (TimeOfDay, u32, u32) {
        let time_zone = chrono_tz::Europe::Amsterdam;
        let time_now = chrono::Utc::now().with_timezone(&time_zone);
        let current_time = TimeOfDay {
            hour: time_now.hour().try_into().unwrap(),
            minute: time_now.minute().try_into().unwrap(),
            second: time_now.second().try_into().unwrap(),
        };

        (current_time, time_now.month(), time_now.weekday().num_days_from_monday())
    }
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum RelayNumber {
    Relay1,
    Relay2,
    Relay3,
    Relay4,
    StatusLed,
}
impl RelayNumber {
    fn get_pin_i32(self) -> i32 {
        match self {
            RelayNumber::Relay1 => 21,
            RelayNumber::Relay2 => 19,
            RelayNumber::Relay3 => 18,
            RelayNumber::Relay4 => 5,
            RelayNumber::StatusLed => 25,
        }
    }

    fn get_name(&self) -> &str {
        match self {
            RelayNumber::Relay1 => "Relay1",
            RelayNumber::Relay2 => "Relay2",
            RelayNumber::Relay3 => "Relay3",
            RelayNumber::Relay4 => "Relay4",
            RelayNumber::StatusLed => "StatusLed",
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
struct LightAmount {
    greater_or_less: bool,
    value: u32,
}
impl LightAmount {
    fn on_or_off(&self, _current_time: &TimeOfDay) -> bool {
        todo!()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
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
    fn on_or_off(&self, current_time: &TimeOfDay) -> bool {
        // *started = false;
        match self {
            Condition::Time(option_times) => option_times
                .clone()
                .is_some_and(|times| times.is_none_or(|time| time.on_or_off(current_time))),
            Condition::LightAmount(light_amount) => light_amount.on_or_off(current_time),
            Condition::LightAmountTimeLimited(light_amount, times) => light_amount.on_or_off(current_time) && times.on_or_off(current_time),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
struct Times {
    start_time: TimeOfDay,
    end_time: TimeOfDay,
}
impl Times {
    fn on_or_off(&self, current_time: &TimeOfDay) -> bool {
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

pub fn relay_controller_func(nvs: nvs::EspNvsPartition<nvs::NvsDefault>, channels: Channels) {
    let relays = Relays::init(nvs.clone());
    info!("Initialized relays");

    // let mut relays_locked = relays.lock().unwrap();
    relays.run(channels, nvs.clone());
}

type Channels = (Receiver<Relay>, Receiver<Relay>, Receiver<Relay>, Receiver<Relay>, Receiver<Relay>);
