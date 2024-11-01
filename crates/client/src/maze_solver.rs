use crate::maze::{Maze, Point};
use std::collections::VecDeque;

pub fn bfs_maze(maze: &Maze) -> bool {
    let mut queue: VecDeque<Point> = VecDeque::new();

    let Maze { entry, exit, row_len, col_len, .. } = *maze;
    queue.push_back(maze.entry);

    let mut visited: Vec<Vec<bool>> = vec![vec![false; col_len]; row_len];
    visited[entry.row as usize][entry.column as usize] = true;

    let directions = [
        Point { row: -1, column: 0 },
        Point { row: 1, column: 0 },
        Point { row: 0, column: 1 },
        Point { row: 0, column: -1 },
    ];

    while !queue.is_empty() {
        let curr: Point = queue.pop_front().unwrap();

        if curr == exit {
            maze.print(&visited);
            return true;
        }

        for direction in directions.iter() {
            let neighbor: Point = curr + *direction;

            if neighbor.row < 0
                || neighbor.column < 0
                || neighbor.row >= (row_len as i8)
                || neighbor.column >= (col_len as i8)
            {
                continue;
            }

            if visited[neighbor.row as usize][neighbor.column as usize]
                || maze.map[neighbor.row as usize][neighbor.column as usize] == 1
            {
                continue;
            }

            queue.push_back(neighbor);
            visited[neighbor.row as usize][neighbor.column as usize] = true;
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
