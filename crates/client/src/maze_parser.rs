use crate::maze::{Maze, Point};

pub fn maze_parser(input: &str) -> Maze {
    if input.is_empty() {
        return Maze::new(Vec::new(), Point { row: 0, column: 0 }, Point { row: 0, column: 0 });
    }

    let lines: Vec<&str> = input.lines().collect();
    let (height, width) = (lines.len(), lines[0].len());

    let map = vec![vec![0u8; width]; height];
    let entry = Point { row: 0, column: 0 };
    let exit = Point { row: 0, column: 0 };
    let mut maze = Maze::new(map, entry, exit);

    const WALL: u8 = 1;
    const SPACE: u8 = 0;
    const ENTRY: u8 = 2;
    const EXIT: u8 = 3;

    for (y, line) in lines.iter().enumerate() {
        for (x, char) in line.chars().enumerate() {
            match char {
                ' ' => {
                    maze.map[y][x] = SPACE;
                }
                '2' => {
                    maze.map[y][x] = ENTRY;
                    maze.entry.row = y as i8;
                    maze.entry.column = x as i8;
                }
                '3' => {
                    maze.map[y][x] = EXIT;
                    maze.exit.row = y as i8;
                    maze.exit.column = x as i8;
                }
                _ => {
                    maze.map[y][x] = WALL;
                }
            }
        }
    }

    maze
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maze_parser() {
        let input = "###\n# #\n###";
        let expected = vec![vec![1, 1, 1], vec![1, 0, 1], vec![1, 1, 1]];
        assert_eq!(maze_parser(input).map, expected);

        let input = "#### \n#  #|\n#### ";
        let expected = vec![vec![1, 1, 1, 1, 0], vec![1, 0, 0, 1, 1], vec![1, 1, 1, 1, 0]];
        assert_eq!(maze_parser(input).map, expected);

        let input = "#  # \n#  # \n#  # ";
        let expected = vec![vec![1, 0, 0, 1, 0], vec![1, 0, 0, 1, 0], vec![1, 0, 0, 1, 0]];
        assert_eq!(maze_parser(input).map, expected);

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
        assert_eq!(maze_parser(input).map, expected);

        assert_eq!(maze_parser("").map, Vec::<Vec<u8>>::new());
    }
}
