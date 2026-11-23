use crate::maze::{Cell, Maze, PositionType};
use rand::{RngExt, SeedableRng, rngs::StdRng, seq::IndexedMutRandom};

/// How to carve a maze.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratorAlgorithm {
    Sidewinder,
    RecursiveBacktracker,
}

/// Generates a `width` by `height` maze; the same seed always gives the same maze.
///
/// ```
/// use shared::maze_generator::{GeneratorAlgorithm, generate};
///
/// let maze = generate(GeneratorAlgorithm::Sidewinder, 10, 10, false, 42);
/// ```
pub fn generate(
    algorithm: GeneratorAlgorithm,
    width: usize,
    height: usize,
    print: bool,
    seed: u64,
) -> Maze {
    match algorithm {
        GeneratorAlgorithm::Sidewinder => sidewinder(width, height, print, seed),
        GeneratorAlgorithm::RecursiveBacktracker => recursive_backtracker(width, height, seed),
    }
}

/// Sidewinder: an open top row that every cell can reach going up.
fn sidewinder(width: usize, height: usize, print: bool, seed: u64) -> Maze {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut maze = Maze::new(
        vec![vec![PositionType::WALL; width * 2 + 1]; height * 2 + 1],
        Cell { row: 0, column: 0 },
        Cell { row: 0, column: 0 },
    );

    for row in 0..height {
        let mut current = Vec::new();

        for col in 0..width {
            if print {
                maze.print_visited(&vec![vec![-1; width * 2 + 1]; height * 2 + 1]);
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            let cell_row = row * 2 + 1;
            let cell_col = col * 2 + 1;

            maze.map[cell_row][cell_col] = PositionType::SPACE;

            current.push(Cell { row: cell_row as i16, column: cell_col as i16 });

            if row > 0 && (rng.random_bool(0.5) || col == width - 1) {
                let random_cell = match current.choose_mut(&mut rng) {
                    Some(cell) => *cell,
                    None => Cell { row: 0, column: 0 },
                };

                maze.map[random_cell.row as usize - 1][random_cell.column as usize] =
                    PositionType::SPACE;

                current.clear();
            } else if col < width - 1 {
                maze.map[cell_row][cell_col + 1] = PositionType::SPACE;
            }
        }
    }
    (maze.entry, maze.exit) = generate_random_entry_exit(width, height, seed);
    maze.map[maze.entry.row as usize][maze.entry.column as usize] = PositionType::ENTRY;
    maze.map[maze.exit.row as usize][maze.exit.column as usize] = PositionType::EXIT;
    maze
}

/// Picks an entry on the West side and an exit on the East side, in grid indices.
fn generate_random_entry_exit(width: usize, height: usize, seed: u64) -> (Cell, Cell) {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut cell_row = || rng.random_range(0..height as i16) * 2 + 1;
    let entry = Cell { row: cell_row(), column: 1 };
    let exit = Cell { row: cell_row(), column: width as i16 * 2 - 1 };

    (entry, exit)
}

/// Generates a maze by walking randomly and backing up at dead ends (recursive
/// backtracking), the reference server's default style: long winding corridors and deep
/// dead ends, unlike Sidewinder's open top row.
///
/// The result is a grid of `(2 * height + 1) x (2 * width + 1)`, cells at odd indices,
/// with exactly one path between any two cells.
fn recursive_backtracker(width: usize, height: usize, seed: u64) -> Maze {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut map = vec![vec![PositionType::WALL; width * 2 + 1]; height * 2 + 1];
    let mut stack: Vec<(usize, usize)> = vec![(0, 0)];
    map[1][1] = PositionType::SPACE;

    while let Some(&(x, y)) = stack.last() {
        let unvisited: Vec<(usize, usize)> = [(0, -1), (1, 0), (0, 1), (-1, 0)]
            .iter()
            .filter_map(|&(dx, dy)| Some((x.checked_add_signed(dx)?, y.checked_add_signed(dy)?)))
            .filter(|&(nx, ny)| {
                nx < width && ny < height && map[ny * 2 + 1][nx * 2 + 1] == PositionType::WALL
            })
            .collect();
        let Some(&(nx, ny)) = unvisited.get(rng.random_range(0..unvisited.len().max(1))) else {
            stack.pop();
            continue;
        };
        map[y + ny + 1][x + nx + 1] = PositionType::SPACE;
        map[ny * 2 + 1][nx * 2 + 1] = PositionType::SPACE;
        stack.push((nx, ny));
    }

    let (entry, exit) = generate_random_entry_exit(width, height, seed);
    Maze::new(map, entry, exit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sizes_the_maze() {
        let maze = generate(GeneratorAlgorithm::Sidewinder, 5, 10, false, 0);

        assert_eq!(maze.row_len, 10 * 2 + 1);
        assert_eq!(maze.col_len, 5 * 2 + 1);
        assert!(!maze.is_cell_out_of_bound(&maze.entry));
        assert!(!maze.is_cell_out_of_bound(&maze.exit));
    }

    #[test]
    fn test_sidewinder() {
        let seed = 42;
        let maze = sidewinder(10, 10, false, seed);
        assert_eq!(maze.map.len(), 21);

        let maze2 = sidewinder(10, 10, false, seed);
        assert_eq!(maze.map, maze2.map);
    }

    /// Counts open cells reachable from the first one, and open passages between cells.
    fn cells_and_passages(maze: &Maze) -> (usize, usize) {
        let (rows, columns) = (maze.row_len, maze.col_len);
        let mut seen = vec![vec![false; columns]; rows];
        let mut stack = vec![(1, 1)];
        seen[1][1] = true;
        let mut cells = 0;
        while let Some((r, c)) = stack.pop() {
            cells += 1;
            for (dr, dc) in [(0, 2), (2, 0), (0, -2), (-2, 0)] {
                let (nr, nc) = (r as i32 + dr, c as i32 + dc);
                if nr < 1 || nc < 1 || nr >= rows as i32 || nc >= columns as i32 {
                    continue;
                }
                let wall = maze.map[(r as i32 + dr / 2) as usize][(c as i32 + dc / 2) as usize];
                if wall != PositionType::WALL && !seen[nr as usize][nc as usize] {
                    seen[nr as usize][nc as usize] = true;
                    stack.push((nr as usize, nc as usize));
                }
            }
        }
        let passages = (0..rows)
            .flat_map(|r| (0..columns).map(move |c| (r, c)))
            .filter(|&(r, c)| (r + c) % 2 == 1 && maze.map[r][c] != PositionType::WALL)
            .count();
        (cells, passages)
    }

    #[test]
    fn test_recursive_backtracker_joins_every_cell_by_exactly_one_path() {
        for seed in 0..20 {
            let maze = recursive_backtracker(12, 7, seed);

            // A spanning tree: all cells reached, one passage fewer than cells.
            assert_eq!(cells_and_passages(&maze), (84, 83), "seed {seed}");
        }
        assert_eq!(recursive_backtracker(12, 7, 3).map, recursive_backtracker(12, 7, 3).map);
    }

    #[test]
    fn test_entry_and_exit_are_cells_on_the_west_and_east_sides() {
        for seed in 0..50 {
            let (entry, exit) = generate_random_entry_exit(10, 6, seed);
            for cell in [entry, exit] {
                // Cells sit at odd grid indices; even ones are wall slots.
                assert_eq!(cell.row % 2, 1, "seed {seed}: {cell:?} is not on a cell row");
                assert!((1..12).contains(&cell.row));
            }
            assert_eq!(entry.column, 1);
            assert_eq!(exit.column, 19);
        }
    }
}
