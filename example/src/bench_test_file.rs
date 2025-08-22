use fast_log::bencher::TPS;
use fast_log::config::Config;
use fast_log::{info, Loggers};
use log::{log, LevelFilter, Log};
use std::time::Instant;

/// cargo run --release --package example --bin bench_test_file
fn main() {
    //clear data
    let _ = std::fs::remove_file("target/test.log");
    log::set_logger(&Loggers{key: "test"})
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .unwrap();
    let logger = Loggers::new(
        "test",
        Config::new()
            .file("target/test.log")
            .chan_len(Some(100000)),
        
    );
    let total = 1000000;
    let now = Instant::now();
    for index in 0..total {
        info!(logger: &logger, "Commencing yak shaving{}", index);
    }
    // log::__private_api::log(log::Level::Info, "Commencing yak shaving");
    //wait log finish write all
    logger.flush();
    now.time(total);
    now.tps(total);
}
