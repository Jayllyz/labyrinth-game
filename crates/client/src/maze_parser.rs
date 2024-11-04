use shared::maze::{Cell, Maze, PositionType};

pub fn maze_parser(input: &str) -> Maze {
    if input.is_empty() {
        return Maze::new(Vec::new(), Cell { row: 0, column: 0 }, Cell { row: 0, column: 0 });
    }

    let mut untrimmed_lines: Vec<&str> = input.lines().collect();
    untrimmed_lines =
        untrimmed_lines.into_iter().skip_while(|line| line.trim().is_empty()).collect();
    untrimmed_lines = untrimmed_lines.into_iter().map(|line| line.trim_start()).collect();

    let lines: Vec<&str> = untrimmed_lines;
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
    use super::*;

    #[test]
    fn test_maze_parser() {
        let input = "###\n# #\n###";
        let expected = vec![vec![1, 1, 1], vec![1, 0, 1], vec![1, 1, 1]];
        assert_eq!(maze_parser(input).map, expected);

        let input = "#### \n#  ##\n#### ";
        let expected = vec![vec![1, 1, 1, 1, 0], vec![1, 0, 0, 1, 1], vec![1, 1, 1, 1, 0]];
        assert_eq!(maze_parser(input).map, expected);

        let input = "#  # \n#  # \n#  # ";
        let expected = vec![vec![1, 0, 0, 1, 0], vec![1, 0, 0, 1, 0], vec![1, 0, 0, 1, 0]];
        assert_eq!(maze_parser(input).map, expected);

        /*
            # # # # # # # # # # #
            #   #   2   #       #
            # 3 # # #   #   # # #
            #   #       #       #
            #   # # #   #   # # #
            #   #               #
            #   #   # # #   # # #
            #           #       #
            # # #   #   #   #   #
            #       #       #   #
            # # # # # # # # # # #
        */
        let input = "###########\n# # 2 #   #\n#3### # ###\n# #   #   #\n# ### # ###\n# #       #\n# # ### ###\n#     #   #\n### # # # #\n#   #   # #\n###########";
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

        // Test with empty input
        assert_eq!(maze_parser("").map, Vec::<Vec<u16>>::new());
    }
}
