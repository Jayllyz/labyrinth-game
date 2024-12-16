use shared::{
    maze::{Cell, Maze, PositionType},
    radar::{CellType, Passages},
};

use crate::data_structures::maze_graph::{MazeCell, MazeGraph};

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

pub fn maze_to_graph((horizontal, vertical, cells): (Vec<Passages>, Vec<Passages>, Vec<CellType>)) {
    let mut maze_graph = MazeGraph::new();
    let player_pos = Cell { row: 0, column: 0 };

    let cell_mask = vec![
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

    for cell_id in 0..cells.len() {
        if cells[cell_id] == CellType::INVALID {
            continue;
        }

        if !maze_graph.contains(&player_pos) {
            maze_graph.add(player_pos, cells[cell_id]);
        }

        if cell_id > 2 && horizontal[cell_id] == Passages::OPEN {
            let top_cell = player_pos + cell_mask[cell_id];

            if !maze_graph.contains(&top_cell) {
                maze_graph.add(top_cell, cells[cell_id].clone());
            }

            maze_graph.add_neighbor(&top_cell, &player_pos);
            maze_graph.add_neighbor(&player_pos, &top_cell);
        }

        if cell_id < 6 && horizontal[cell_id + 3] == Passages::OPEN {
            // create bottom cell (y+1)
        }

        if cell_id % 3 != 0 && vertical[cell_id] == Passages::OPEN {
            // create left cell (x-1)
        }

        if cell_id % 3 != 2 && vertical[cell_id] == Passages::OPEN {
            // create right cell (x+1)
        }
    }
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
