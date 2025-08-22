use fast_log::bencher::TPS;
use fast_log::config::Config;
use std::time::Instant;
use log::{log, LevelFilter};
use fast_log::Loggers;

/// cargo run --release --package example --bin bench_test_file
fn main() {
    //clear data
    let _ = std::fs::remove_file("target/test.log");
    log::set_logger(&Loggers)
        .map(|()| log::set_max_level(LevelFilter::Debug)).unwrap();
    fast_log::init(
        Config::new()
            .file("target/test.log")
            .chan_len(Some(1000000)), "test"
    )
    .unwrap();
    let total = 1000000;
    let now = Instant::now();
    for index in 0..total {
        log::info!("Commencing yak shaving{}", index);
    }
    log::__private_api::log(log::Level::Info, "Commencing yak shaving")
    //wait log finish write all
    log::logger().flush();
    now.time(total);
    now.tps(total);
}
