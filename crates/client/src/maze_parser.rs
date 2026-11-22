use shared::maze::{Maze, PositionType};
use shared::messages::{Action, Direction};
use shared::{
    maze::Cell,
    radar::{CellType, Passages, Radar},
};

use crate::data_structures::maze_graph::MazeGraph;
use crate::heading::Heading;

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub heading: Heading,
    pub position: Cell,
    /// Last compass hint, as clockwise degrees from North.
    pub exit_bearing: Option<f32>,
}

impl Player {
    pub fn new() -> Self {
        Self { position: Cell { row: 0, column: 0 }, heading: Heading::North, exit_bearing: None }
    }

    /// Records a compass hint: clockwise degrees from the heading to the exit.
    pub fn record_compass(&mut self, angle: f32) {
        self.exit_bearing = Some((self.heading.degrees() + angle).rem_euclid(360.0));
    }

    pub fn move_forward(&mut self) {
        self.position = self.position + self.heading.delta();
    }

    pub fn turn(&mut self, direction: Direction) {
        self.heading = self.heading.turn(direction);
    }

    /// The move toward a neighboring cell, or `Front` if `target` is not one.
    pub fn get_next_direction(&self, target: &Cell) -> Direction {
        let delta = Cell {
            row: target.row - self.position.row,
            column: target.column - self.position.column,
        };
        Heading::from_delta(delta).map_or(Direction::Front, |to| self.heading.direction_to(to))
    }

    pub fn step_toward(&mut self, target: &Cell) -> Action {
        let direction = self.get_next_direction(target);
        self.turn(direction);
        self.move_forward();
        Action::MoveTo(direction)
    }

    pub fn get_back_position(&self) -> Cell {
        self.position + self.heading.turn(Direction::Back).delta()
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

pub fn maze_to_graph(radar_view: &Radar, player: &Player, maze_graph: &mut MazeGraph) {
    let directions_mask = get_direction_mask(player);

    for cell_id in 0..radar_view.cells.len() {
        if radar_view.cells[cell_id] == CellType::INVALID {
            continue;
        }

        let cell_pos = player.position + directions_mask[cell_id];

        if !maze_graph.contains(&cell_pos) {
            maze_graph.add(cell_pos, radar_view.cells[cell_id].clone());
        }

        let walls = get_cell_walls(cell_id, &radar_view.horizontal, &radar_view.vertical);
        maze_graph.update_walls(cell_pos, walls as usize);

        let mut neigbors_to_add: Vec<Cell> = Vec::new();

        if is_top_cell_accessible(cell_id, &radar_view.horizontal) {
            let top_cell_id = cell_id - 3;
            let top_cell = player.position + directions_mask[top_cell_id];

            if !maze_graph.contains(&top_cell) {
                maze_graph.add(top_cell, radar_view.cells[top_cell_id].clone());
            }

            neigbors_to_add.push(top_cell);
        }

        if is_bottom_cell_accessible(cell_id, &radar_view.horizontal) {
            let bottom_cell_id = cell_id + 3;
            let bottom_cell = player.position + directions_mask[bottom_cell_id];

            if !maze_graph.contains(&bottom_cell) && bottom_cell_id < radar_view.cells.len() {
                maze_graph.add(bottom_cell, radar_view.cells[bottom_cell_id].clone());
            }

            neigbors_to_add.push(bottom_cell);
        }

        if is_left_cell_accessible(cell_id, &radar_view.vertical) {
            let left_cell_id = cell_id - 1;
            let left_cell = player.position + directions_mask[left_cell_id];

            if !maze_graph.contains(&left_cell) {
                maze_graph.add(left_cell, radar_view.cells[left_cell_id].clone());
            }

            neigbors_to_add.push(left_cell);
        }

        if is_right_cell_accessible(cell_id, &radar_view.vertical) {
            let right_cell_id = cell_id + 1;
            let right_cell = player.position + directions_mask[right_cell_id];

            if !maze_graph.contains(&right_cell) && right_cell_id < radar_view.cells.len() {
                maze_graph.add(right_cell, radar_view.cells[right_cell_id].clone());
            }

            neigbors_to_add.push(right_cell);
        }

        for neighbor in neigbors_to_add {
            maze_graph.add_neighbor(&cell_pos, &neighbor);
            maze_graph.add_neighbor(&neighbor, &cell_pos);
        }
    }
}

fn is_right_cell_accessible(cell_id: usize, vertical: &[Passages]) -> bool {
    cell_id % 3 != 2 && vertical[cell_id + cell_id / 3 + 1] == Passages::OPEN
}

fn is_left_cell_accessible(cell_id: usize, vertical: &[Passages]) -> bool {
    !cell_id.is_multiple_of(3) && vertical[cell_id + cell_id / 3] == Passages::OPEN
}

fn is_bottom_cell_accessible(cell_id: usize, horizontal: &[Passages]) -> bool {
    cell_id < 6 && horizontal[cell_id + 3] == Passages::OPEN
}

fn is_top_cell_accessible(cell_id: usize, horizontal: &[Passages]) -> bool {
    cell_id > 2 && horizontal[cell_id] == Passages::OPEN
}

fn get_cell_walls(cell_id: usize, horizontal: &[Passages], vertical: &[Passages]) -> u8 {
    let mut walls = 0;

    // Top wall
    if horizontal[cell_id] == Passages::WALL {
        walls += 1;
    }

    // Right wall
    if vertical[cell_id + cell_id / 3 + 1] == Passages::WALL {
        walls += 1;
    }

    // Left wall
    if vertical[cell_id + cell_id / 3] == Passages::WALL {
        walls += 1;
    }

    // Bottom wall
    if horizontal[cell_id + 3] == Passages::WALL {
        walls += 1;
    }

    walls
}

/// Map offsets of the radar's 3x3 cells, row by row, as seen by the player.
fn get_direction_mask(player: &Player) -> Vec<Cell> {
    (-1..=1)
        .flat_map(|column| (-1..=1).map(move |row| Cell { row, column }))
        .map(|offset| player.heading.rotate(offset))
        .collect()
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use shared::radar;

    #[test]
    fn test_maze_to_graph() {
        let decoded = radar::decode_base64("Hjeikcyc/W8a8pa");
        let data = radar::extract_data(&decoded);

        let mut p = Player {
            position: Cell { row: 0, column: 0 },
            heading: Heading::North,
            exit_bearing: None,
        };
        let mut m = MazeGraph::new();
        maze_to_graph(&data, &p, &mut m);

        p.heading = Heading::East;
        p.position = p.position + Cell { row: 1, column: 0 };

        let decoded = radar::decode_base64("kOuczzGa//apaaa");
        let data = radar::extract_data(&decoded);
        maze_to_graph(&data, &p, &mut m);
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
    }

    #[test]
    fn test_get_cell_walls() {
        let horizontal = vec![
            Passages::WALL,
            Passages::OPEN,
            Passages::WALL,
            Passages::OPEN,
            Passages::WALL,
            Passages::OPEN,
            Passages::WALL,
            Passages::OPEN,
            Passages::WALL,
        ];
        let vertical = vec![
            Passages::WALL,
            Passages::OPEN,
            Passages::WALL,
            Passages::WALL,
            Passages::WALL,
            Passages::OPEN,
        ];

        let walls = get_cell_walls(0, &horizontal, &vertical);
        assert_eq!(walls, 2);

        let walls = get_cell_walls(1, &horizontal, &vertical);
        assert_eq!(walls, 2);
    }

    #[test]
    fn test_player_get_next_direction() {
        let player = Player::new();

        let target = Cell { row: -1, column: 0 }; // Left
        assert_eq!(player.get_next_direction(&target), Direction::Left);

        let target = Cell { row: 1, column: 0 }; // Right
        assert_eq!(player.get_next_direction(&target), Direction::Right);

        let target = Cell { row: 0, column: 1 }; // Back
        assert_eq!(player.get_next_direction(&target), Direction::Back);

        let target = Cell { row: 0, column: -1 }; // Front
        assert_eq!(player.get_next_direction(&target), Direction::Front);
    }

    #[test]
    fn test_player_get_back_position() {
        let mut player = Player::new();

        assert_eq!(player.get_back_position(), Cell { row: 0, column: 1 });

        player.turn(Direction::Right);
        assert_eq!(player.get_back_position(), Cell { row: -1, column: 0 });

        player.turn(Direction::Right);
        assert_eq!(player.get_back_position(), Cell { row: 0, column: -1 });

        player.turn(Direction::Right);
        assert_eq!(player.get_back_position(), Cell { row: 1, column: 0 });
    }

    #[test]
    fn test_get_direction_mask() {
        let player = Player::new();
        let mask = get_direction_mask(&player);
        assert_eq!(mask.len(), 9);
        assert_eq!(mask[4], Cell { row: 0, column: 0 });

        let mut player_right = Player::new();
        player_right.heading = Heading::East;
        let mask_right = get_direction_mask(&player_right);
        assert_ne!(mask, mask_right);

        let mut player_back = Player::new();
        player_back.heading = Heading::South;
        let mask_back = get_direction_mask(&player_back);
        assert_ne!(mask, mask_back);
    }

    #[test]
    fn test_mask_facing_east() {
        let mut player = Player::new();
        player.heading = Heading::East;
        let expected_rotation = vec![
            Cell { row: 1, column: -1 },
            Cell { row: 1, column: 0 },
            Cell { row: 1, column: 1 },
            Cell { row: 0, column: -1 },
            Cell { row: 0, column: 0 },
            Cell { row: 0, column: 1 },
            Cell { row: -1, column: -1 },
            Cell { row: -1, column: 0 },
            Cell { row: -1, column: 1 },
        ];

        assert_eq!(get_direction_mask(&player), expected_rotation);
    }

    #[test]
    fn test_mask_facing_west() {
        let mut player = Player::new();
        player.heading = Heading::West;
        let expected_rotation = vec![
            Cell { row: -1, column: 1 },
            Cell { row: -1, column: 0 },
            Cell { row: -1, column: -1 },
            Cell { row: 0, column: 1 },
            Cell { row: 0, column: 0 },
            Cell { row: 0, column: -1 },
            Cell { row: 1, column: 1 },
            Cell { row: 1, column: 0 },
            Cell { row: 1, column: -1 },
        ];

        assert_eq!(get_direction_mask(&player), expected_rotation);
    }

    #[test]
    fn test_player_movement() {
        let mut player = Player {
            position: Cell { row: 5, column: 5 },
            heading: Heading::North,
            exit_bearing: None,
        };

        player.move_forward();
        assert_eq!(player.position, Cell { row: 5, column: 4 });

        player.turn(Direction::Right);
        assert_eq!(player.heading, Heading::East);
        player.move_forward();
        assert_eq!(player.position, Cell { row: 6, column: 4 });

        player.turn(Direction::Left);
        assert_eq!(player.heading, Heading::North);
        player.move_forward();
        assert_eq!(player.position, Cell { row: 6, column: 3 });

        player.turn(Direction::Back);
        assert_eq!(player.heading, Heading::South);
        player.move_forward();
        assert_eq!(player.position, Cell { row: 6, column: 4 });
    }
}
