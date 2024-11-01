use shared::utils::ColorsAnsi;
use std::ops::Add;

pub struct Maze {
    pub map: Vec<Vec<u8>>,
    pub row_len: usize,
    pub col_len: usize,
    pub entry: Position,
    pub exit: Position,
}

pub struct PositionType;

impl PositionType {
    pub const WALL: u8 = 1;
    pub const SPACE: u8 = 0;
    pub const ENTRY: u8 = 2;
    pub const EXIT: u8 = 3;
}

impl Maze {
    pub fn new(map: Vec<Vec<u8>>, entry: Position, exit: Position) -> Self {
        let row_len = map.len();
        let col_len = if map.is_empty() { 0 } else { map[0].len() };
        Self { map, row_len, col_len, entry, exit }
    }

    pub fn print(&self, visited_points: &[Vec<bool>]) {
        for (row_idx, row) in self.map.iter().enumerate() {
            for (col_idx, &cell) in row.iter().enumerate() {
                let point = Position { row: row_idx as i8, column: col_idx as i8 };

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

    pub fn is_point_out_of_bound(&self, point: &Position) -> bool {
        point.row < 0
            || point.column < 0
            || point.row >= (self.row_len as i8)
            || point.column >= (self.col_len as i8)
    }

    pub fn is_point_walkable(&self, point: &Position, visited_points: &[Vec<bool>]) -> bool {
        !visited_points[point.row as usize][point.column as usize]
            && self.map[point.row as usize][point.column as usize] != PositionType::WALL
    }
}

#[derive(Clone, PartialEq, Copy)]
pub struct Position {
    pub row: i8,
    pub column: i8,
}

impl Add for Position {
    type Output = Position;

    fn add(self, other: Position) -> Position {
        Position { row: self.row + other.row, column: self.column + other.column }
    }
}
