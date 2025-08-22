#![feature(test)]
#![feature(bench_black_box)]
extern crate test;

use fast_log::Config;
use fast_log::appender::{FastLogRecord, LogAppender};

use test::{Bencher, black_box};

#[bench]
fn bench_log(b: &mut Bencher) {
    struct BenchRecvLog {}
    impl LogAppender for BenchRecvLog {
        fn do_logs(&mut self, _records: &[FastLogRecord]) {
            //nothing
        }
    }
    fast_log::init(
        Config::new()
            .custom(BenchRecvLog {})
            .chan_len(Some(1000000)),
    )
    .unwrap();
    b.iter(|| {
        black_box({
            log::info!("Commencing yak shaving");
        });
    });
}
