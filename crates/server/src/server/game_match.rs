//! One game: the board, the players on it, and the hints and challenges they get.

use super::{PlayerId, ServerConfig};
use crate::game::{Game, Moved, Position};
use rand::{RngExt, SeedableRng, rngs::StdRng};
use shared::heading::Heading;
use shared::maze::Maze;
use shared::messages::{Action, ActionError, Challenge, Direction, Hint, Message, RadarView};
use std::collections::HashMap;
use std::time::Instant;
use tracing::info;

/// Odds per move of each random message, close to what the reference server sends.
const SECRET_ODDS: f64 = 0.012;
const COMPASS_ODDS: f64 = 0.012;
const GRID_SIZE_ODDS: f64 = 0.001;
const CHALLENGE_ODDS: f64 = 0.001;

/// One game: the board, and which player and team each name stands for.
pub(super) struct Match {
    pub(super) game: Game,
    rng: StdRng,
    pub(super) players: HashMap<PlayerId, usize>,
    team_names: Vec<String>,
    /// Players who joined, with their team, until the game starts and places them.
    pub(super) waiting: Vec<PlayerId>,
    pub(super) first_join: Option<Instant>,
    pub(super) started: bool,
}

impl Match {
    /// Builds the maze and places the exit from the seed, so a game can be replayed.
    pub(super) fn new(config: &ServerConfig) -> Self {
        let mut rng = StdRng::seed_from_u64(config.seed);
        let maze =
            Maze::generate(config.generator, config.width, config.height, false, config.seed);
        let exit = random_cell(&mut rng, config);
        Self {
            game: Game::new(maze, exit),
            rng,
            players: HashMap::new(),
            team_names: Vec::new(),
            waiting: Vec::new(),
            first_join: None,
            started: false,
        }
    }

    /// Places a new player on a free cell, facing North like on the reference server.
    /// Places everyone in name order, so a seed gives each player the same cell whatever
    /// order they connected in.
    pub(super) fn start(&mut self) {
        let mut waiting = std::mem::take(&mut self.waiting);
        waiting.sort();
        for player in waiting {
            self.add_player(player);
        }
        self.started = true;
    }

    pub(super) fn add_player(&mut self, player: PlayerId) -> usize {
        let team = self.team_names.iter().position(|t| *t == player.team).unwrap_or_else(|| {
            self.team_names.push(player.team.clone());
            self.team_names.len() - 1
        });
        // Subscriptions stop once the maze is full, so a free cell is always left.
        let free = self.game.free_cells();
        let position = free[self.rng.random_range(0..free.len())];
        let id = self.game.add_player(team, position, Heading::North);
        self.players.insert(player, id);
        id
    }

    pub(super) fn view(&self, id: usize) -> Message {
        Message::RadarView(RadarView(self.game.radar(id)))
    }

    /// The messages that answer a player's action, and whether it reached the exit.
    pub(super) fn act(&mut self, id: usize, action: Action) -> (Vec<Message>, bool) {
        match action {
            Action::MoveTo(direction) => self.move_player(id, direction),
            Action::SolveChallenge { answer } => (self.solve(id, &answer), false),
        }
    }

    fn move_player(&mut self, id: usize, direction: Direction) -> (Vec<Message>, bool) {
        if self.game.player(id).challenge.is_some() {
            return (vec![Message::ActionError(ActionError::SolveChallengeFirst)], false);
        }
        match self.game.apply_move(id, direction) {
            Ok(Moved::Stepped) => (self.after_move(id), false),
            // The reference server shows the exit under the player once more before closing.
            Ok(Moved::Exited) => (vec![self.view(id)], true),
            Err(err) => (vec![Message::ActionError(err), self.view(id)], false),
        }
    }

    /// The optional hint, then the view or a new challenge that replaces it.
    fn after_move(&mut self, id: usize) -> Vec<Message> {
        let mut replies = Vec::new();
        if let Some(hint) = self.hint(id) {
            replies.push(Message::Hint(hint));
        }
        if self.rng.random_bool(CHALLENGE_ODDS) {
            let modulo = self.rng.random_range(2..=1000);
            self.game.player_mut(id).challenge = Some(modulo);
            replies.push(Message::Challenge(Challenge::SecretSumModulo(modulo)));
        } else {
            replies.push(self.view(id));
        }
        replies
    }

    fn hint(&mut self, id: usize) -> Option<Hint> {
        if self.rng.random_bool(GRID_SIZE_ODDS) {
            let (columns, rows) = self.game.size();
            return Some(Hint::GridSize { columns, rows });
        }
        if self.rng.random_bool(SECRET_ODDS) {
            let secret = self.rng.random::<u64>().into();
            self.game.player_mut(id).secret = Some(secret);
            return Some(Hint::Secret(secret));
        }
        self.rng
            .random_bool(COMPASS_ODDS)
            .then(|| Hint::RelativeCompass { angle: self.game.exit_bearing(id) })
    }

    fn solve(&mut self, id: usize, answer: &str) -> Vec<Message> {
        let Some(modulo) = self.game.player(id).challenge else {
            return vec![Message::ActionError(ActionError::NoRunningChallenge), self.view(id)];
        };
        self.game.count_action(id);
        let expected = self.game.team_secret_sum(self.game.player(id).team, modulo);
        if answer.trim().parse::<u128>().ok() != Some(expected) {
            return vec![Message::ActionError(ActionError::InvalidChallengeSolution)];
        }
        self.game.player_mut(id).challenge = None;
        vec![self.view(id)]
    }

    /// Logs the team's score once its last member is out.
    pub(super) fn log_if_team_escaped(&self, id: usize) {
        let team = self.game.player(id).team;
        let members: Vec<_> = self.game.players().filter(|p| p.team == team).collect();
        if members.iter().all(|p| p.exited) {
            let moves: u32 = members.iter().map(|p| p.moves).sum();
            let score = f64::from(moves) / members.len() as f64;
            info!("Team {} escaped with a score of {score:.1}", self.team_names[team]);
        }
    }
}

fn random_cell(rng: &mut StdRng, config: &ServerConfig) -> Position {
    Position {
        x: rng.random_range(0..config.width) as i16,
        y: rng.random_range(0..config.height) as i16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A started game with two teammates, the first one in a challenge.
    fn game_in_challenge(modulo: u128) -> Match {
        let mut game = Match::new(&ServerConfig { width: 8, height: 8, ..ServerConfig::default() });
        let player = |name: &str| PlayerId { team: "A".into(), name: name.into() };
        game.waiting = vec![player("a"), player("b")];
        game.start();
        game.game.player_mut(0).challenge = Some(modulo);
        game
    }

    #[test]
    fn test_challenge_must_be_solved_before_moving() {
        let mut game = game_in_challenge(10);
        game.game.player_mut(0).secret = Some(11);
        game.game.player_mut(1).secret = Some(32);

        let (replies, _) = game.act(0, Action::MoveTo(Direction::Front));
        assert!(matches!(replies[..], [Message::ActionError(ActionError::SolveChallengeFirst)]));

        let (replies, _) = game.act(0, Action::SolveChallenge { answer: "4".into() });
        assert!(matches!(
            replies[..],
            [Message::ActionError(ActionError::InvalidChallengeSolution)]
        ));

        let (replies, _) = game.act(0, Action::SolveChallenge { answer: "3".into() });
        assert!(matches!(replies.last(), Some(Message::RadarView(_))));
        assert_eq!(game.game.player_mut(0).challenge, None);
    }

    #[test]
    fn test_teammate_without_secret_counts_as_zero() {
        let mut game = game_in_challenge(7);
        game.game.player_mut(0).secret = Some(9);

        let (replies, _) = game.act(0, Action::SolveChallenge { answer: "2".into() });

        assert!(matches!(replies.last(), Some(Message::RadarView(_))));
    }
}
