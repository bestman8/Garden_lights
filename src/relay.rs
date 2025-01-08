use time::Time;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Relays {
    relay_1: Relay,
    relay_2: Relay,
    relay_3: Relay,
    relay_4: Relay,
    status_led: Relay,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Relay {
    condition: Condition,
    days_off_the_week: [time::Weekday; 7],
    operating_months: [time::Month; 12],
    exclude_times: [Time; 2],
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
