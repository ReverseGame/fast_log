use fast_log::appender::{FastLogRecord, LogAppender};
use fast_log::config::Config;
use fastdate::DateTime;
use log::{Level, Log};
use fast_log::{error, info, Loggers};

struct CustomLog {}

impl LogAppender for CustomLog {
    fn do_logs(&mut self, records: &[FastLogRecord]) {
        for record in records {
            let now = DateTime::from(record.now);
            let data;
            match record.level {
                Level::Warn | Level::Error => {
                    data = format!(
                        "{} {} {} - {}\n",
                        now, record.level, record.module_path, record.args,
                    );
                }
                _ => {
                    data = format!(
                        "{} {} {} - {}\n",
                        &now, record.level, record.module_path, record.args
                    );
                }
            }
            print!("{}", data);
        }
    }
}

fn main() {
    let logger = Loggers::new("test",Config::new().custom(CustomLog {}));
    info!(logger: &logger, "Commencing yak shaving");
    error!(logger: &logger ,"Commencing error");
    logger.flush();
}
