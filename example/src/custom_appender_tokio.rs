use fast_log::appender::{FastLogRecord, LogAppender};
use fast_log::config::Config;
use fastdate::DateTime;
use log::{Level, Log};
use tokio::runtime::Runtime;
use fast_log::Loggers;

struct CustomLog {
    rt: Runtime,
}

impl LogAppender for CustomLog {
    fn do_logs(&mut self, records: &[FastLogRecord]) {
        let mut datas = String::new();
        for record in records {
            let now = DateTime::from(record.now);
            let data;
            match record.level {
                Level::Warn | Level::Error => {
                    data = format!(
                        "{} {} {} - {}  {}\n",
                        now, record.level, record.module_path, record.args, record.formated
                    );
                }
                _ => {
                    data = format!(
                        "{} {} {} - {}\n",
                        &now, record.level, record.module_path, record.args
                    );
                }
            }
            datas.push_str(&data);
        }
        self.rt.block_on(async move {
            //send to web,file,any way
            print!("{}", datas);
        });
    }
}

#[tokio::main]
async fn main() {
    let logger = Loggers::new("custom",  Config::new().custom(CustomLog {
        rt: tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap(),
    }));
    log::info!(logger: &logger, "Commencing yak shaving");
    log::error!(logger: &logger, "Commencing error");
    logger.flush();
}
