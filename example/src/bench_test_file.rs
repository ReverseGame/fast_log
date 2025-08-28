use chrono::{DateTime, Utc};
use fast_log::appender::{FastLogRecord, LogAppender, RecordFormat};
use fast_log::bencher::TPS;
use fast_log::config::Config;
use fast_log::consts::LogSize;
use fast_log::filter::Filter;
use fast_log::plugin::console::ConsoleAppender;
use fast_log::plugin::logger_buffer::BufferAppender;
use fast_log::{info, Loggers};
use log::{log, LevelFilter, Log};
use std::time::Instant;

struct LogFormat;

impl RecordFormat for LogFormat {
    fn do_format(&self, arg: &mut FastLogRecord) {
        let now = DateTime::<Utc>::from(arg.now);
        let level = arg.level;
        let message = &arg.args;
        let file = &arg.file;
        let line = arg.line;
        let log_message = if cfg!(any(env = "product", env = "beta")) {
            format!(
                "[{}] [{}]: {}\n",
                now.format("%Y-%m-%d %H:%M:%S%.6f"),
                level,
                message,
            )
        } else {
            format!(
                "[{}] [{}] [{}:{}]: {}\n",
                now.format("%Y-%m-%d %H:%M:%S%.6f"),
                level,
                file,
                line.unwrap_or_default(),
                message
            )
        };
        arg.formated = log_message;
    }
}

struct ConsoleAppenderWithFilter {
    inner: ConsoleAppender,
}

impl LogAppender for ConsoleAppenderWithFilter {
    fn do_logs(&mut self, records: &[fast_log::appender::FastLogRecord]) {
        let records = records
            .iter()
            .filter(|record| record.level <= log::Level::Info)
            .map(|record| {
                let mut record = record.clone();
                // 根据日志级别设置颜色
                let colored_message = match record.level {
                    log::Level::Error => format!("\x1b[31m{}\x1b[0m", record.formated), // 红色
                    log::Level::Warn => format!("\x1b[33m{}\x1b[0m", record.formated),  // 黄色
                    log::Level::Info => format!("\x1b[32m{}\x1b[0m", record.formated),  // 绿色
                    log::Level::Debug => format!("\x1b[36m{}\x1b[0m", record.formated), // 青色
                    log::Level::Trace => format!("\x1b[35m{}\x1b[0m", record.formated), // 紫色
                };
                record.formated = colored_message;
                record
            })
            .collect::<Vec<fast_log::appender::FastLogRecord>>();
        self.inner.do_logs(&records);
    }
}
struct LogFilter;

impl Filter for LogFilter {
    fn do_log(&self, record: &log::Record) -> bool {
        record.file().is_none_or(|file| !file.contains(".cargo"))
    }
}
/// cargo run --release --package example --bin bench_test_file
fn main() {
    //clear data
    let _ = std::fs::remove_file("target/test.log");
    log::set_logger(&Loggers { key: "unknown" })
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .unwrap();
    let filter = LogFilter {};
    let logger = Loggers::new(
        "unknown",
        fast_log::Config::new()
            .format(LogFormat {})
            .add_filter(filter)
            .add_appender(ConsoleAppenderWithFilter {
                inner: ConsoleAppender {},
            })
            .chan_len(Some(5000))
            .add_appender(BufferAppender::new(LogSize::MB(1)).unwrap())
            .level(log::LevelFilter::Debug),
    );
    let total = 1000000;
    let now = Instant::now();
    for index in 0..total {
        log::info!("Commencing yak shaving{}", index);
    }
    // log::__private_api::log(log::Level::Info, "Commencing yak shaving");
    //wait log finish write all
    logger.flush();
    now.time(total);
    now.tps(total);
}
