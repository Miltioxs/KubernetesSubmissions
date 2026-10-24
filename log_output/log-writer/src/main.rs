use chrono::{SecondsFormat, Utc};
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    thread,
    time::Duration,
};
use uuid::Uuid;

fn main() {
    // Read the file path from the LOG_FILE env var, with a default for the container
    let log_file = env::var("LOG_FILE").unwrap_or_else(|_| "/usr/src/app/files/log.txt".to_string());

    // Make sure the parent directory exists before writing
    if let Some(dir) = Path::new(&log_file).parent() {
        fs::create_dir_all(dir).expect("failed to create log directory");
    }

    // Random string generated once at startup and kept in memory
    let random_string = Uuid::new_v4().to_string();

    loop {
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let line = format!("{timestamp}: {random_string}");

        // Append the line to the shared file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .expect("failed to open log file");
        writeln!(file, "{line}").expect("failed to write log file");

        // Also print it so it shows up in kubectl logs
        println!("{line}");

        thread::sleep(Duration::from_secs(5));
    }
}
