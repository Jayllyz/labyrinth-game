use crate::maze_generator::sidewinder;
use crate::utils::ColorsAnsi;
use std::ops::Add;

pub struct Maze {
    pub map: Vec<Vec<u16>>,
    pub row_len: usize,
    pub col_len: usize,
    pub entry: Cell,
    pub exit: Cell,
}

pub enum GeneratorAlgorithm {
    Sidewinder,
}

impl Maze {
    pub fn new(map: Vec<Vec<u16>>, entry: Cell, exit: Cell) -> Self {
        let row_len = map.len();
        let col_len = if map.is_empty() { 0 } else { map[0].len() };
        Self { map, row_len, col_len, entry, exit }
    }

    pub fn generate(
        algorithm: GeneratorAlgorithm,
        width: usize,
        height: usize,
        print: bool,
    ) -> Self {
        match algorithm {
            GeneratorAlgorithm::Sidewinder => sidewinder(width, height, print),
        }
    }

    pub fn print_maze(maze: &Maze) {
        for row in &maze.map {
            for cell in row {
                print!(
                    "{} ",
                    match *cell {
                        PositionType::WALL => '#',
                        PositionType::SPACE => ' ',
                        _ => '?',
                    }
                );
            }
            println!();
        }
    }

    pub fn print_visited(&self, visited_points: &[Vec<i32>]) {
        let steps = *visited_points.iter().flatten().max().unwrap();
        let gradient: Vec<String> = (0..=steps).map(|step| {
            let gradient_value = 255 - (255 * step / steps);
            format!("\x1b[38;2;255;{};0m", gradient_value)
        }).collect();

        for (row_idx, row) in self.map.iter().enumerate() {
            for (col_idx, &cell) in row.iter().enumerate() {
                let point = Cell { row: row_idx as i16, column: col_idx as i16 };

                if point == self.entry {
                    print!("3 ");
                } else if point == self.exit {
                    print!("2 ");
                } else if visited_points[row_idx][col_idx] != -1 {
                    print!("{}X{} ", gradient[visited_points[row_idx][col_idx] as usize], ColorsAnsi::RESET);
                } else {
                    print!(
                        "{} ",
                        match cell {
                            PositionType::WALL => '#',
                            PositionType::SPACE => ' ',
                            _ => '?',
                        }
                    );
                }
            }
            println!();
        }
        println!();
    }

    pub fn print_visited_number(&self, visited_points: &[Vec<i32>]) {
        let steps = *visited_points.iter().flatten().max().unwrap();
        let gradient: Vec<String> = (0..=steps).map(|step| {
            let gradient_value = 255 - (255 * step / steps);
            format!("\x1b[38;2;255;{};0m", gradient_value)
        }).collect();
    
        for (row_idx, row) in self.map.iter().enumerate() {
            for (col_idx, &cell) in row.iter().enumerate() {
                let point = Cell { row: row_idx as i16, column: col_idx as i16 };
    
                if point == self.entry {
                    print!("  3  ");
                } else if point == self.exit {
                    print!("  2  ");
                } else if visited_points[row_idx][col_idx] != -1 {
                    let step_num = visited_points[row_idx][col_idx];
                    if step_num < 10 {
                        print!("{}  {}  {}", gradient[step_num as usize], step_num, ColorsAnsi::RESET);
                    } else if step_num < 100 {
                        print!("{}  {} {}", gradient[step_num as usize], step_num, ColorsAnsi::RESET);
                    } else {
                        print!("{} {} {}", gradient[step_num as usize], step_num, ColorsAnsi::RESET);
                    }
                } else {
                    match cell {
                        PositionType::WALL => print!("#####"),
                        PositionType::SPACE => print!("     "),
                        _ => print!("?  "),
                    }
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
                            PositionType::WALL => '#',
                            PositionType::SPACE => ' ',
                            _ => '?',
                        }
                    );
                }
            }
            println!();
        }
        println!();
    }

    pub fn is_cell_out_of_bound(&self, point: &Cell) -> bool {
        point.row < 0
            || point.column < 0
            || point.row >= (self.row_len as i16)
            || point.column >= (self.col_len as i16)
    }

    pub fn is_cell_walkable(&self, point: &Cell, visited_points: &[Vec<i32>]) -> bool {
        visited_points[point.row as usize][point.column as usize] == -1
            && self.map[point.row as usize][point.column as usize] != PositionType::WALL
    }
}

#[derive(Clone, PartialEq, Copy, Debug, Eq, Hash)]
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
