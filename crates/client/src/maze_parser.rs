use shared::maze::{Maze, PositionType};
use shared::messages::{Action, Direction};
use shared::{
    maze::Cell,
    radar::{CellType, Passages, Radar},
};

use crate::data_structures::maze_graph::{MazeGraph, Side};
use shared::heading::Heading;

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

        for side in Heading::CLOCKWISE {
            let known = match passage(cell_id, side, &radar_view.horizontal, &radar_view.vertical) {
                Passages::OPEN => Side::Open,
                Passages::WALL => Side::Wall,
                Passages::UNDEFINED => continue,
            };
            let heading = Heading::from_delta(player.heading.rotate(side.delta()));
            if let Some(heading) = heading {
                maze_graph.set_side(cell_pos, heading, known);
            }
        }

        let mut neigbors_to_add: Vec<Cell> = Vec::new();
        for (side, neighbor_id) in radar_neighbors(cell_id) {
            if passage(cell_id, side, &radar_view.horizontal, &radar_view.vertical)
                != Passages::OPEN
            {
                continue;
            }
            let neighbor = player.position + directions_mask[neighbor_id];
            if !maze_graph.contains(&neighbor) {
                maze_graph.add(neighbor, radar_view.cells[neighbor_id].clone());
            }
            neigbors_to_add.push(neighbor);
        }

        for neighbor in neigbors_to_add {
            maze_graph.add_neighbor(&cell_pos, &neighbor);
            maze_graph.add_neighbor(&neighbor, &cell_pos);
        }
    }
}

/// The radar cells next to `cell_id`, with the side they are on, the radar's top being North.
fn radar_neighbors(cell_id: usize) -> impl Iterator<Item = (Heading, usize)> {
    let (row, column) = (cell_id / 3, cell_id % 3);
    [
        (Heading::North, (row > 0).then(|| cell_id - 3)),
        (Heading::East, (column < 2).then(|| cell_id + 1)),
        (Heading::South, (row < 2).then(|| cell_id + 3)),
        (Heading::West, (column > 0).then(|| cell_id - 1)),
    ]
    .into_iter()
    .filter_map(|(side, neighbor)| neighbor.map(|id| (side, id)))
}

/// The passage on one side of a radar cell, the radar's top being North.
fn passage(
    cell_id: usize,
    side: Heading,
    horizontal: &[Passages],
    vertical: &[Passages],
) -> Passages {
    let passage = match side {
        Heading::North => &horizontal[cell_id],
        Heading::South => &horizontal[cell_id + 3],
        Heading::West => &vertical[cell_id + cell_id / 3],
        Heading::East => &vertical[cell_id + cell_id / 3 + 1],
    };
    passage.clone()
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
    fn test_passage_reads_each_side() {
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

        let walls = |cell_id| {
            Heading::CLOCKWISE
                .iter()
                .filter(|&&side| passage(cell_id, side, &horizontal, &vertical) == Passages::WALL)
                .count()
        };
        assert_eq!(walls(0), 2);
        assert_eq!(walls(1), 2);
    }

    #[test]
    fn test_radar_sides_are_stored_in_map_headings() {
        let mut horizontal = vec![Passages::UNDEFINED; 12];
        horizontal[4] = Passages::WALL;
        let radar = Radar {
            horizontal,
            vertical: vec![Passages::UNDEFINED; 12],
            cells: vec![CellType::NOTHING; 9],
        };
        let mut player = Player::new();
        player.heading = Heading::East;
        let mut graph = MazeGraph::new();

        maze_to_graph(&radar, &player, &mut graph);

        let center = &graph.cell_map[&player.position];
        assert_eq!(center.side(Heading::East), Side::Wall);
        assert_eq!(center.side(Heading::North), Side::Unknown);
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
