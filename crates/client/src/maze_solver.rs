use crate::data_structures::priority_queue::{Node, PriorityQueue};
use shared::maze::{Cell, Directions, Maze};
use std::collections::VecDeque;

/// Algorithme du breadth-first search (BFS) pour trouver le chemin le plus court
/// 
/// ## Arguments
/// 
/// * `maze` - La structure de données du labyrinthe
/// * `print` - Un u8 pour afficher le labyrinthe
///     * 0 - Ne pas afficher
///     * 1 - Afficher les points visités
///     * 2 - Afficher les points visités numérotés
pub fn bfs_shortest_path(maze: &Maze, print: u8) -> Vec<Cell> {
    if (print != 0) && (print != 1) && (print != 2) {
        panic!("Invalid print value");
    }
    let mut queue: VecDeque<Cell> = VecDeque::new();

    let Maze { entry, exit, row_len, col_len, .. } = *maze;
    queue.push_back(maze.entry);

    let mut visited_points: Vec<Vec<i32>> = vec![vec![-1; col_len]; row_len];
    visited_points[entry.row as usize][entry.column as usize] = 0;
    
    let mut previous_path: Vec<Vec<Cell>> =
    vec![vec![Cell { row: -1, column: -1 }; col_len]; row_len];
    
    let directions = [Directions::NORTH, Directions::SOUTH, Directions::WEST, Directions::EAST];
    let mut index = 1;
    
    while !queue.is_empty() {
        let curr: Cell = queue.pop_front().unwrap();

        if curr == exit {
            if print == 1 {
                maze.print_visited(&visited_points);
                println!("Number of steps: {}", index);
            }
            if print == 2 {
                maze.print_visited_number(&visited_points);
                println!("Number of steps: {}", index);
            }
            return reconstruct_shortest_path(maze, previous_path);
        }

        for direction in directions.iter() {
            let neighbour_cell: Cell = curr + *direction;

            if maze.is_cell_out_of_bound(&neighbour_cell)
                || !maze.is_cell_walkable(&neighbour_cell, &visited_points)
                || visited_points[neighbour_cell.row as usize][neighbour_cell.column as usize] != -1
            {
                continue;
            }

            let row = neighbour_cell.row as usize;
            let column = neighbour_cell.column as usize;

            queue.push_back(neighbour_cell);
            visited_points[row][column] = index;
            previous_path[row][column].row = curr.row;
            previous_path[row][column].column = curr.column;

            index += 1;
        }
    }
    vec![]
}

fn reconstruct_shortest_path(maze: &Maze, previous_path: Vec<Vec<Cell>>) -> Vec<Cell> {
    let mut shortest_path: Vec<Cell> = Vec::new();
    const NO_PREV_PATH: Cell = Cell { row: -1, column: -1 };
    let mut end = maze.exit;

    while end != NO_PREV_PATH {
        shortest_path.push(end);
        end = previous_path[end.row as usize][end.column as usize];
    }

    shortest_path.reverse();
    shortest_path
}

fn get_manhattan_distance(source_cell: &Cell, goal_cell: &Cell) -> i32 {
    ((source_cell.row - goal_cell.row).abs() + (source_cell.column - goal_cell.column).abs()).into()
}

pub fn a_star_shortest_path(maze: &Maze, print: u8) -> Vec<Cell> {
    if (print != 0) && (print != 1) && (print != 2) {
        panic!("Invalid print value");
    }
    let Maze { entry, exit, row_len, col_len, .. } = *maze;
    let directions = [Directions::NORTH, Directions::SOUTH, Directions::WEST, Directions::EAST];

    let mut g_cost = vec![vec![-1; col_len]; row_len];
    let mut f_cost = vec![vec![-1; col_len]; row_len];

    let mut previous_path: Vec<Vec<Cell>> =
        vec![vec![Cell { row: -1, column: -1 }; col_len]; row_len];
    let mut visited_points: Vec<Vec<i32>> = vec![vec![-1; col_len]; row_len];

    let start_row = entry.row as usize;
    let start_column = entry.column as usize;

    g_cost[start_row][start_column] = 0;
    f_cost[start_row][start_column] = get_manhattan_distance(&entry, &exit);

    let mut open = PriorityQueue::new();
    let mut index = 0;

    open.enqueue(Node { priority_f: f_cost[start_row][start_column], cell: entry });

    while !open.is_empty() {
        let Node { cell: curr_cell, .. } = open.dequeue();

        let curr_row = curr_cell.row as usize;
        let curr_col = curr_cell.column as usize;

        visited_points[curr_row][curr_col] = index;

        if curr_cell.row == maze.exit.row && curr_cell.column == maze.exit.column {
            if print == 1 {
                maze.print_visited(&visited_points);
                println!("Number of steps: {}", index);
            }
            if print == 2 {
                maze.print_visited_number(&visited_points);
                println!("Number of steps: {}", index);
            }
            return reconstruct_shortest_path(maze, previous_path);
        }

        for direction in directions.iter() {
            let neighbour_cell: Cell = curr_cell + *direction;

            if maze.is_cell_out_of_bound(&neighbour_cell)
                || !maze.is_cell_walkable(&neighbour_cell, &visited_points)
                || visited_points[neighbour_cell.row as usize][neighbour_cell.column as usize] != -1
            {
                continue;
            }

            let neighbour_g_score = g_cost[curr_row][curr_col] + 1;

            if neighbour_g_score
                < g_cost[neighbour_cell.row as usize][neighbour_cell.column as usize]
                || g_cost[neighbour_cell.row as usize][neighbour_cell.column as usize] < 0
            {
                if open.contains(&neighbour_cell) {
                    continue;
                }
                let neighbour_h_score = get_manhattan_distance(&neighbour_cell, &exit);
                let neighbour_f_score = neighbour_g_score + neighbour_h_score;

                previous_path[neighbour_cell.row as usize][neighbour_cell.column as usize] =
                    curr_cell;
                g_cost[neighbour_cell.row as usize][neighbour_cell.column as usize] =
                    neighbour_g_score;
                f_cost[neighbour_cell.row as usize][neighbour_cell.column as usize] =
                    neighbour_f_score;
                open.enqueue(Node { priority_f: neighbour_f_score, cell: neighbour_cell });
            }
        }
        index += 1;
    }
    vec![]
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

        assert_eq!(bfs_shortest_path(&maze, 1), shortest_path);

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

        assert_eq!(bfs_shortest_path(&maze, 0), shortest_path);
    }

    #[test]
    fn test_a_star_exit_finder() {
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

        assert_eq!(a_star_shortest_path(&maze, 0), shortest_path);

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

        assert_eq!(a_star_shortest_path(&maze, 0), shortest_path);
    }

    #[test]
    fn test_random_generated() {
        let maze = sidewinder(10, 10, false);
        let shortest_path = bfs_shortest_path(&maze, 0);
        assert!(!shortest_path.is_empty());
    }
}
