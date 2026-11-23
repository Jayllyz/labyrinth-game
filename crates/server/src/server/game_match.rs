//! One game: the board, the players on it, and the hints and challenges they get.

use super::{PlayerId, ServerConfig};
use crate::game::{Game, Moved, Position};
use rand::{RngExt, SeedableRng, rngs::StdRng};
use shared::heading::Heading;
use shared::maze_generator;
use shared::messages::{Action, ActionError, Challenge, Direction, Hint, Message, RadarView};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::info;

/// Odds per move of each random message, close to what the reference server sends.
const SECRET_ODDS: f64 = 0.012;
const COMPASS_ODDS: f64 = 0.012;
const GRID_SIZE_ODDS: f64 = 0.001;
const CHALLENGE_ODDS: f64 = 0.001;

/// One game, from the lobby to the last player out: who waits, when the game starts,
/// the board, and the hints and challenges players get.
pub(super) struct Match {
    game: Game,
    rng: StdRng,
    players: HashMap<PlayerId, usize>,
    team_names: Vec<String>,
    /// Players who joined, until the game starts and places them.
    waiting: Vec<PlayerId>,
    first_join: Option<Instant>,
    started: bool,
}

impl Match {
    /// Builds the maze and places the exit from the seed, so a game can be replayed.
    pub(super) fn new(config: &ServerConfig) -> Self {
        let mut rng = StdRng::seed_from_u64(config.seed);
        let maze = maze_generator::generate(
            config.generator,
            config.width,
            config.height,
            false,
            config.seed,
        );
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

    pub(super) fn has_started(&self) -> bool {
        self.started
    }

    pub(super) fn players_out(&self) -> usize {
        self.game.players().filter(|p| p.exited).count()
    }

    /// Adds a player to the lobby, or to the board if the game already started. Starts the
    /// game once `expected` players are waiting; true if this join started it.
    pub(super) fn join(&mut self, player: PlayerId, expected: Option<usize>) -> bool {
        // Only reachable when the start timeout fired while this player was joining.
        if self.started {
            self.add_player(player);
            return false;
        }
        self.first_join.get_or_insert_with(Instant::now);
        self.waiting.push(player);
        if expected != Some(self.waiting.len()) {
            return false;
        }
        self.start();
        info!("All teams are full, the game starts");
        true
    }

    /// Starts the game once `timeout` has passed since the first player joined. `None` once
    /// the game has started, otherwise the time left to wait.
    pub(super) fn start_or_wait(&mut self, timeout: Duration) -> Option<Duration> {
        if self.started {
            return None;
        }
        let deadline = self.first_join.map_or_else(Instant::now, |at| at + timeout);
        let left = deadline.saturating_duration_since(Instant::now());
        if !left.is_zero() {
            return Some(left);
        }
        self.start();
        info!("Start timeout reached, the game starts with the players who joined");
        None
    }

    /// What the player sees, or `None` if it is not on the board.
    pub(super) fn view(&self, player: &PlayerId) -> Option<Message> {
        self.players.get(player).map(|&id| self.view_of(id))
    }

    /// The messages that answer a player's action, and whether it reached the exit, or
    /// `None` if the player is not on the board.
    pub(super) fn act(
        &mut self,
        player: &PlayerId,
        action: Action,
    ) -> Option<(Vec<Message>, bool)> {
        let id = *self.players.get(player)?;
        let (replies, exited) = self.act_by_id(id, action);
        if exited {
            info!("{player} found the exit in {} moves", self.game.player(id).moves);
            self.log_if_team_escaped(id);
        }
        Some((replies, exited))
    }

    /// Takes a disconnected player off the board.
    pub(super) fn leave(&mut self, player: &PlayerId) {
        if let Some(&id) = self.players.get(player) {
            self.game.leave(id);
        }
    }

    /// Places everyone in name order, so a seed gives each player the same cell whatever
    /// order they connected in.
    fn start(&mut self) {
        let mut waiting = std::mem::take(&mut self.waiting);
        waiting.sort();
        for player in waiting {
            self.add_player(player);
        }
        self.started = true;
    }

    /// Places a player on a free cell, facing North like on the reference server.
    fn add_player(&mut self, player: PlayerId) {
        let team = self.team_names.iter().position(|t| *t == player.team).unwrap_or_else(|| {
            self.team_names.push(player.team.clone());
            self.team_names.len() - 1
        });
        // Subscriptions stop once the maze is full, so a free cell is always left.
        let free = self.game.free_cells();
        let position = free[self.rng.random_range(0..free.len())];
        let id = self.game.add_player(team, position, Heading::North);
        self.players.insert(player, id);
    }

    fn view_of(&self, id: usize) -> Message {
        Message::RadarView(RadarView(self.game.radar(id)))
    }

    fn act_by_id(&mut self, id: usize, action: Action) -> (Vec<Message>, bool) {
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
            Ok(Moved::Exited) => (vec![self.view_of(id)], true),
            Err(err) => (vec![Message::ActionError(err), self.view_of(id)], false),
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
            replies.push(self.view_of(id));
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
            return vec![Message::ActionError(ActionError::NoRunningChallenge), self.view_of(id)];
        };
        self.game.count_action(id);
        let expected = self.game.team_secret_sum(self.game.player(id).team, modulo);
        if answer.trim().parse::<u128>().ok() != Some(expected) {
            return vec![Message::ActionError(ActionError::InvalidChallengeSolution)];
        }
        self.game.player_mut(id).challenge = None;
        vec![self.view_of(id)]
    }

    /// Logs the team's score once its last member is out.
    fn log_if_team_escaped(&self, id: usize) {
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

        let (replies, _) = game.act_by_id(0, Action::MoveTo(Direction::Front));
        assert!(matches!(replies[..], [Message::ActionError(ActionError::SolveChallengeFirst)]));

        let (replies, _) = game.act_by_id(0, Action::SolveChallenge { answer: "4".into() });
        assert!(matches!(
            replies[..],
            [Message::ActionError(ActionError::InvalidChallengeSolution)]
        ));

        let (replies, _) = game.act_by_id(0, Action::SolveChallenge { answer: "3".into() });
        assert!(matches!(replies.last(), Some(Message::RadarView(_))));
        assert_eq!(game.game.player_mut(0).challenge, None);
    }

    #[test]
    fn test_teammate_without_secret_counts_as_zero() {
        let mut game = game_in_challenge(7);
        game.game.player_mut(0).secret = Some(9);

        let (replies, _) = game.act_by_id(0, Action::SolveChallenge { answer: "2".into() });

        assert!(matches!(replies.last(), Some(Message::RadarView(_))));
    }
}
