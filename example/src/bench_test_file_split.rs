use std::sync::Arc;
use fast_log::bencher::TPS;
use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::plugin::file_split::{KeepType, Rolling, RollingType};
use fast_log::plugin::packer::LogPacker;
use std::time::Instant;
use log::LevelFilter;
use fast_log::Loggers;
use fast_log::info;

/// cargo run --release --package example --bin bench_test_file_split
fn main() {
    //clear data
    log::set_logger(&Loggers{key: "test"})
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .unwrap();
    let _ = std::fs::remove_dir("target/logs/");
    let config = Config::new()
        .file_split(
            "target/logs/",
            Rolling::new(RollingType::BySize(LogSize::MB(1))),
            KeepType::All,
            LogPacker {},
        )
        .chan_len(Some(100000));
    let logger = Loggers::new(
        "test",
        config,
        
    );
    let total = 1000000;
    let now = Instant::now();
    for index in 0..total {
        info!(logger: &logger, "Commencing yak shaving{}", index);
    }
    //wait log finish write all
    log::logger().flush();
    now.time(total);
    now.tps(total);
}
