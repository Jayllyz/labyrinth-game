use crate::data_structures::{
    maze_graph::MazeGraph,
    priority_queue::{Node, PriorityQueue},
};
use shared::maze::Cell;
use std::collections::{HashMap, HashSet};

pub enum PrintPathMode {
    None = 0,
    Visited = 1,
    VisitedNumber = 2,
}

pub fn a_star_shortest_path(maze_graph: &mut MazeGraph, start: Cell, exit: Cell) -> Vec<Cell> {
    let mut g_cost = HashMap::new();
    let mut f_cost = HashMap::new();
    let mut open = PriorityQueue::new();

    let mut visited_points: HashSet<Cell> = HashSet::new();
    let mut parents: HashMap<Cell, Cell> = HashMap::new();

    g_cost.insert(start, 0);
    f_cost.insert(start, get_manhattan_distance(&start, &exit));

    open.enqueue(Node { priority_f: f_cost[&start], cell: start });

    while !open.is_empty() {
        let Node { cell: curr_cell, .. } = open.dequeue();

        if curr_cell == exit {
            return reconstruct_shortest_path_graph(parents, start, exit);
        }

        visited_points.insert(curr_cell);

        let cell_data = maze_graph.get_cell(curr_cell).cloned();

        if let Some(current_maze_cell) = cell_data {
            for neighbor_cell in &current_maze_cell.neighbors {
                if visited_points.contains(neighbor_cell) {
                    continue;
                }

                let tentative_g_score = g_cost[&curr_cell] + 1;

                if tentative_g_score < *g_cost.get(neighbor_cell).unwrap_or(&i32::MAX) {
                    g_cost.insert(*neighbor_cell, tentative_g_score);
                    let h_cost = get_manhattan_distance(neighbor_cell, &exit);
                    let f_score = tentative_g_score + h_cost;
                    f_cost.insert(*neighbor_cell, f_score);

                    visited_points.insert(*neighbor_cell);
                    parents.insert(*neighbor_cell, curr_cell);

                    open.enqueue(Node { priority_f: f_score, cell: *neighbor_cell });
                }
            }
        }
    }

    vec![]
}

fn reconstruct_shortest_path_graph(
    parents: HashMap<Cell, Cell>,
    start: Cell,
    exit: Cell,
) -> Vec<Cell> {
    let mut shortest_path: Vec<Cell> = Vec::new();
    let mut current_cell = exit;

    while current_cell != start {
        shortest_path.push(current_cell);

        if current_cell == parents[&current_cell] {
            break;
        }

        current_cell = parents[&current_cell];
    }

    shortest_path.push(start);

    shortest_path.reverse();

    shortest_path
}

fn get_manhattan_distance(source_cell: &Cell, goal_cell: &Cell) -> i32 {
    ((source_cell.row - goal_cell.row).abs() + (source_cell.column - goal_cell.column).abs()).into()
}
#[cfg(test)]
mod tests {
    use shared::radar::CellType;

    fn convert_to_maze_graph(maze_map: Vec<Vec<u16>>) -> MazeGraph {
        let mut maze_graph = MazeGraph::new();
        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        for (row, row_cells) in maze_map.iter().enumerate() {
            for (col, &cell_value) in row_cells.iter().enumerate() {
                let cell = Cell { row: row as i16, column: col as i16 };

                let cell_type = match cell_value {
                    0 => CellType::NOTHING,
                    2 => CellType::OBJECTIVE,
                    3 => CellType::NOTHING,
                    _ => continue,
                };

                maze_graph.add(cell, cell_type);

                for direction in &directions {
                    let neighbor_row = row as i16 + direction.0;
                    let neighbor_col = col as i16 + direction.1;

                    if neighbor_row >= 0
                        && neighbor_row < maze_map.len() as i16
                        && neighbor_col >= 0
                        && neighbor_col < maze_map[0].len() as i16
                        && maze_map[neighbor_row as usize][neighbor_col as usize] != 1
                    {
                        let neighbor_cell = Cell { row: neighbor_row, column: neighbor_col };
                        maze_graph.add_neighbor(&cell, &neighbor_cell);
                    }
                }
            }
        }

        maze_graph
    }

    use super::*;
    #[test]
    fn test_a_star_exit_finder() {
        // vec![1, 0, 3],
        // vec![1, 0, 1],
        // vec![2, 0, 1],
        let maze_map = vec![vec![1, 0, 3], vec![1, 0, 1], vec![2, 0, 1]];
        let mut m: MazeGraph = convert_to_maze_graph(maze_map);

        let shortest_path = vec![
            Cell { row: 2, column: 0 },
            Cell { row: 2, column: 1 },
            Cell { row: 1, column: 1 },
            Cell { row: 0, column: 1 },
            Cell { row: 0, column: 2 },
        ];

        assert_eq!(
            a_star_shortest_path(&mut m, Cell { row: 2, column: 0 }, Cell { row: 0, column: 2 }),
            shortest_path
        );
    }
}
