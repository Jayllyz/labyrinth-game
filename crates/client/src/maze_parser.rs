use shared::heading::Heading;
use shared::maze::Cell;
use shared::maze::{Maze, PositionType};
use shared::messages::{Action, Direction};

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
