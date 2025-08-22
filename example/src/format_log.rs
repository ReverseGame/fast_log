use fast_log::config::Config;
use fast_log::{FastLogFormat, Loggers};
use log::{LevelFilter, Log};

fn main() {
    let logger = Loggers::new(
        "test",
        Config::new()
            .format(FastLogFormat::new().set_display_line_level(LevelFilter::Trace))
            .console(),
    );
    log::info!(logger: &logger, "Commencing yak shaving{}", 0);
    logger.flush();
}
