use shared::{
    maze::Cell,
    radar::{CellType, Passages},
};

use crate::data_structures::maze_graph::MazeGraph;

fn rotate_left_90(cells: &mut Vec<Cell>) {
    for cell in cells.iter_mut() {
        let new_row = -cell.column;
        let new_column = cell.row;
        cell.row = new_row;
        cell.column = new_column;
    }
}

fn rotate_right_90(cells: &mut Vec<Cell>) {
    for cell in cells.iter_mut() {
        let new_row = cell.column;
        let new_column = -cell.row;
        cell.row = new_row;
        cell.column = new_column;
    }
}

pub struct Player {
    pub direction: String,
    pub position: Cell,
}

pub fn maze_to_graph(
    (horizontal, vertical, cells): (Vec<Passages>, Vec<Passages>, Vec<CellType>),
    player: &Player,
    maze_graph: &mut MazeGraph,
) {
    let directions_mask = get_direction_mask(player);

    for cell_id in 0..cells.len() {
        if cells[cell_id] == CellType::INVALID {
            continue;
        }

        let cell_pos = player.position + directions_mask[cell_id];

        if !maze_graph.contains(&cell_pos) {
            maze_graph.add(cell_pos, cells[cell_id].clone());
        }

        let mut neigbors_to_add: Vec<Cell> = Vec::new();

        if is_top_cell_accessible(cell_id, &horizontal) {
            let top_cell_id = cell_id - 3;
            let top_cell = player.position + directions_mask[top_cell_id];

            if !maze_graph.contains(&top_cell) {
                maze_graph.add(top_cell, cells[top_cell_id].clone());
            }

            neigbors_to_add.push(top_cell);
        }

        if is_bottom_cell_accessible(cell_id, &horizontal) {
            let bottom_cell_id = cell_id + 3;
            let bottom_cell = player.position + directions_mask[bottom_cell_id];

            if !maze_graph.contains(&bottom_cell) {
                maze_graph.add(bottom_cell, cells[bottom_cell_id].clone());
            }

            neigbors_to_add.push(bottom_cell);
        }

        if is_left_cell_accessible(cell_id, &vertical) {
            let left_cell_id = cell_id - 1;
            let left_cell = player.position + directions_mask[left_cell_id];

            if !maze_graph.contains(&left_cell) {
                maze_graph.add(left_cell, cells[left_cell_id].clone());
            }

            neigbors_to_add.push(left_cell);
        }

        if is_right_cell_accessible(cell_id, &vertical) {
            let right_cell_id = cell_id + 1;
            let right_cell = player.position + directions_mask[right_cell_id];

            if !maze_graph.contains(&right_cell) {
                maze_graph.add(right_cell, cells[right_cell_id].clone());
            }

            neigbors_to_add.push(right_cell);
        }

        for neighbor in neigbors_to_add {
            maze_graph.add_neighbor(&cell_pos, &neighbor);
            maze_graph.add_neighbor(&neighbor, &cell_pos);
        }
    }
}

fn is_right_cell_accessible(cell_id: usize, vertical: &Vec<Passages>) -> bool {
    cell_id % 3 != 2 && vertical[cell_id + cell_id / 3 + 1] == Passages::OPEN
}

fn is_left_cell_accessible(cell_id: usize, vertical: &Vec<Passages>) -> bool {
    cell_id % 3 != 0 && vertical[cell_id + cell_id / 3] == Passages::OPEN
}

fn is_bottom_cell_accessible(cell_id: usize, horizontal: &Vec<Passages>) -> bool {
    cell_id < 6 && horizontal[cell_id + 3] == Passages::OPEN
}

fn is_top_cell_accessible(cell_id: usize, horizontal: &Vec<Passages>) -> bool {
    cell_id > 2 && horizontal[cell_id] == Passages::OPEN
}

fn get_direction_mask(player: &Player) -> Vec<Cell> {
    let mut cell_mask = vec![
        Cell { row: -1, column: -1 },
        Cell { row: 0, column: -1 },
        Cell { row: 1, column: -1 },
        Cell { row: -1, column: 0 },
        Cell { row: 0, column: 0 },
        Cell { row: 1, column: 0 },
        Cell { row: -1, column: 1 },
        Cell { row: 0, column: 1 },
        Cell { row: 1, column: 1 },
    ];

    if player.direction == "right" {
        rotate_left_90(&mut cell_mask);
    }
    if player.direction == "left" {
        rotate_right_90(&mut cell_mask);
    }
    if player.direction == "back" {
        rotate_right_90(&mut cell_mask);
        rotate_right_90(&mut cell_mask);
    }
    cell_mask
}

#[cfg(test)]
mod tests {
    use shared::radar;

    use super::*;

    #[test]
    fn testo() {
        let decoded = radar::decode_base64("Hjeikcyc/W8a8pa");
        let data = radar::extract_data(&decoded);

        let mut p = Player { position: Cell { row: 0, column: 0 }, direction: "front".to_string() };
        let mut m = MazeGraph::new();
        maze_to_graph(data, &p, &mut m);

        println!("{:?}", m);

        p.direction = "right".to_string();
        p.position = p.position + Cell { row: 1, column: 0 };

        let decoded = radar::decode_base64("kOuczzGa//apaaa");
        let data = radar::extract_data(&decoded);
        maze_to_graph(data, &p, &mut m);

        println!("{:?}", m);
    }

    #[test]
    fn test_rotate_left() {
        let mut cell_mask = vec![
            Cell { row: -1, column: -1 },
            Cell { row: 0, column: -1 },
            Cell { row: 1, column: -1 },
            Cell { row: -1, column: 0 },
            Cell { row: 0, column: 0 },
            Cell { row: 1, column: 0 },
            Cell { row: -1, column: 1 },
            Cell { row: 0, column: 1 },
            Cell { row: 1, column: 1 },
        ];

        let expected_rotation = vec![
            Cell { row: 1, column: -1 },
            Cell { row: 1, column: 0 },
            Cell { row: 1, column: 1 },
            Cell { row: 0, column: -1 },
            Cell { row: 0, column: 0 },
            Cell { row: 0, column: 1 },
            Cell { row: -1, column: -1 },
            Cell { row: -1, column: 0 },
            Cell { row: -1, column: 1 },
        ];

        rotate_left_90(&mut cell_mask);

        assert_eq!(cell_mask, expected_rotation)
    }

    #[test]
    fn test_rotate_right() {
        let mut cell_mask = vec![
            Cell { row: -1, column: -1 },
            Cell { row: 0, column: -1 },
            Cell { row: 1, column: -1 },
            Cell { row: -1, column: 0 },
            Cell { row: 0, column: 0 },
            Cell { row: 1, column: 0 },
            Cell { row: -1, column: 1 },
            Cell { row: 0, column: 1 },
            Cell { row: 1, column: 1 },
        ];

        let expected_rotation = vec![
            Cell { row: -1, column: 1 },
            Cell { row: -1, column: 0 },
            Cell { row: -1, column: -1 },
            Cell { row: 0, column: 1 },
            Cell { row: 0, column: 0 },
            Cell { row: 0, column: -1 },
            Cell { row: 1, column: 1 },
            Cell { row: 1, column: 0 },
            Cell { row: 1, column: -1 },
        ];

        rotate_right_90(&mut cell_mask);

        assert_eq!(cell_mask, expected_rotation)
    }
}
