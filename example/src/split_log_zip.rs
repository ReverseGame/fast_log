use fast_log::consts::LogSize;
use fast_log::plugin::file_split::{KeepType, Rolling, RollingType};
use fast_log::plugin::packer::ZipPacker;

use fast_log::config::Config;

fn main() {
    fast_log::init(Config::new().chan_len(Some(100000)).console().file_split(
        "target/logs/",
        Rolling::new(RollingType::BySize(LogSize::KB(50))),
        KeepType::KeepNum(5),
        ZipPacker {},
    ))
    .unwrap();
    for _ in 0..20000 {
        log::info!("Commencing yak shaving");
    }
    log::logger().flush();
    println!("you can see log files in path: {}", "target/logs/")
}
