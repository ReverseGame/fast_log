use log::Log;
use fast_log::config::Config;
use fast_log::{FastLogFormatJson, Loggers};

fn main() {
    log::set_max_level(log::LevelFilter::Info);
    let logger = Loggers::new("test", Config::new().format(FastLogFormatJson::new()).console());
    log::info!(logger: &logger, "Commencing \"yak\" shaving{}", 0);
    logger.flush();
}
