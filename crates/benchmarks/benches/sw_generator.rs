use std::time::Duration;
extern crate criterion as criterion2;
use criterion2::{criterion_group, criterion_main, Criterion};
use shared::maze_generator::sidewinder;

fn bench_sidewinder_generator(c: &mut Criterion) {
    let mut group = c.benchmark_group("sidewinder_generator");
    group.warm_up_time(Duration::from_secs(3));

    group.bench_function("maze1", |b| b.iter(|| sidewinder(10, 10, false, 5849)));
    group.bench_function("maze2", |b| b.iter(|| sidewinder(20, 20, false, 5849)));
    group.bench_function("maze3", |b| b.iter(|| sidewinder(30, 30, false, 5849)));
    group.bench_function("maze4", |b| b.iter(|| sidewinder(40, 40, false, 5849)));
    group.bench_function("maze5", |b| b.iter(|| sidewinder(50, 50, false, 5849)));

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_secs(3));
    targets = bench_sidewinder_generator
);
criterion_main!(benches);
