use fast_log::{Config, Loggers, info};

use criterion::Criterion;
pub fn bench_log_file(c: &mut Criterion) {
    let _ = std::fs::remove_file("target/test_bench.log");
    let logger = Loggers::new(
        "test",
        Config::new()
            .file("target/test_bench.log")
            .chan_len(Some(1000000)),
    );
    c.bench_function("log_file", |b| {
        b.iter(|| {
            info!(logger: &logger, "Commencing yak shaving");
        });
    });
}
