use std::{net::Ipv4Addr, str::FromStr, time::Duration};

use cfspeedtest::SpeedTestCLIOptions;
use itertools::Itertools;
use ping_rs::send_ping;

fn main() {
    send_ping(&std::net::IpAddr::from_str("1.1.1.1").expect("this is a valid ip you shit"), Duration::from_secs(5), b"rawr xd uwu", None).expect("ping failed");

    let options = SpeedTestCLIOptions {
        nr_tests: 4,
        nr_latency_tests: 4,
        max_payload_size: cfspeedtest::speedtest::PayloadSize::M1,
        output_format: cfspeedtest::OutputFormat::JsonPretty,
        verbose: true,
        ipv4: false,
        ipv6: false,
        disable_dynamic_max_payload_size: false
    };

    let measurements = cfspeedtest::speedtest::speed_test(reqwest::blocking::Client::new(), options);

    println!("{}", measurements.iter().format("\n"))
}
