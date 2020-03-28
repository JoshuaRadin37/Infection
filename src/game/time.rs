use chrono::{Date, Local};

pub struct Time {
    start_time: Date<Local>,
    hours_since: usize
}

