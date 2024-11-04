use crate::maze::{Cell, Maze, PositionType};
use rand::{seq::SliceRandom, thread_rng, Rng};

pub fn sidewinder(width: usize, height: usize, print: bool) -> Maze {
    let mut rng = thread_rng();
    let mut maze = Maze::new(
        vec![vec![PositionType::WALL; width * 2 + 1]; height * 2 + 1],
        Cell { row: 0, column: 0 },
        Cell { row: 0, column: 0 },
    );

    for row in 0..height {
        let mut current = Vec::new();

        for col in 0..width {
            if print {
                maze.print_visited(&vec![vec![false; width * 2 + 1]; height * 2 + 1]);
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            let cell_row = row * 2 + 1;
            let cell_col = col * 2 + 1;

            maze.map[cell_row][cell_col] = PositionType::SPACE;

            current.push(Cell { row: cell_row as i16, column: cell_col as i16 });

            if row > 0 && (rand::random() || col == width - 1) {
                let random_cell = current.choose(&mut rng).unwrap();

                maze.map[random_cell.row as usize - 1][random_cell.column as usize] =
                    PositionType::SPACE;

                current.clear();
            } else if col < width - 1 {
                maze.map[cell_row][cell_col + 1] = PositionType::SPACE;
            }
        }
    }
    (maze.entry, maze.exit) = generate_random_entry_exit(width, height);
    maze.map[maze.entry.row as usize][maze.entry.column as usize] = PositionType::SPACE;
    maze.map[maze.exit.row as usize][maze.exit.column as usize] = PositionType::SPACE;
    maze
}

fn generate_random_entry_exit(width: usize, height: usize) -> (Cell, Cell) {
    let mut rng = thread_rng();
    let entry = Cell { row: rng.gen_range(1..height as i16), column: 1 };
    let exit = Cell { row: rng.gen_range(1..height as i16), column: width as i16 - 1 };

    (entry, exit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidewinder() {
        let maze = sidewinder(10, 10, false);
        assert_eq!(maze.map.len(), 21);
    }

    #[test]
    fn test_generate_random_entry_exit() {
        let (entry, exit) = generate_random_entry_exit(10, 10);
        assert!(entry.row > 0 && entry.row < 10);
        assert_eq!(entry.column, 1);
        assert!(exit.row > 0 && exit.row < 10);
        assert_eq!(exit.column, 9);
    }
}
