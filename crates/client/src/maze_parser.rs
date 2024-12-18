use shared::maze::{Maze, PositionType};
use shared::messages::Direction;
use shared::{
    maze::Cell,
    radar::{CellType, Passages, Radar},
};

use crate::data_structures::maze_graph::MazeGraph;

fn rotate_left_90(cells: &mut [Cell]) {
    for cell in cells.iter_mut() {
        let new_row = -cell.column;
        let new_column = cell.row;
        cell.row = new_row;
        cell.column = new_column;
    }
}

fn rotate_right_90(cells: &mut [Cell]) {
    for cell in cells.iter_mut() {
        let new_row = cell.column;
        let new_column = -cell.row;
        cell.row = new_row;
        cell.column = new_column;
    }
}

pub struct Player {
    pub direction: Direction,
    pub position: Cell,
}

impl Player {
    pub fn move_forward(&mut self) {
        self.position = match self.direction {
            Direction::Front => Cell { row: self.position.row, column: self.position.column - 1 },
            Direction::Right => Cell { row: self.position.row + 1, column: self.position.column },
            Direction::Back => Cell { row: self.position.row, column: self.position.column + 1 },
            Direction::Left => Cell { row: self.position.row - 1, column: self.position.column },
        }
    }

    pub fn turn_right(&mut self) {
        self.direction = match self.direction {
            Direction::Front => Direction::Right,
            Direction::Right => Direction::Back,
            Direction::Back => Direction::Left,
            Direction::Left => Direction::Front,
        };
    }

    pub fn turn_left(&mut self) {
        self.direction = match self.direction {
            Direction::Front => Direction::Left,
            Direction::Right => Direction::Front,
            Direction::Back => Direction::Right,
            Direction::Left => Direction::Back,
        };
    }

    pub fn turn_back(&mut self) {
        self.direction = match self.direction {
            Direction::Front => Direction::Back,
            Direction::Right => Direction::Left,
            Direction::Back => Direction::Front,
            Direction::Left => Direction::Right,
        };
    }

    pub fn get_next_direction(&mut self, target: &Cell) -> Direction {
        let diff = Cell {
            row: target.row - self.position.row,
            column: target.column - self.position.column,
        };

        let cell_mask: Vec<Cell> = get_direction_mask(self);

        if cell_mask[3] == diff {
            return Direction::Left;
        };
        if cell_mask[5] == diff {
            return Direction::Left;
        };
        if cell_mask[7] == diff {
            return Direction::Left;
        };
        Direction::Front
    }
}

pub fn maze_to_graph(radar_view: &Radar, player: &Player, maze_graph: &mut MazeGraph) {
    let directions_mask = get_direction_mask(player);

    for cell_id in 0..radar_view.cells.len() {
        if radar_view.cells[cell_id] == CellType::INVALID {
            continue;
        }

        let cell_pos = player.position + directions_mask[cell_id];

        if !maze_graph.contains(&cell_pos) {
            maze_graph.add(cell_pos, radar_view.cells[cell_id].clone());
        }

        let mut neigbors_to_add: Vec<Cell> = Vec::new();

        if is_top_cell_accessible(cell_id, &radar_view.horizontal) {
            let top_cell_id = cell_id - 3;
            let top_cell = player.position + directions_mask[top_cell_id];

            if !maze_graph.contains(&top_cell) {
                maze_graph.add(top_cell, radar_view.cells[top_cell_id].clone());
            }

            neigbors_to_add.push(top_cell);
        }

        if is_bottom_cell_accessible(cell_id, &radar_view.horizontal) {
            let bottom_cell_id = cell_id + 3;
            let bottom_cell = player.position + directions_mask[bottom_cell_id];

            if !maze_graph.contains(&bottom_cell) {
                maze_graph.add(bottom_cell, radar_view.cells[bottom_cell_id].clone());
            }

            neigbors_to_add.push(bottom_cell);
        }

        if is_left_cell_accessible(cell_id, &radar_view.vertical) {
            let left_cell_id = cell_id - 1;
            let left_cell = player.position + directions_mask[left_cell_id];

            if !maze_graph.contains(&left_cell) {
                maze_graph.add(left_cell, radar_view.cells[left_cell_id].clone());
            }

            neigbors_to_add.push(left_cell);
        }

        if is_right_cell_accessible(cell_id, &radar_view.vertical) {
            let right_cell_id = cell_id + 1;
            let right_cell = player.position + directions_mask[right_cell_id];

            if !maze_graph.contains(&right_cell) {
                maze_graph.add(right_cell, radar_view.cells[right_cell_id].clone());
            }

            neigbors_to_add.push(right_cell);
        }

        for neighbor in neigbors_to_add {
            maze_graph.add_neighbor(&cell_pos, &neighbor);
            maze_graph.add_neighbor(&neighbor, &cell_pos);
        }
    }
}

fn is_right_cell_accessible(cell_id: usize, vertical: &[Passages]) -> bool {
    cell_id % 3 != 2 && vertical[cell_id + cell_id / 3 + 1] == Passages::OPEN
}

fn is_left_cell_accessible(cell_id: usize, vertical: &[Passages]) -> bool {
    cell_id % 3 != 0 && vertical[cell_id + cell_id / 3] == Passages::OPEN
}

fn is_bottom_cell_accessible(cell_id: usize, horizontal: &[Passages]) -> bool {
    cell_id < 6 && horizontal[cell_id + 3] == Passages::OPEN
}

fn is_top_cell_accessible(cell_id: usize, horizontal: &[Passages]) -> bool {
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

    if player.direction == Direction::Right {
        rotate_left_90(&mut cell_mask);
    }
    if player.direction == Direction::Left {
        rotate_right_90(&mut cell_mask);
    }
    if player.direction == Direction::Back {
        rotate_right_90(&mut cell_mask);
        rotate_right_90(&mut cell_mask);
    }
    cell_mask
}

pub fn maze_parser(input: &str) -> Maze {
    if input.is_empty() {
        return Maze::new(Vec::new(), Cell { row: 0, column: 0 }, Cell { row: 0, column: 0 });
    }

    let lines: Vec<&str> =
        input.lines().skip_while(|line| line.chars().all(char::is_whitespace)).collect();
    let (height, width) = (lines.len(), lines[0].len());

    let map = vec![vec![0u16; width]; height];
    let entry = Cell { row: 0, column: 0 };
    let exit = Cell { row: 0, column: 0 };
    let mut maze = Maze::new(map, entry, exit);

    for (row, line) in lines.iter().enumerate() {
        for (col, char) in line.chars().enumerate() {
            match char {
                ' ' => {
                    maze.map[row][col] = PositionType::SPACE;
                }
                '2' => {
                    maze.map[row][col] = PositionType::ENTRY;
                    maze.entry.row = row as i16;
                    maze.entry.column = col as i16;
                }
                '3' => {
                    maze.map[row][col] = PositionType::EXIT;
                    maze.exit.row = row as i16;
                    maze.exit.column = col as i16;
                }
                _ => {
                    maze.map[row][col] = PositionType::WALL;
                }
            }
        }
    }

    maze
}

#[cfg(test)]
mod tests {
    use shared::radar;

    use super::*;

    #[test]
    fn testo() {
        let decoded = radar::decode_base64("Hjeikcyc/W8a8pa");
        let data = radar::extract_data(&decoded);

        let mut p = Player { position: Cell { row: 0, column: 0 }, direction: Direction::Front };
        let mut m = MazeGraph::new();
        maze_to_graph(&data, &p, &mut m);

        println!("{:?}", m);

        p.direction = Direction::Right;
        p.position = p.position + Cell { row: 1, column: 0 };

        let decoded = radar::decode_base64("kOuczzGa//apaaa");
        let data = radar::extract_data(&decoded);
        maze_to_graph(&data, &p, &mut m);

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

    #[test]
    fn test_player_movement() {
        let mut player =
            Player { position: Cell { row: 5, column: 5 }, direction: Direction::Front };

        player.move_forward();
        assert_eq!(player.position, Cell { row: 5, column: 4 });

        player.turn_right();
        assert_eq!(player.direction, Direction::Right);
        player.move_forward();
        assert_eq!(player.position, Cell { row: 6, column: 4 });

        player.turn_left();
        assert_eq!(player.direction, Direction::Front);
        player.move_forward();
        assert_eq!(player.position, Cell { row: 6, column: 3 });

        player.turn_back();
        assert_eq!(player.direction, Direction::Back);
        player.move_forward();
        assert_eq!(player.position, Cell { row: 6, column: 4 });
    }
}
