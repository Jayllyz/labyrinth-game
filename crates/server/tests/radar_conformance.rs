//! Checks the radar encoding against views recorded from the reference server
//! (`e2e/server-arm --debug`) on the same mazes.

use server::game::{Game, Position};
use shared::{
    heading::Heading,
    maze::{Cell, Maze, PositionType},
};

struct Fixture {
    maze: Maze,
    exit: Position,
    views: Vec<(Position, Heading, String)>,
}

/// Fixture layout: a comment, `exit x y`, the maze as exported by
/// `server-arm generate maze`, then one `x y heading view` line per recorded view.
fn parse(text: &str) -> Fixture {
    let mut lines = text.lines().skip(1);
    let exit: Vec<i16> = lines
        .next()
        .and_then(|line| line.strip_prefix("exit "))
        .map_or_else(Vec::new, |rest| rest.split(' ').filter_map(|n| n.parse().ok()).collect());
    let (maze_lines, view_lines): (Vec<&str>, Vec<&str>) =
        lines.partition(|line| !line.starts_with(|c: char| c.is_ascii_digit()));

    let map = maze_lines
        .iter()
        .map(|line| {
            line.chars()
                .map(|c| if c == ' ' { PositionType::SPACE } else { PositionType::WALL })
                .collect()
        })
        .collect();
    let origin = Cell { row: 0, column: 0 };
    let views = view_lines
        .iter()
        .map(|line| {
            let fields: Vec<&str> = line.split(' ').collect();
            let heading = match fields[2] {
                "North" => Heading::North,
                "East" => Heading::East,
                "South" => Heading::South,
                _ => Heading::West,
            };
            let position =
                Position { x: fields[0].parse().unwrap_or(-1), y: fields[1].parse().unwrap_or(-1) };
            (position, heading, fields[3].to_string())
        })
        .collect();
    Fixture {
        maze: Maze::new(map, origin, origin),
        exit: Position { x: exit[0], y: exit[1] },
        views,
    }
}

fn check(name: &str, text: &str) {
    let fixture = parse(text);
    let mismatches: Vec<String> = fixture
        .views
        .iter()
        .filter_map(|(position, heading, expected)| {
            let mut game = Game::new(fixture.maze.clone(), fixture.exit);
            let id = game.add_player(0, *position, *heading);
            let actual = game.radar(id);
            (actual != *expected)
                .then(|| format!("{position:?} {heading:?}: expected {expected}, got {actual}"))
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "{name}: {} mismatches\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

#[test]
fn test_radar_matches_reference_server() {
    check("recursive-backtracking-1", include_str!("fixtures/recursive-backtracking-1.txt"));
    check("prims-2", include_str!("fixtures/prims-2.txt"));
    check("growing-tree-3", include_str!("fixtures/growing-tree-3.txt"));
    check("recursive-backtracking-5", include_str!("fixtures/recursive-backtracking-5.txt"));
}
