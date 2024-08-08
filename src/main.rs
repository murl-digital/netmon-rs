use std::{str::FromStr, time::Duration};

use cfspeedtest::
    speedtest::{self, TestType}
;
use chrono::{DateTime, Utc};
use color_eyre::Result;
use ping_rs::send_ping;
use rusqlite::{params, Connection};

enum Report {
    PingFailed,
    PingSucceeded(Measurement),
}

impl Report {
    fn insert(self, now: DateTime<Utc>, connection: &Connection) -> Result<usize, rusqlite::Error> {
        const QUERY: &str =
            "INSERT INTO reports (time, ping_succeeded, metadata, avg_latency, avg_down, avg_up) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
        match self {
            Self::PingFailed => connection.execute(QUERY, params![now, false, "N/A", 0.0, 0.0, 0.0]),
            Self::PingSucceeded(report) => {
                connection.execute(QUERY, params![now, true, report.metadata, report.avg_latency, report.avg_down, report.avg_up])
            }
        }
    }
}

struct Measurement {
    metadata: String,
    avg_latency: f64,
    avg_down: f64,
    avg_up: f64,
}

impl From<(String, f64, Vec<cfspeedtest::measurements::Measurement>)> for Measurement {
    fn from(value: (String, f64, Vec<cfspeedtest::measurements::Measurement>)) -> Self {
        let (metadata, avg_latency, measurements) = value;
        let num_download = measurements
            .iter()
            .filter(|m| m.test_type == TestType::Download)
            .count();
        let num_upload = measurements
            .iter()
            .filter(|m| m.test_type == TestType::Upload)
            .count();

        Self {
            metadata,
            avg_latency,
            avg_down: measurements
                .iter()
                .filter(|m| m.test_type == TestType::Download)
                .map(|m| m.mbit)
                .sum::<f64>()
                / num_download as f64,
            avg_up: measurements
                .iter()
                .filter(|m| m.test_type == TestType::Upload)
                .map(|m| m.mbit)
                .sum::<f64>()
                / num_upload as f64,
        }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    println!("setting up database...");
    let connection = Connection::open("./reports.sqlite3")?;
    connection.execute(
        "CREATE TABLE IF NOT EXISTS reports(
        time DATETIME,
        ping_succeeded BOOL,
        metadata TEXT,
        avg_latency REAL,
        avg_down REAL,
        avg_up REAL
    )",
        (),
    )?;
    println!("done! pinging 1.1.1.1...");

    let now = chrono::Utc::now();

    let ping = send_ping(
        &std::net::IpAddr::from_str("1.1.1.1").expect("this is a valid ip you shit"),
        Duration::from_secs(5),
        b"rawr xd uwu",
        None,
    );

    let report = match ping {
        Ok(_) => {
            println!("ping success! speedtest time.");
            let client = reqwest::blocking::Client::new();

            let metadata = speedtest::fetch_metadata(&client);
            let (latencies, avg_latency) =
                speedtest::run_latency_test(&client, 16, cfspeedtest::OutputFormat::StdOut);

            println!("all reported latencies: {latencies:?}");

            let mut measurements = speedtest::run_tests(
                &client,
                speedtest::test_download,
                TestType::Download,
                vec![1_000_000],
                16,
                cfspeedtest::OutputFormat::StdOut,
                false,
            );
            measurements.extend(speedtest::run_tests(
                &client,
                speedtest::test_upload,
                TestType::Upload,
                vec![1_000_000],
                16,
                cfspeedtest::OutputFormat::StdOut,
                false,
            ));

            Report::PingSucceeded(
                (format!("{metadata}"), avg_latency, measurements).into(),
            )
        }
        Err(err) => {
            println!("ping failed! reason: {err:?}");
            Report::PingFailed
        },
    };

    println!("writing report to database...");
    report.insert(now, &connection)?;

    println!("done!");

    Ok(())
}
