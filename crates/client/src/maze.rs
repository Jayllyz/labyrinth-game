use shared::utils::ColorsAnsi;
use std::ops::Add;

pub struct Maze {
    pub map: Vec<Vec<u16>>,
    pub row_len: usize,
    pub col_len: usize,
    pub entry: Cell,
    pub exit: Cell,
}

impl Maze {
    pub fn new(map: Vec<Vec<u16>>, entry: Cell, exit: Cell) -> Self {
        let row_len = map.len();
        let col_len = if map.is_empty() { 0 } else { map[0].len() };
        Self { map, row_len, col_len, entry, exit }
    }

    pub fn print(&self, visited_points: &[Vec<bool>]) {
        for (row_idx, row) in self.map.iter().enumerate() {
            for (col_idx, &cell) in row.iter().enumerate() {
                let point = Cell { row: row_idx as i16, column: col_idx as i16 };

                if point == self.entry {
                    print!("3 ");
                } else if point == self.exit {
                    print!("2 ");
                } else if visited_points[row_idx][col_idx] {
                    print!("{}X{} ", ColorsAnsi::RED, ColorsAnsi::RESET);
                } else {
                    print!(
                        "{} ",
                        match cell {
                            1 => '#',
                            0 => ' ',
                            _ => '?',
                        }
                    );
                }
            }
            println!();
        }
        println!();
    }

    pub fn print_path(&self, path: &[Cell]) {
        for (row_idx, row) in self.map.iter().enumerate() {
            for (col_idx, &cell) in row.iter().enumerate() {
                let point = Cell { row: row_idx as i16, column: col_idx as i16 };

                if point == self.entry {
                    print!("3 ");
                } else if point == self.exit {
                    print!("2 ");
                } else if path.contains(&point) {
                    print!("{}X{} ", ColorsAnsi::RED, ColorsAnsi::RESET);
                } else {
                    print!(
                        "{} ",
                        match cell {
                            1 => '#',
                            0 => ' ',
                            _ => '?',
                        }
                    );
                }
            }
            println!();
        }
        println!();
    }

    pub fn is_point_out_of_bound(&self, point: &Cell) -> bool {
        point.row < 0
            || point.column < 0
            || point.row >= (self.row_len as i16)
            || point.column >= (self.col_len as i16)
    }

    pub fn is_point_walkable(&self, point: &Cell, visited_points: &[Vec<bool>]) -> bool {
        !visited_points[point.row as usize][point.column as usize]
            && self.map[point.row as usize][point.column as usize] != PositionType::WALL
    }
}

#[derive(Clone, PartialEq, Copy, Debug)]
pub struct Cell {
    pub row: i16,
    pub column: i16,
}

impl Add for Cell {
    type Output = Cell;

    fn add(self, other: Cell) -> Cell {
        Cell { row: self.row + other.row, column: self.column + other.column }
    }
}

pub struct PositionType;

impl PositionType {
    pub const WALL: u16 = 1;
    pub const SPACE: u16 = 0;
    pub const ENTRY: u16 = 2;
    pub const EXIT: u16 = 3;
}

pub struct Directions;

impl Directions {
    pub const NORTH: Cell = Cell { row: -1, column: 0 };
    pub const SOUTH: Cell = Cell { row: 1, column: 0 };
    pub const WEST: Cell = Cell { row: 0, column: -1 };
    pub const EAST: Cell = Cell { row: 0, column: 1 };
}
