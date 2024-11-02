use crate::maze::{Maze, Position};
use std::collections::VecDeque;

pub struct Directions;

impl Directions {
    pub const NORTH: Position = Position { row: -1, column: 0 };
    pub const SOUTH: Position = Position { row: 1, column: 0 };
    pub const WEST: Position = Position { row: 0, column: -1 };
    pub const EAST: Position = Position { row: 0, column: 1 };
}

pub fn bfs_shortest_path(maze: &Maze) -> Vec<Position> {
    let mut queue: VecDeque<Position> = VecDeque::new();

    let Maze { entry, exit, row_len, col_len, .. } = *maze;
    queue.push_back(maze.entry);

    let mut visited_points: Vec<Vec<bool>> = vec![vec![false; col_len]; row_len];
    visited_points[entry.row as usize][entry.column as usize] = true;

    let mut previous_path: Vec<Vec<Position>> =
        vec![vec![Position { row: -1, column: -1 }; col_len]; row_len];

    let directions = [Directions::NORTH, Directions::SOUTH, Directions::WEST, Directions::EAST];

    while !queue.is_empty() {
        let curr: Position = queue.pop_front().unwrap();

        if curr == exit {
            maze.print(&visited_points);
            return reconstruct_shortest_path(maze, previous_path);
        }

        for direction in directions.iter() {
            let neighbour_point: Position = curr + *direction;

            if maze.is_point_out_of_bound(&neighbour_point)
                || !maze.is_point_walkable(&neighbour_point, &visited_points)
            {
                continue;
            }

            let row = neighbour_point.row as usize;
            let column = neighbour_point.column as usize;

            queue.push_back(neighbour_point);
            visited_points[row][column] = true;
            previous_path[row][column].row = curr.row;
            previous_path[row][column].column = curr.column;
        }
    }
    vec![]
}

fn reconstruct_shortest_path(maze: &Maze, previous_path: Vec<Vec<Position>>) -> Vec<Position> {
    let mut shortest_path: Vec<Position> = Vec::new();
    const NO_PREV_PATH: Position = Position { row: -1, column: -1 };
    let mut end = maze.exit;

    while end != NO_PREV_PATH {
        shortest_path.push(end);
        end = previous_path[end.row as usize][end.column as usize];
    }

    shortest_path.reverse();
    maze.print_path(&shortest_path);
    shortest_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bfs_exit_finder() {
        let maze_map = vec![
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            vec![1, 0, 1, 0, 2, 0, 1, 0, 0, 0, 1],
            vec![1, 3, 1, 1, 1, 0, 1, 0, 1, 1, 1],
            vec![1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1],
            vec![1, 0, 1, 1, 1, 0, 1, 0, 1, 1, 1],
            vec![1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1],
            vec![1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1],
            vec![1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1],
            vec![1, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1],
            vec![1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1],
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];
        let maze =
            Maze::new(maze_map, Position { row: 1, column: 4 }, Position { row: 2, column: 1 });

        let shortest_path = vec![
            Position { row: 1, column: 4 },
            Position { row: 1, column: 5 },
            Position { row: 2, column: 5 },
            Position { row: 3, column: 5 },
            Position { row: 4, column: 5 },
            Position { row: 5, column: 5 },
            Position { row: 5, column: 4 },
            Position { row: 5, column: 3 },
            Position { row: 6, column: 3 },
            Position { row: 7, column: 3 },
            Position { row: 7, column: 2 },
            Position { row: 7, column: 1 },
            Position { row: 6, column: 1 },
            Position { row: 5, column: 1 },
            Position { row: 4, column: 1 },
            Position { row: 3, column: 1 },
            Position { row: 2, column: 1 },
        ];

        assert_eq!(bfs_shortest_path(&maze), shortest_path);

        let maze_map = vec![
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            vec![1, 3, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1],
            vec![1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1],
            vec![1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1],
            vec![1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1],
            vec![1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 2, 0, 1],
            vec![1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1],
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];
        let maze =
            Maze::new(maze_map, Position { row: 7, column: 10 }, Position { row: 1, column: 1 });

        let shortest_path = vec![
            Position { row: 7, column: 10 },
            Position { row: 7, column: 9 },
            Position { row: 7, column: 8 },
            Position { row: 7, column: 7 },
            Position { row: 6, column: 7 },
            Position { row: 6, column: 6 },
            Position { row: 6, column: 5 },
            Position { row: 6, column: 4 },
            Position { row: 5, column: 4 },
            Position { row: 4, column: 4 },
            Position { row: 4, column: 3 },
            Position { row: 3, column: 3 },
            Position { row: 2, column: 3 },
            Position { row: 1, column: 3 },
            Position { row: 1, column: 2 },
            Position { row: 1, column: 1 },
        ];

        assert_eq!(bfs_shortest_path(&maze), shortest_path);
    }
}
