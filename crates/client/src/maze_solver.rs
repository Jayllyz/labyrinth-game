use std::collections::VecDeque;

use shared::maze::{Cell, Directions, Maze};

pub fn bfs_shortest_path(maze: &Maze, print: bool) -> Vec<Cell> {
    let mut queue: VecDeque<Cell> = VecDeque::new();

    let Maze { entry, exit, row_len, col_len, .. } = *maze;
    queue.push_back(maze.entry);

    let mut visited_points: Vec<Vec<bool>> = vec![vec![false; col_len]; row_len];
    visited_points[entry.row as usize][entry.column as usize] = true;

    let mut previous_path: Vec<Vec<Cell>> =
        vec![vec![Cell { row: -1, column: -1 }; col_len]; row_len];

    let directions = [Directions::NORTH, Directions::SOUTH, Directions::WEST, Directions::EAST];

    while !queue.is_empty() {
        let curr: Cell = queue.pop_front().unwrap();

        if curr == exit {
            if print {
                maze.print_visited(&visited_points);
            }
            return reconstruct_shortest_path(maze, previous_path, print);
        }

        for direction in directions.iter() {
            let neighbour_point: Cell = curr + *direction;

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

fn reconstruct_shortest_path(maze: &Maze, previous_path: Vec<Vec<Cell>>, print: bool) -> Vec<Cell> {
    let mut shortest_path: Vec<Cell> = Vec::new();
    const NO_PREV_PATH: Cell = Cell { row: -1, column: -1 };
    let mut end = maze.exit;

    while end != NO_PREV_PATH {
        shortest_path.push(end);
        end = previous_path[end.row as usize][end.column as usize];
    }

    shortest_path.reverse();
    if print {
        maze.print_path(&shortest_path);
    }
    shortest_path
}

#[cfg(test)]
mod tests {
    use shared::maze_generator::sidewinder;

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
        let maze = Maze::new(maze_map, Cell { row: 1, column: 4 }, Cell { row: 2, column: 1 });

        let shortest_path = vec![
            Cell { row: 1, column: 4 },
            Cell { row: 1, column: 5 },
            Cell { row: 2, column: 5 },
            Cell { row: 3, column: 5 },
            Cell { row: 4, column: 5 },
            Cell { row: 5, column: 5 },
            Cell { row: 5, column: 4 },
            Cell { row: 5, column: 3 },
            Cell { row: 6, column: 3 },
            Cell { row: 7, column: 3 },
            Cell { row: 7, column: 2 },
            Cell { row: 7, column: 1 },
            Cell { row: 6, column: 1 },
            Cell { row: 5, column: 1 },
            Cell { row: 4, column: 1 },
            Cell { row: 3, column: 1 },
            Cell { row: 2, column: 1 },
        ];

        assert_eq!(bfs_shortest_path(&maze, false), shortest_path);

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
        let maze = Maze::new(maze_map, Cell { row: 7, column: 10 }, Cell { row: 1, column: 1 });

        let shortest_path = vec![
            Cell { row: 7, column: 10 },
            Cell { row: 7, column: 9 },
            Cell { row: 7, column: 8 },
            Cell { row: 7, column: 7 },
            Cell { row: 6, column: 7 },
            Cell { row: 6, column: 6 },
            Cell { row: 6, column: 5 },
            Cell { row: 6, column: 4 },
            Cell { row: 5, column: 4 },
            Cell { row: 4, column: 4 },
            Cell { row: 4, column: 3 },
            Cell { row: 3, column: 3 },
            Cell { row: 2, column: 3 },
            Cell { row: 1, column: 3 },
            Cell { row: 1, column: 2 },
            Cell { row: 1, column: 1 },
        ];

        assert_eq!(bfs_shortest_path(&maze, false), shortest_path);
    }

    #[test]
    fn test_random_generated() {
        let maze = sidewinder(10, 10, false);
        let shortest_path = bfs_shortest_path(&maze, false);
        assert!(!shortest_path.is_empty());
    }
}
