use fast_log::config::Config;
use fast_log::{FastLogFormatJson, Loggers};
use log::Log;
use std::sync::Arc;

fn main() {
    log::set_logger(&Loggers { key: "test" }).unwrap();
    log::set_max_level(log::LevelFilter::Info);
    let logger = Arc::new(Loggers::new(
        "test",
        Config::new().format(FastLogFormatJson::new()).console(),
    ));
    log::info!("Commencing 11111yak shaving");
    // log::info!(logger: &logger, "Commencing \"yak\" shaving{}", 0);
    // logger.flush();
    log::logger().flush()
}
