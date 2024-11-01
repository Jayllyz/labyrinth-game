use crate::maze::{Maze, Point};
use std::collections::VecDeque;

pub struct Directions;

impl Directions {
    pub const NORTH: Point = Point { row: -1, column: 0 };
    pub const SOUTH: Point = Point { row: 1, column: 0 };
    pub const WEST: Point = Point { row: 0, column: -1 };
    pub const EAST: Point = Point { row: 0, column: 1 };
}

pub fn bfs_maze(maze: &Maze) -> bool {
    let mut queue: VecDeque<Point> = VecDeque::new();

    let Maze { entry, exit, row_len, col_len, .. } = *maze;
    queue.push_back(maze.entry);

    let mut visited_points: Vec<Vec<bool>> = vec![vec![false; col_len]; row_len];
    visited_points[entry.row as usize][entry.column as usize] = true;

    let directions = [Directions::NORTH, Directions::SOUTH, Directions::WEST, Directions::EAST];

    while !queue.is_empty() {
        let curr: Point = queue.pop_front().unwrap();

        if curr == exit {
            maze.print(&visited_points);
            return true;
        }

        for direction in directions.iter() {
            let neighbour_point: Point = curr + *direction;

            if maze.is_out_of_bound_point(&neighbour_point)
                || !maze.is_walkable_point(&neighbour_point, &visited_points)
            {
                continue;
            }

            queue.push_back(neighbour_point);
            visited_points[neighbour_point.row as usize][neighbour_point.column as usize] = true;
        }
    }
    false
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
        let maze = Maze::new(maze_map, Point { row: 1, column: 4 }, Point { row: 2, column: 1 });

        assert_eq!(bfs_maze(&maze), true);

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
        let maze = Maze::new(maze_map, Point { row: 7, column: 10 }, Point { row: 1, column: 1 });

        assert_eq!(bfs_maze(&maze), true);
    }
}
