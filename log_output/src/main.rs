use chrono::{SecondsFormat, Utc};
use std::{thread, time::Duration};
use uuid::Uuid;

fn main() {
    // Random string generated once at startup and kept in memory
    let random_string = Uuid::new_v4().to_string();

    loop {
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        println!("{timestamp}: {random_string}");
        thread::sleep(Duration::from_secs(5));
    }
}
