use crate::log::bench_log;
use crate::log_file::bench_log_file;
use criterion::{criterion_group, criterion_main};

pub mod log;
pub mod log_file;

criterion_group!(benches, bench_log, bench_log_file);
criterion_main!(benches);
