use shared::utils::ColorsAnsi;
use std::ops::Add;

pub struct Maze {
    pub map: Vec<Vec<u8>>,
    pub row_len: usize,
    pub col_len: usize,
    pub entry: Point,
    pub exit: Point,
}

impl Maze {
    pub fn new(map: Vec<Vec<u8>>, entry: Point, exit: Point) -> Self {
        let row_len = map.len();
        let col_len = if map.is_empty() { 0 } else { map[0].len() };
        Self { map, row_len, col_len, entry, exit }
    }

    pub fn print(&self, visited_points: &[Vec<bool>]) {
        for (row_idx, row) in self.map.iter().enumerate() {
            for (col_idx, &cell) in row.iter().enumerate() {
                let point = Point { row: row_idx as i8, column: col_idx as i8 };

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
}

#[derive(Clone, PartialEq, Copy)]
pub struct Point {
    pub row: i8,
    pub column: i8,
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point { row: self.row + other.row, column: self.column + other.column }
    }
}
