use criterion::Criterion;
use fast_log::appender::{FastLogRecord, LogAppender};
use fast_log::{Config, Loggers, info};

pub fn bench_log(c: &mut Criterion) {
    struct BenchRecvLog {}
    impl LogAppender for BenchRecvLog {
        fn do_logs(&mut self, _records: &[FastLogRecord]) {
            //nothing
        }
    }
    let logger = Loggers::new(
        "test1",
        Config::new()
            .custom(BenchRecvLog {})
            .chan_len(Some(1000000)),
    );
    c.bench_function("log", |b| {
        b.iter(|| {
            info!(logger: &logger, "Commencing yak shaving");
        });
    });
}
