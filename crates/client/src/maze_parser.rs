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

pub fn maze_to_graph(
    (horizontal, vertical, cells): (Vec<Passages>, Vec<Passages>, Vec<CellType>),
) -> MazeGraph {
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

        let cell_pos = player_pos + cell_mask[cell_id];

        if !maze_graph.contains(&cell_pos) {
            maze_graph.add(cell_pos, cells[cell_id].clone());
        }

        // top cell
        if cell_id > 2 && horizontal[cell_id] == Passages::OPEN {
            let top_cell = Cell { row: cell_pos.row, column: cell_pos.column - 1 };

            if !maze_graph.contains(&top_cell) {
                maze_graph.add(top_cell, cells[cell_id - 3].clone());
            }

            maze_graph.add_neighbor(&top_cell, &cell_pos);
            maze_graph.add_neighbor(&cell_pos, &top_cell);
        }

        // bottom cell
        if cell_id < 6 && horizontal[cell_id + 3] == Passages::OPEN {
            let bottom_cell = Cell { row: cell_pos.row, column: cell_pos.column + 1 };

            if !maze_graph.contains(&bottom_cell) {
                maze_graph.add(bottom_cell, cells[cell_id + 3].clone());
            }

            maze_graph.add_neighbor(&bottom_cell, &cell_pos);
            maze_graph.add_neighbor(&cell_pos, &bottom_cell);
        }

        // left cell
        if cell_id % 3 != 0 && vertical[cell_id + cell_id / 3] == Passages::OPEN {
            let left_cell = Cell { row: cell_pos.row - 1, column: cell_pos.column };

            if !maze_graph.contains(&left_cell) {
                maze_graph.add(left_cell, cells[cell_id - 1].clone());
            }

            maze_graph.add_neighbor(&left_cell, &cell_pos);
            maze_graph.add_neighbor(&cell_pos, &left_cell);
        }

        // right cell
        if cell_id % 3 != 2 && vertical[cell_id + cell_id / 3 + 1] == Passages::OPEN {
            let right_cell = Cell { row: cell_pos.row + 1, column: cell_pos.column };

            if !maze_graph.contains(&right_cell) {
                maze_graph.add(right_cell, cells[cell_id + 1].clone());
            }

            maze_graph.add_neighbor(&right_cell, &cell_pos);
            maze_graph.add_neighbor(&cell_pos, &right_cell);
        }
    }
    maze_graph
}

#[cfg(test)]
mod tests {
    use shared::radar;

    use super::*;

    #[test]
    fn testo() {
        let decoded = radar::decode_base64("giLbMjIad/apapa");
        let data = radar::extract_data(&decoded);

        let m = maze_to_graph(data);
        println!("{:?}", m);
    }

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
