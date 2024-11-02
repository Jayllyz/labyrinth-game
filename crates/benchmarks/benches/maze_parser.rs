extern crate client;
extern crate criterion as criterion2;
use client::maze_parser::maze_parser;
use criterion2::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

struct BenchData {
    maze1: String,
    maze2: String,
    maze3: String,
    maze4: String,
    maze5: String,
}

fn load_bench_data() -> BenchData {
    BenchData {
        maze1: include_str!("../inputs/maze1.txt").to_string(),
        maze2: include_str!("../inputs/maze2.txt").to_string(),
        maze3: include_str!("../inputs/maze3.txt").to_string(),
        maze4: include_str!("../inputs/maze4.txt").to_string(),
        maze5: include_str!("../inputs/maze5.txt").to_string(),
    }
}

fn bench_maze_parser(c: &mut Criterion) {
    let data = load_bench_data();

    let mut group = c.benchmark_group("maze_parser");
    group.warm_up_time(Duration::from_secs(3));

    group.bench_function("maze1", |b| b.iter(|| maze_parser(black_box(&data.maze1))));
    group.bench_function("maze2", |b| b.iter(|| maze_parser(black_box(&data.maze2))));
    group.bench_function("maze3", |b| b.iter(|| maze_parser(black_box(&data.maze3))));
    group.bench_function("maze4", |b| b.iter(|| maze_parser(black_box(&data.maze4))));
    group.bench_function("maze5", |b| b.iter(|| maze_parser(black_box(&data.maze5))));

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_secs(3));
    targets = bench_maze_parser
);
criterion_main!(benches);
