use log::{error, log, warn};
use time::Time;

use crate::relay;
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Relays {
    relay_1: Relay,
    relay_2: Relay,
    relay_3: Relay,
    relay_4: Relay,
    status_led: Relay,
}

impl Relays {
    async fn update(&mut self, reciever: smol::channel::Receiver<Relay>) {
        use RelayNumber::*;
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

    fn new() -> Relays {
        todo!()
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Relay {
    number: RelayNumber,
    condition: Condition,
    days_off_the_week: [time::Weekday; 7],
    operating_months: [time::Month; 12],
    exclude_times: [Time; 2],
}
// impl Relay {
//     async fn should_be_on_or_off() -> bool {
//         true
//     }
// }

#[derive(serde::Serialize, serde::Deserialize)]
enum RelayNumber {
    Relay1,
    Relay2,
    Relay3,
    Relay4,
    StatusLed,
}

#[derive(serde::Serialize, serde::Deserialize)]

struct LightAmount {
    greater_or_less: bool,
    value: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum Condition {
    // Weather,
    Time([Time; 2]),
    LightAmount(LightAmount),
    LightAmountTimeLimited(u32, Time, Time),
}

pub async fn relay_controller_func(pins: esp_idf_hal::gpio::Pins, reciever: smol::channel::Receiver<Relay>) {
    let mut relays = Relays::new();
    relays.update(reciever).await;
}
