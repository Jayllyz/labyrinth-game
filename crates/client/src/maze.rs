use std::{collections::VecDeque, ops::Add};

pub fn maze_parser(input: &str) -> Vec<Vec<u8>> {
    if input.is_empty() {
        return Vec::new();
    }

    let lines: Vec<&str> = input.lines().collect();
    let (height, width) = (lines.len(), lines[0].len());

    let mut maze = vec![vec![0u8; width]; height];

    const WALL: u8 = 1;
    const SPACE: u8 = 0;
    const ENTRY: u8 = 2;
    const EXIT: u8 = 3;

    for (y, line) in lines.iter().enumerate() {
        for (x, char) in line.chars().enumerate() {
            maze[y][x] = match char {
                ' ' => SPACE,
                '2' => ENTRY,
                '3' => EXIT,
                _ => WALL,
            };
        }
    }

    maze
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

fn print_maze(input: &[Vec<u8>], visited: &[Vec<bool>], entry: Point, exit: Point) {
    for (row_idx, row) in input.iter().enumerate() {
        for (col_idx, &cell) in row.iter().enumerate() {
            let point = Point { row: row_idx as i8, column: col_idx as i8 };

            if point == entry {
                print!("3 ");
            } else if point == exit {
                print!("2 ");
            } else if visited[row_idx][col_idx] {
                print!("\x1b[31mX\x1b[0m ");
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

pub fn bfs_maze(input: &[Vec<u8>], entry: Point, exit: Point) -> bool {
    let mut q: VecDeque<Point> = VecDeque::new();
    q.push_back(entry);

    let row_len = input.len();
    let col_len = input[0].len();

    let mut visited: Vec<Vec<bool>> = vec![vec![false; col_len]; row_len];
    visited[entry.row as usize][entry.column as usize] = true;

    let directions = [
        Point { row: -1, column: 0 },
        Point { row: 1, column: 0 },
        Point { row: 0, column: 1 },
        Point { row: 0, column: -1 },
    ];

    while !q.is_empty() {
        print_maze(input, &visited, entry, exit);
        let curr: Point = q.pop_front().unwrap();

        if curr == exit {
            return true;
        }

        for direction in directions.iter() {
            let neighbor: Point = curr + *direction;

            if neighbor.row < 0
                || neighbor.column < 0
                || neighbor.row >= (row_len as i8)
                || neighbor.column >= (col_len as i8)
            {
                continue;
            }

            if visited[neighbor.row as usize][neighbor.column as usize]
                || input[neighbor.row as usize][neighbor.column as usize] == 1
            {
                continue;
            }

            q.push_back(neighbor);
            visited[neighbor.row as usize][neighbor.column as usize] = true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maze_parser() {
        let input = "###\n# #\n###";
        let expected = vec![vec![1, 1, 1], vec![1, 0, 1], vec![1, 1, 1]];
        assert_eq!(maze_parser(input), expected);

        let input = "#### \n#  #|\n#### ";
        let expected = vec![vec![1, 1, 1, 1, 0], vec![1, 0, 0, 1, 1], vec![1, 1, 1, 1, 0]];
        assert_eq!(maze_parser(input), expected);

        let input = "#  # \n#  # \n#  # ";
        let expected = vec![vec![1, 0, 0, 1, 0], vec![1, 0, 0, 1, 0], vec![1, 0, 0, 1, 0]];
        assert_eq!(maze_parser(input), expected);

        /*
            # # # # # # # # # # #
            #   #   3   #       #
            # 2 # # #   #   # # #
            #   #       #       #
            #   # # #   #   # # #
            #   #               #
            #   #   # # #   # # #
            #           #       #
            # # #   #   #   #   #
            #       #       #   #
            # # # # # # # # # # #
        */

        let input = "###########\n# # 2 #   #\n#3### # ###\n# #   #   #\n# ### # ###\n# #       #\n# # ### ###\n#     #   #\n### # # # #\n#   #   # #\n###########\n";
        let expected = vec![
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
        assert_eq!(maze_parser(input), expected);

        assert_eq!(maze_parser(""), Vec::<Vec<u8>>::new());
    }

    #[test]
    fn test_bfs_exit_finder() {
        let input = vec![
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

        assert_eq!(
            bfs_maze(&input, Point { row: 1, column: 4 }, Point { row: 2, column: 1 }),
            true
        );

        let input = vec![
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

        assert_eq!(
            bfs_maze(&input, Point { row: 7, column: 10 }, Point { row: 1, column: 1 }),
            true
        );
    }
}
