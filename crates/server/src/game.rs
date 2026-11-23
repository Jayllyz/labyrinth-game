//! Game rules: moving players, reaching the exit, and encoding what each player sees.

use shared::{
    heading::Heading,
    maze::{Maze, PositionType},
    messages::{ActionError, Direction},
    radar::encode_base64,
};

/// A maze cell, `x` towards East and `y` towards South.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i16,
    pub y: i16,
}

impl Position {
    /// The cell `steps` cells away along `heading`.
    fn walk(self, heading: Heading, steps: i16) -> Self {
        let delta = heading.delta();
        Self { x: self.x + delta.row * steps, y: self.y + delta.column * steps }
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub team: usize,
    pub position: Position,
    pub heading: Heading,
    pub moves: u32,
    pub exited: bool,
    /// Disconnected before reaching the exit.
    pub left: bool,
}

impl Player {
    fn on_board(&self) -> bool {
        !self.exited && !self.left
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Moved {
    Stepped,
    Exited,
}

pub struct Game {
    maze: Maze,
    exit: Position,
    players: Vec<Player>,
}

impl Game {
    /// `maze` is a wall grid of `(2 * height + 1) x (2 * width + 1)`, cells at odd indices.
    pub const fn new(maze: Maze, exit: Position) -> Self {
        Self { maze, exit, players: Vec::new() }
    }

    pub fn add_player(&mut self, team: usize, position: Position, heading: Heading) -> usize {
        self.players.push(Player { team, position, heading, moves: 0, exited: false, left: false });
        self.players.len() - 1
    }

    pub fn player(&self, id: usize) -> &Player {
        &self.players[id]
    }

    /// Takes a disconnected player off the board, so it blocks nobody.
    pub fn leave(&mut self, id: usize) {
        self.players[id].left = true;
    }

    /// Cells where a new player may start.
    pub fn free_cells(&self) -> Vec<Position> {
        let (width, height) = ((self.maze.col_len / 2) as i16, (self.maze.row_len / 2) as i16);
        (0..height)
            .flat_map(|y| (0..width).map(move |x| Position { x, y }))
            .filter(|&cell| self.is_free(cell))
            .collect()
    }

    pub fn players(&self) -> impl Iterator<Item = &Player> {
        self.players.iter()
    }

    /// Whether a new player may start on this cell: not the exit, and nobody there.
    pub fn is_free(&self, position: Position) -> bool {
        position != self.exit
            && self.players.iter().all(|p| !p.on_board() || p.position != position)
    }

    /// Moves a player; a rejected move changes neither its cell nor its heading.
    pub fn apply_move(&mut self, id: usize, direction: Direction) -> Result<Moved, ActionError> {
        let player = &self.players[id];
        let heading = player.heading.turn(direction);
        let target = player.position.walk(heading, 1);
        if !self.is_open(player.position, target) {
            return Err(ActionError::CannotPassThroughWall);
        }
        // Teammates may share a cell; players who left through the exit are off the board.
        if self.opponent_at(target, player.team) {
            return Err(ActionError::CannotPassThroughOpponent);
        }

        let player = &mut self.players[id];
        player.position = target;
        player.heading = heading;
        player.moves += 1;
        player.exited = target == self.exit;
        Ok(if player.exited { Moved::Exited } else { Moved::Stepped })
    }

    /// The encoded 3x3 view around a player, its heading at the top.
    pub fn radar(&self, id: usize) -> String {
        let me = &self.players[id];
        let right = me.heading.turn(Direction::Right);
        // Radar rows and columns run from -1 to 3 so the outer passages have a cell
        // on both sides.
        let cell =
            |row: i16, column: i16| me.position.walk(me.heading, 1 - row).walk(right, column - 1);
        let visible = self.visibility(&cell);
        let inside = |(row, column): (i16, i16)| (0..3).contains(&row) && (0..3).contains(&column);
        let lit = |at: (i16, i16)| inside(at) && visible[at.0 as usize][at.1 as usize];
        let is_corner = |(row, column): (i16, i16)| row != 1 && column != 1;
        // A passage is seen from the center when it borders a visible center or side cell,
        // or is an outer side of a visible corner.
        let passage = |a: (i16, i16), b: (i16, i16)| -> u32 {
            let seen_from =
                |near: (i16, i16), far: (i16, i16)| lit(near) && (!is_corner(near) || !inside(far));
            if !seen_from(a, b) && !seen_from(b, a) {
                return 0;
            }
            if self.is_open(cell(a.0, a.1), cell(b.0, b.1)) { 1 } else { 2 }
        };

        let horizontal = (0..4)
            .flat_map(|row| (0..3).map(move |column| ((row - 1, column), (row, column))))
            .fold(0, |bits, (above, below)| (bits << 2) | passage(above, below));
        let vertical = (0..3)
            .flat_map(|row| (0..4).map(move |column| ((row, column - 1), (row, column))))
            .fold(0, |bits, (left, right)| (bits << 2) | passage(left, right));
        let items = (0..3).flat_map(|row| (0..3).map(move |column| (row, column))).fold(
            0u64,
            |bits, (row, column)| {
                let item = if visible[row as usize][column as usize] {
                    self.item(cell(row, column), id)
                } else {
                    0b1111
                };
                (bits << 4) | item
            },
        );

        let mut bytes = Vec::with_capacity(11);
        bytes.extend_from_slice(&horizontal.to_le_bytes()[..3]);
        bytes.extend_from_slice(&vertical.to_le_bytes()[..3]);
        bytes.extend_from_slice(&(items << 4).to_be_bytes()[3..]);
        encode_base64(&bytes)
    }

    /// Which radar cells the player sees: those in a straight line through open
    /// passages, and corners reached through one open side cell.
    fn visibility(&self, cell: &impl Fn(i16, i16) -> Position) -> [[bool; 3]; 3] {
        let mut visible = [[false; 3]; 3];
        visible[1][1] = true;
        for (row, column) in [(0, 1), (1, 0), (1, 2), (2, 1)] {
            visible[row][column] = self.is_open(cell(1, 1), cell(row as i16, column as i16));
        }
        for (row, column) in [(0, 0), (0, 2), (2, 0), (2, 2)] {
            let corner = cell(row as i16, column as i16);
            let through = |side_row: usize, side_column: usize| {
                visible[side_row][side_column]
                    && self.is_open(cell(side_row as i16, side_column as i16), corner)
            };
            visible[row][column] = through(row, 1) || through(1, column);
        }
        visible
    }

    /// The radar item of a visible cell: the exit in the high bits, any other player in
    /// the low bits.
    fn item(&self, position: Position, viewer: usize) -> u64 {
        let nature = if position == self.exit { 0b1000 } else { 0 };
        let team = self.players[viewer].team;
        let entity = self
            .players
            .iter()
            .enumerate()
            .filter(|&(id, player)| {
                id != viewer && player.on_board() && player.position == position
            })
            .map(|(_, player)| if player.team == team { 0b01 } else { 0b10 })
            .max()
            .unwrap_or(0);
        nature | entity
    }

    fn opponent_at(&self, position: Position, team: usize) -> bool {
        self.players.iter().any(|p| p.on_board() && p.team != team && p.position == position)
    }

    fn contains(&self, position: Position) -> bool {
        let width = (self.maze.col_len / 2) as i16;
        let height = (self.maze.row_len / 2) as i16;
        (0..width).contains(&position.x) && (0..height).contains(&position.y)
    }

    /// Whether two neighboring cells are joined; leaving the maze never is.
    fn is_open(&self, a: Position, b: Position) -> bool {
        if !self.contains(a) || !self.contains(b) {
            return false;
        }
        let (row, column) = ((a.y + b.y + 1) as usize, (a.x + b.x + 1) as usize);
        self.maze.map[row][column] != PositionType::WALL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::{
        maze::Cell,
        radar::{CellType, decode_base64, extract_data},
    };

    /// A one-row corridor of three cells, the exit at the East end.
    fn corridor() -> Game {
        let map = ["#######", "#     #", "#######"]
            .iter()
            .map(|row| {
                row.chars()
                    .map(|c| if c == '#' { PositionType::WALL } else { PositionType::SPACE })
                    .collect()
            })
            .collect();
        let origin = Cell { row: 0, column: 0 };
        Game::new(Maze::new(map, origin, origin), Position { x: 2, y: 0 })
    }

    #[test]
    fn test_move_into_a_wall_keeps_cell_and_heading() {
        let mut game = corridor();
        let id = game.add_player(0, Position { x: 0, y: 0 }, Heading::North);

        assert_eq!(game.apply_move(id, Direction::Right), Ok(Moved::Stepped));
        assert_eq!(game.apply_move(id, Direction::Left), Err(ActionError::CannotPassThroughWall));

        assert_eq!(game.player(id).position, Position { x: 1, y: 0 });
        assert_eq!(game.player(id).heading, Heading::East);
    }

    #[test]
    fn test_only_teammates_share_a_cell() {
        for (other_team, expected) in
            [(0, Ok(Moved::Stepped)), (1, Err(ActionError::CannotPassThroughOpponent))]
        {
            let mut game = corridor();
            let id = game.add_player(0, Position { x: 0, y: 0 }, Heading::East);
            game.add_player(other_team, Position { x: 1, y: 0 }, Heading::East);

            assert_eq!(game.apply_move(id, Direction::Front), expected);
        }
    }

    #[test]
    fn test_reaching_the_exit_takes_the_player_off_the_board() {
        let mut game = corridor();
        let id = game.add_player(0, Position { x: 1, y: 0 }, Heading::East);
        let opponent = game.add_player(1, Position { x: 0, y: 0 }, Heading::East);

        assert_eq!(game.apply_move(id, Direction::Front), Ok(Moved::Exited));
        assert_eq!(game.apply_move(opponent, Direction::Front), Ok(Moved::Stepped));
        assert_eq!(game.apply_move(opponent, Direction::Front), Ok(Moved::Exited));
    }

    #[test]
    fn test_a_player_who_left_frees_its_cell() {
        let mut game = corridor();
        let id = game.add_player(0, Position { x: 0, y: 0 }, Heading::East);
        let gone = game.add_player(1, Position { x: 1, y: 0 }, Heading::East);

        game.leave(gone);

        assert_eq!(game.apply_move(id, Direction::Front), Ok(Moved::Stepped));
    }

    #[test]
    fn test_radar_shows_teammates_and_opponents_but_not_the_viewer() {
        let mut game = corridor();
        let id = game.add_player(0, Position { x: 1, y: 0 }, Heading::East);
        game.add_player(0, Position { x: 2, y: 0 }, Heading::North);
        game.add_player(1, Position { x: 0, y: 0 }, Heading::North);

        let radar = extract_data(&decode_base64(&game.radar(id)));

        assert_eq!(radar.cells[1], CellType::ObjectiveAlly);
        assert_eq!(radar.cells[4], CellType::NOTHING);
        assert_eq!(radar.cells[7], CellType::ENEMY);
    }
}
