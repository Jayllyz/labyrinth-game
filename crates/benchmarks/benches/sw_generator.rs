use std::time::Duration;
extern crate criterion as criterion2;
use criterion2::{Criterion, criterion_group, criterion_main};
use shared::maze_generator::{GeneratorAlgorithm, generate};

fn bench_sidewinder_generator(c: &mut Criterion) {
    let mut group = c.benchmark_group("sidewinder_generator");
    group.warm_up_time(Duration::from_secs(3));

    for (index, size) in [10, 20, 30, 40, 50].into_iter().enumerate() {
        group.bench_function(format!("maze{}", index + 1), |b| {
            b.iter(|| generate(GeneratorAlgorithm::Sidewinder, size, size, false, 5849));
        });
    }

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
