use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::task::block_on;
use esp_idf_svc::nvs;
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

    fn do_stuff(data: Relay) {
        info!("inside do stuff");
        // todo!();
        // fn return_pin_for_relay

        let pin: AnyOutputPin = unsafe { AnyOutputPin::new(data.get_pin_i32()) };
        let mut pindriver = PinDriver::output(pin).unwrap();

        info!("after pin driver creation");
        loop {
            pindriver.set_high().unwrap();
            std::thread::sleep(core::time::Duration::from_secs(5));
            pindriver.set_low().unwrap();
            std::thread::sleep(core::time::Duration::from_secs(5));
        }
    }

    async fn run(mut self, reciever: smol::channel::Receiver<Relay>, nvs: &esp_idf_svc::nvs::EspNvs<nvs::NvsDefault>) {
        println!("inside the run fn");
        println!("inside the run fn");

        // std::

        // let arc_mutex: std::sync::Arc<std::sync::Mutex<Relays>> = std::sync::Arc::new(std::sync::Mutex::new(self));

        let rw_mutex = std::sync::Arc::new(std::sync::RwLock::new(self));
        macro_rules! create_threads {
            ($($relay:ident)+) => {
                $(
                    let rw_reader_clone_relay = rw_mutex.clone();
                    std::thread::spawn(move || {
                        let data = rw_reader_clone_relay.read().unwrap().$relay;
                        // info!("right before do_stuff gets called");
                        Relays::do_stuff(data);
                    });
                )*
            };
        }
        create_threads!(relay_1 relay_2 relay_3 relay_4 status_led);

        std::thread::sleep(core::time::Duration::from_secs(10));
        let rw_writer_clone = rw_mutex.clone();
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
impl RelayWithPin {
    fn custom_from(&mut self, relay: Relay) {
        self.condition = relay.condition;
        self.days_off_the_week = relay.days_off_the_week;
        self.operating_months = relay.operating_months;
        self.exclude_times = relay.exclude_times;
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
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

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
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
