use chrono::{Datelike, Timelike};
use esp_idf_hal::gpio::{AnyOutputPin, PinDriver};
use esp_idf_svc::nvs;
use log::{error, info, log, warn};
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

    fn run(
        self,
        reciever_relay_1: crossbeam::channel::Receiver<Relay>,
        reciever_relay_2: crossbeam::channel::Receiver<Relay>,
        reciever_relay_3: crossbeam::channel::Receiver<Relay>,
        reciever_relay_4: crossbeam::channel::Receiver<Relay>,
        reciever_status_led: crossbeam::channel::Receiver<Relay>,
        nvs: nvs::EspNvsPartition<nvs::NvsDefault>,
    ) {
        println!("inside the run fn");

        let relay_1_nvs = nvs.clone();
        let relay_2_nvs = nvs.clone();
        let relay_3_nvs = nvs.clone();
        let relay_4_nvs = nvs.clone();
        let status__nvs = nvs.clone();

        spawn(move || {
            // let shared_data_relay_1_clone = shared_data_relay_1_clone.clone();
            Relay::do_stuff(self.relay_1, RelayNumber::Relay1, reciever_relay_1, relay_1_nvs)
        });
        spawn(move || {
            Relay::do_stuff(self.relay_2, RelayNumber::Relay2, reciever_relay_2, relay_2_nvs);
        });

        spawn(move || {
            Relay::do_stuff(self.relay_3, RelayNumber::Relay3, reciever_relay_3, relay_3_nvs);
        });

        spawn(move || {
            Relay::do_stuff(self.relay_4, RelayNumber::Relay4, reciever_relay_4, relay_4_nvs);
        });

        spawn(move || {
            Relay::do_stuff(self.status_led, RelayNumber::StatusLed, reciever_status_led, status__nvs);
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
            number: number,
            days_off_the_week: DaysOffTheWeek::all(),
            operating_months: Month::all(),
            exclude_times: None,
        }
    }

    fn init(number: RelayNumber, nvs: esp_idf_svc::nvs::EspNvsPartition<esp_idf_svc::nvs::NvsDefault>) -> Relay {
        let name = number.get_name();
        let mut nvs = match esp_idf_svc::nvs::EspNvs::new(nvs, number.get_name(), true) {
            Ok(nvs) => nvs,
            Err(error) => {
                error!("{}", error);
                return Relay::new(number);
            }
        };
        match nvs.get_raw(name, &mut [0; 128]) {
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

    fn do_stuff(
        relay: Relay,
        relay_number: RelayNumber,
        reciever_relay: crossbeam::channel::Receiver<Relay>,
        nvs: esp_idf_svc::nvs::EspNvsPartition<esp_idf_svc::nvs::NvsDefault>,
    ) {
        let mut relay = relay;
        // let nvs = nvs::EspDefaultNvsPartition::().unwrap();
        // let mut nvs_clone: nvs::EspNvsPartition<nvs::NvsDefault> = nvs.clone();
        let mut nsv_ds: nvs::EspNvs<nvs::NvsDefault> = nvs::EspNvs::new(nvs, relay.number.get_name(), true).unwrap();

        // if nsv_ds

        let key_raw_struct_data: &mut [u8] = &mut [0; 128];

        match nsv_ds.get_raw(relay.number.get_name(), key_raw_struct_data) {
            Ok(v) => {
                if let Some(the_struct) = v {
                    info!("{:?} = {:?}", relay.number.get_name(), postcard::from_bytes::<Relay>(the_struct));
                    if let Ok(stored_relay) = postcard::from_bytes::<Relay>(the_struct) {
                        relay = stored_relay;
                    }
                }
            }
            Err(e) => info!("Couldn't get key {} because {:?}", relay.number.get_name(), e),
        };

        info!("inside do stuff");
        let relay_number: i32 = relay_number.get_pin_i32();
        let pin: AnyOutputPin = unsafe { AnyOutputPin::new(relay_number) };
        let mut pindriver: PinDriver<'_, AnyOutputPin, esp_idf_hal::gpio::Output> = PinDriver::output(pin).unwrap();
        loop {
            if let Ok(new_relay_data) = reciever_relay.try_recv() {
                let clone_relay = new_relay_data.clone();
                relay = new_relay_data;

                info!("RECIEVED THE STRUCT {:?}", clone_relay);
                if nsv_ds
                    .set_raw(
                        clone_relay.clone().number.get_name(),
                        &postcard::to_vec::<Relay, 128>(&clone_relay).unwrap(),
                    )
                    .is_ok()
                {
                    info!("succesfully stored the struct")
                } else {
                    warn!("failed storing the struct")
                };
            } else {
                info!("nothing to recieve yet")
            }

            std::thread::sleep(Duration::from_secs(20));
            let time_zone = chrono_tz::Europe::Amsterdam;
            let time_now = chrono::Utc::now().with_timezone(&time_zone);
            let current_time = TimeOfDay {
                hour: time_now.hour().try_into().unwrap(),
                minute: time_now.minute().try_into().unwrap(),
                second: time_now.second().try_into().unwrap(),
            };
            info!("debug data: {:?}", relay);
            let current_month = time_now.month();
            if !relay.operating_months.is_current_month(current_month) {
                let _ = pindriver.set_low();
                continue;
            }
            info!("passes month");

            let day = time_now.weekday().num_days_from_monday();
            info!("the current day in u32 : {}", day);
            info!("days_off_the_week.is_current_day(day) : {}", relay.days_off_the_week.is_current_day(day));
            if !relay.days_off_the_week.is_current_day(day) {
                info!("pin set low");
                let _ = pindriver.set_low();
                continue;
            }
            info!("passes day");

            if let Some(exclude_times) = &relay.exclude_times {
                if !exclude_times.on_or_off(current_time) {
                    let _ = pindriver.set_low();
                    continue;
                }
            }
            info!("passes exclude");

            if relay.condition.on_or_off(current_time) {
                info!("pin high");
                let _ = pindriver.set_high();
            } else {
                info!("pin low");
                let _ = pindriver.set_low();
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
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
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
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
            0 => self.sun,
            1 => self.mon,
            2 => self.tue,
            3 => self.wed,
            4 => self.thu,
            5 => self.fri,
            6 => self.sat,
            _ => false,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
struct TimeOfDay {
    hour: u8,
    minute: u8,
    second: u8,
}
impl TimeOfDay {
    fn to_sec(&self) -> u32 {
        u32::from(self.hour) * 3600 + u32::from(self.minute) * 60 + u32::from(self.second)
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
    fn on_or_off(&self, current_time: TimeOfDay) -> bool {
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
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
    nvs: esp_idf_svc::nvs::EspNvsPartition<esp_idf_svc::nvs::NvsDefault>,
    reciever_relay_1: crossbeam::channel::Receiver<Relay>,
    reciever_relay_2: crossbeam::channel::Receiver<Relay>,
    reciever_relay_3: crossbeam::channel::Receiver<Relay>,
    reciever_relay_4: crossbeam::channel::Receiver<Relay>,
    reciever_status_led: crossbeam::channel::Receiver<Relay>,
) {
    let relays = Relays::init(nvs.clone());
    info!("Initialized relays");

    // let mut relays_locked = relays.lock().unwrap();
    relays.run(
        reciever_relay_1,
        reciever_relay_2,
        reciever_relay_3,
        reciever_relay_4,
        reciever_status_led,
        nvs.clone(),
    );
}
