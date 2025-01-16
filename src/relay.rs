use std::rc::Rc;
use std::time::Duration;

use crate::async_wifi_task;
use crate::sensor;
use chrono::Datelike;
use chrono::Timelike;
use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::sys::esp_alloc_failed_hook_t;
use esp_idf_hal::task::block_on;
use esp_idf_svc::nvs;
use log::error;
use log::{info, log, warn};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
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

    // async fn update(mut self, reciever: &std::sync::mpsc::Receiver<Relay>) {
    //     use RelayNumber::*;
    //     loop {
    //         info!("from update");
    //         if let Ok(obtained_relay) = reciever.recv() {
    //             println!("from update receved {:?}", obtained_relay);
    //             match obtained_relay.number {
    //                 Relay1 => self.relay_1 = obtained_relay,
    //                 Relay2 => self.relay_2 = obtained_relay,
    //                 Relay3 => self.relay_3 = obtained_relay,
    //                 Relay4 => self.relay_4 = obtained_relay,
    //                 StatusLed => self.status_led = obtained_relay,
    //                 _ => {
    //                     warn!("nothing recieved yet idk how this works");
    //                     std::thread::sleep(Duration::from_secs(10));
    //                 }
    //             }
    //         }
    //     }
    // }

    fn run<'a>(self, receiver: smol::channel::Receiver<Relay>, nvs: &'a esp_idf_svc::nvs::EspNvs<nvs::NvsDefault>) {
        let shared_data = Arc::new(Mutex::new(self));
        println!("inside the run fn");
        let shared_data_relay_1 = Arc::new(Mutex::new(self.relay_1));
        let shared_data_relay_2 = Arc::new(Mutex::new(self.relay_2));
        let shared_data_relay_3 = Arc::new(Mutex::new(self.relay_3));
        let shared_data_relay_4 = Arc::new(Mutex::new(self.relay_4));
        let status_led_relay___ = Arc::new(Mutex::new(self.status_led));

        let read_only = shared_data.clone();
        let writer = shared_data.clone();

        let shared_data_relay_1_clone = shared_data_relay_1.clone();
        let shared_data_relay_2_clone = shared_data_relay_2.clone();
        let shared_data_relay_3_clone = shared_data_relay_3.clone();
        let shared_data_relay_4_clone = shared_data_relay_4.clone();
        let status_led_relay____clone = status_led_relay___.clone();

        std::thread::spawn(move || {
            // let shared_data_relay_1_clone = shared_data_relay_1_clone.clone();
            Relay::do_stuff(shared_data_relay_1_clone, RelayNumber::Relay1)
        });
        std::thread::spawn(move || {
            Relay::do_stuff(shared_data_relay_2_clone, RelayNumber::Relay2);
        });

        std::thread::spawn(move || {
            Relay::do_stuff(shared_data_relay_3_clone, RelayNumber::Relay3);
        });

        std::thread::spawn(move || {
            Relay::do_stuff(shared_data_relay_4_clone, RelayNumber::Relay4);
        });

        std::thread::spawn(move || {
            Relay::do_stuff(status_led_relay____clone, RelayNumber::StatusLed);
        });

        // Spawn a thread for reading and printing the state
        std::thread::spawn(move || {
            let read_only = read_only.lock().unwrap();
            println!("in the read function before looper");
            loop {
                println!("{:?}", *read_only);
                let time_zone = chrono_tz::Europe::Amsterdam;
                let time_now = chrono::Utc::now().with_timezone(&time_zone);
                println!("{}", time_now.format("%Y-%m-%d %H:%M:%S").to_string());
                std::thread::sleep(Duration::from_secs(40));
            }
        });

        // Spawn a thread for updating the relays
        std::thread::spawn(move || {
            let mut writer = writer.lock().unwrap();
            block_on(async {
                println!("waiting for relay updates");
                loop {
                    if let Ok(obtained_relay) = receiver.recv().await {
                        println!("from update received {:?}", obtained_relay);
                        match obtained_relay.number {
                            RelayNumber::Relay1 => writer.relay_1 = obtained_relay,
                            RelayNumber::Relay2 => writer.relay_2 = obtained_relay,
                            RelayNumber::Relay3 => writer.relay_3 = obtained_relay,
                            RelayNumber::Relay4 => writer.relay_4 = obtained_relay,
                            RelayNumber::StatusLed => writer.status_led = obtained_relay,
                        }
                    }
                }
            });
        });

        // Keep the main thread alive
        esp_idf_hal::task::block_on(smol::Timer::never());
    }

    // *data = relay_s;
    // std::

    // // let arc_mutex: std::sync::Arc<std::sync::Mutex<Relays>> = std::sync::Arc::new(std::sync::Mutex::new(self));
    // std::thread::spawn(|| sensor::setup_sensor(1));

    // let shared_data = std::sync::Arc::new(std::sync::RwLock::new(*self));
    // macro_rules! create_threads {
    //     ($($relay:ident)+) => {
    //         $(
    //             let rw_reader_clone_relay = shared_data.clone();
    //             std::thread::spawn(move || {
    //                 let mut pin_num:i32 = 0;
    //                 println!("currently at pin definition");
    //                 loop{
    //                     println!("at pindriver");
    //                     let pin: AnyOutputPin = unsafe { AnyOutputPin::new(pin_num) };
    //                     let mut pindriver: PinDriver<'_, AnyOutputPin, esp_idf_hal::gpio::Output> = PinDriver::output(pin).unwrap();
    //                     loop{
    //                          println!("inner_loop");
    //                         let data: Relay = rw_reader_clone_relay.read().unwrap().$relay;
    //                         if pin_num != data.get_pin_i32(){
    //                             pin_num = data.get_pin_i32();
    //                             break
    //                         }
    //                         std::thread::scope(|s|{
    //                             s.spawn(||{

    //                                 loop{
    //                                     if Relays::do_stuff(data){
    //                                        let _=  pindriver.set_high();
    //                                     }else{
    //                                        let _=  pindriver.set_low();
    //                                     }
    //                                 }
    //                             }
    //                         );
    //                     }
    //                 );

    //                     }
    //                 }
    //             });
    //         )*
    //     };
    // }

    // create_threads!(relay_1 relay_2 relay_3 relay_4 status_led);
    // // create_threads!(status_led);

    // std::thread::sleep(core::time::Duration::from_secs(10));
    // let rw_writer_clone = shared_data.clone();
    // std::thread::spawn(move || {
    //     use RelayNumber::*;
    //     // let mut current_state = *self;

    //     loop {
    //         let mut data = rw_writer_clone.write().unwrap();
    //         // esp_idf_hal::task::block_on(Relays::update(&mut *data, &reciever));
    //         let mut relay_s = *data;
    //         // let test = relay_1.relay_1;
    //         info!("from update");
    //     }
    // });

    // loop {
    //     info!("BEFORE NEVER");
    //     smol::Timer::never().await;
    //     log::error!("AFTER NEVER");
    // }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
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

    fn do_stuff(relay: std::sync::Arc<std::sync::Mutex<Relay>>, relay_number: RelayNumber) {
        let relay_number: i32 = relay_number.get_pin_i32();

        info!("inside do stuff");
        let pin: AnyOutputPin = unsafe { AnyOutputPin::new(relay_number) };
        let mut pindriver: PinDriver<'_, AnyOutputPin, esp_idf_hal::gpio::Output> = PinDriver::output(pin).unwrap();
        loop {
            let binding = relay.clone();
            let relay_clone = binding.lock().unwrap();

            std::thread::sleep(Duration::from_secs(20));
            let time_zone = chrono_tz::Europe::Amsterdam;
            let time_now = chrono::Utc::now().with_timezone(&time_zone);
            let current_time = TimeOfDay {
                hour: time_now.hour().try_into().unwrap(),
                minute: time_now.minute().try_into().unwrap(),
                second: time_now.second().try_into().unwrap(),
            };
            let current_month = time_now.month();
            if !relay_clone.operating_months.is_current_month(current_month) {
                pindriver.set_low();
                continue;
            }

            let day = time_now.day();
            if !relay_clone.days_off_the_week.is_current_day(day) {
                pindriver.set_low();
                continue;
            }
            if let Some(exclude_times) = relay_clone.exclude_times {
                if !exclude_times.on_or_off(current_time) {
                    pindriver.set_low();
                    continue;
                }
            }

            if relay_clone.condition.on_or_off(current_time) {
                pindriver.set_high();
            } else {
                pindriver.set_low();
            }
        }
        todo!()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
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
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
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
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
enum RelayNumber {
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
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
struct LightAmount {
    greater_or_less: bool,
    value: u32,
}
impl LightAmount {
    fn on_or_off(&self, current_time: TimeOfDay) -> bool {
        todo!()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq)]
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
    receiver: smol::channel::Receiver<Relay>,
) {
    let relays = Arc::new(Mutex::new(Relays::init(&nvs)));
    info!("Initialized relays");

    let mut relays_locked = relays.lock().unwrap();
    relays_locked.run(receiver, &nvs);
}
