extern crate client;
extern crate criterion as criterion2;
extern crate shared;
use client::maze_parser::maze_parser;
use client::maze_solver::{bfs_shortest_path, PrintPathMode};
use criterion2::{black_box, criterion_group, criterion_main, Criterion};
use shared::maze::Maze;
use std::time::Duration;

struct BenchData {
    maze1: String,
    maze2: String,
    maze3: String,
    maze4: String,
    maze5: String,
}

struct MazeData {
    maze1: Maze,
    maze2: Maze,
    maze3: Maze,
    maze4: Maze,
    maze5: Maze,
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

fn parse_all_mazes() -> MazeData {
    let data = load_bench_data();

    MazeData {
        maze1: maze_parser(&data.maze1),
        maze2: maze_parser(&data.maze2),
        maze3: maze_parser(&data.maze3),
        maze4: maze_parser(&data.maze4),
        maze5: maze_parser(&data.maze5),
    }
}

fn bench_bfs_solver(c: &mut Criterion) {
    let parsed_mazes = parse_all_mazes();

    let mut group = c.benchmark_group("bfs_solver");
    group.warm_up_time(Duration::from_secs(3));

    group.bench_function("maze1", |b| {
        b.iter(|| bfs_shortest_path(black_box(&parsed_mazes.maze1), PrintPathMode::None))
    });
    group.bench_function("maze2", |b| {
        b.iter(|| bfs_shortest_path(black_box(&parsed_mazes.maze2), PrintPathMode::None))
    });
    group.bench_function("maze3", |b| {
        b.iter(|| bfs_shortest_path(black_box(&parsed_mazes.maze3), PrintPathMode::None))
    });
    group.bench_function("maze4", |b| {
        b.iter(|| bfs_shortest_path(black_box(&parsed_mazes.maze4), PrintPathMode::None))
    });
    group.bench_function("maze5", |b| {
        b.iter(|| bfs_shortest_path(black_box(&parsed_mazes.maze5), PrintPathMode::None))
    });

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .warm_up_time(Duration::from_secs(3));
    targets = bench_bfs_solver
);
criterion_main!(benches);
