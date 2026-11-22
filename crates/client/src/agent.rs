use crate::data_structures::maze_graph::MazeGraph;
use crate::instructions::{self, Algorithm};
use crate::maze_parser::{Player, maze_to_graph};
use crate::solver::{Solver, Turn};
use rand::{RngExt, SeedableRng, rngs::StdRng};
use shared::{
    errors::{GameError, GameResult},
    maze::Cell,
    messages::{Action, ActionError, Challenge, Hint, Message, RadarView, SubscribePlayerResult},
    radar::{Radar, decode_base64, extract_data},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing::{debug, error, info, warn};

/// Secrets of the whole team, by agent name. A challenge is answered with all of them.
pub type TeamSecrets = Arc<Mutex<HashMap<String, u128>>>;

/// Opponents move on quickly, so a held cell is only avoided for a few moves.
pub const AVOID_OPPONENT_TURNS: u8 = 3;

/// One agent's game logic. It never touches the network: the transport feeds it server
/// messages and sends back the actions it replies with.
pub struct Agent {
    name: String,
    graph: MazeGraph,
    player: Player,
    solver: Box<dyn Solver>,
    secrets: TeamSecrets,
    challenge_modulo: Option<u128>,
    /// Cell an opponent recently held and turns left to route around it.
    avoid: Option<(Cell, u8)>,
    /// The agent and its map as they will be once the server accepts the last move.
    pending: Option<Pending>,
    rng: StdRng,
}

struct Pending {
    player: Player,
    graph: MazeGraph,
    solver: Box<dyn Solver>,
}

impl Agent {
    pub fn new(name: String, algorithm: Algorithm, secrets: TeamSecrets, seed: u64) -> Self {
        Self {
            graph: MazeGraph::new(),
            player: Player::new(),
            solver: algorithm.solver(&name),
            secrets,
            challenge_modulo: None,
            avoid: None,
            pending: None,
            rng: StdRng::seed_from_u64(seed),
            name,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn graph(&self) -> &MazeGraph {
        &self.graph
    }

    pub const fn player(&self) -> &Player {
        &self.player
    }

    /// Returns the action to send back, if any.
    pub fn on_message(&mut self, message: Message) -> GameResult<Option<Action>> {
        match message {
            Message::SubscribePlayerResult(SubscribePlayerResult::Ok) => {
                info!("Subscribed successfully");
                Ok(None)
            }
            Message::SubscribePlayerResult(SubscribePlayerResult::Err(err)) => {
                Err(GameError::AgentSubscriptionError(format!("{err:?}")))
            }
            Message::RadarView(view) => Ok(Some(self.on_radar_view(&view))),
            Message::Hint(hint) => {
                self.on_hint(hint);
                Ok(None)
            }
            Message::Challenge(challenge) => {
                debug!("{challenge:?}");
                let Challenge::SecretSumModulo(modulo) = challenge;
                self.challenge_modulo = Some(modulo);
                Ok(self.solve_challenge(modulo))
            }
            Message::ActionError(err) => Ok(self.on_action_error(err)),
            Message::MessageError(err) => {
                error!("Server error: {err:?}");
                Ok(None)
            }
            other => {
                warn!("Unhandled message: {other:?}");
                Ok(None)
            }
        }
    }

    fn on_radar_view(&mut self, view: &RadarView) -> Action {
        let radar_view = extract_data(&decode_base64(&view.0));

        // A radar view after a move means the server accepted it.
        if let Some(Pending { player, graph, solver }) = self.pending.take() {
            self.player = player;
            self.graph = graph;
            self.solver = solver;
        }
        maze_to_graph(&radar_view, &self.player, &mut self.graph);

        let mut next = self.copy_state();
        let action = self.decide(&radar_view, &mut next);
        self.pending = Some(next);
        action
    }

    fn copy_state(&self) -> Pending {
        Pending {
            player: self.player.clone(),
            graph: self.graph.clone(),
            solver: self.solver.clone(),
        }
    }

    /// Chooses the next move, updating `next` as if the server accepts it. Steps every
    /// solver needs (stepping aside, heading for a seen exit) come before the solver.
    fn decide(&mut self, radar: &Radar, next: &mut Pending) -> Action {
        let Pending { player, graph, solver } = next;
        // Right after a block, yield half the time so two agents blocking each other don't
        // retry in lockstep.
        let blocked = self
            .avoid
            .filter(|&(_, turns)| turns == AVOID_OPPONENT_TURNS && self.rng.random_bool(0.5))
            .map(|(cell, _)| cell);
        let yielded = blocked.and_then(|cell| instructions::yield_to_opponent(player, graph, cell));

        let avoid = self.avoid.map(|(cell, _)| cell);
        self.avoid = self.avoid.and_then(|(cell, turns)| (turns > 1).then_some((cell, turns - 1)));

        if let Some(action) =
            yielded.or_else(|| instructions::step_to_known_exit(player, graph, avoid))
        {
            return action;
        }

        solver.next_move(&mut Turn { player, graph, radar, avoid })
    }

    fn on_hint(&mut self, hint: Hint) {
        match hint {
            Hint::RelativeCompass { angle } => {
                // The hint describes the agent as of the last radar view, before the pending move.
                self.player.record_compass(angle);
                if let Some(pending) = &mut self.pending {
                    pending.player.exit_bearing = self.player.exit_bearing;
                }
            }
            Hint::Secret(secret) => {
                if let Ok(mut secrets) = self.secrets.lock() {
                    secrets.insert(self.name.clone(), secret);
                }
            }
            Hint::GridSize { columns, rows } => info!("Maze is {columns}x{rows}"),
            Hint::SOS => info!("Received an SOS hint"),
        }
    }

    fn on_action_error(&mut self, err: ActionError) -> Option<Action> {
        if matches!(
            err,
            ActionError::CannotPassThroughWall | ActionError::CannotPassThroughOpponent
        ) {
            debug!("Action error: {err:?}");
        } else {
            error!("Action error: {err:?}");
        }

        // The server follows move errors with a fresh RadarView, which drives the next move.
        match err {
            // Without a pending move the error is stale, so there is no target to avoid.
            ActionError::CannotPassThroughOpponent => {
                if let Some(rejected) = self.pending.take() {
                    self.avoid = Some((rejected.player.position, AVOID_OPPONENT_TURNS));
                }
                None
            }
            ActionError::CannotPassThroughWall
            | ActionError::InvalidMove
            | ActionError::OutOfMap
            | ActionError::Blocked => {
                self.pending = None;
                None
            }
            ActionError::NoRunningChallenge => None,
            ActionError::InvalidChallengeSolution => {
                self.challenge_modulo.and_then(|modulo| self.solve_challenge(modulo))
            }
            ActionError::SolveChallengeFirst => {
                self.pending = None;
                self.challenge_modulo.and_then(|modulo| self.solve_challenge(modulo))
            }
        }
    }

    fn solve_challenge(&self, modulo: u128) -> Option<Action> {
        let secrets = self.secrets.lock().ok()?;
        Some(Action::SolveChallenge { answer: instructions::solve_sum_modulo(modulo, &secrets) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structures::maze_graph::CellStatus;
    use crate::heading::Heading;
    use shared::messages::{self, Direction};
    use shared::radar::CellType;

    fn agent(name: &str) -> Agent {
        Agent::new(name.to_string(), Algorithm::Tremeaux, TeamSecrets::default(), 0)
    }

    /// Chooses a move from the current map, as a radar view would.
    fn propose_move(agent: &mut Agent) -> Action {
        let radar = Radar { horizontal: Vec::new(), vertical: Vec::new(), cells: Vec::new() };
        let mut next = agent.copy_state();
        let action = agent.decide(&radar, &mut next);
        agent.pending = Some(next);
        action
    }

    /// Moves forward on its first turn and back after that, to show whether its state
    /// survived.
    #[derive(Clone, Default)]
    struct CountingSolver {
        turns: u32,
    }

    impl Solver for CountingSolver {
        fn next_move(&mut self, _: &mut Turn) -> Action {
            self.turns += 1;
            let direction = if self.turns == 1 { Direction::Front } else { Direction::Back };
            Action::MoveTo(direction)
        }
    }

    fn answer(action: Option<Action>) -> String {
        match action {
            Some(Action::SolveChallenge { answer }) => answer,
            other => panic!("unexpected action: {other:?}"),
        }
    }

    #[test]
    fn test_failed_move_waits_for_next_radar_view() {
        for err in [
            ActionError::CannotPassThroughWall,
            ActionError::CannotPassThroughOpponent,
            ActionError::NoRunningChallenge,
        ] {
            let mut agent = agent("Player1");

            let reply = agent.on_message(Message::ActionError(err)).unwrap();

            assert_eq!(reply, None);
        }
    }

    #[test]
    fn test_opponent_block_reverts_and_remembers_blocked_cell() {
        let mut agent = agent("Player1");
        let mut moved = agent.player.clone();
        moved.move_forward();
        let blocked = moved.position;
        agent.pending = Some(Pending {
            player: moved,
            graph: agent.graph.clone(),
            solver: agent.solver.clone(),
        });

        agent.on_message(Message::ActionError(ActionError::CannotPassThroughOpponent)).unwrap();

        assert_eq!(agent.player.position, Player::new().position);
        assert_eq!(agent.avoid, Some((blocked, AVOID_OPPONENT_TURNS)));
    }

    #[test]
    fn test_rejected_backtrack_restores_graph() {
        let mut agent = agent("Player1");
        let current = Cell { row: 0, column: 0 };
        let parent = Cell { row: 0, column: 1 };
        for cell in [current, parent] {
            agent.graph.add(cell, CellType::NOTHING);
            agent.graph.update_cell_status(cell, CellStatus::VISITED);
        }
        agent.graph.add_neighbor(&current, &parent);
        agent.graph.add_neighbor(&parent, &current);
        agent.graph.set_parent(current, parent);

        propose_move(&mut agent);
        agent.on_message(Message::ActionError(ActionError::CannotPassThroughOpponent)).unwrap();

        assert_eq!(agent.player.position, current);
        assert_eq!(agent.graph.get_cell_status(current), CellStatus::VISITED);
    }

    #[test]
    fn test_move_rejected_for_open_challenge_is_dropped() {
        let mut agent = agent("Player1");
        propose_move(&mut agent);

        agent.on_message(Message::ActionError(ActionError::SolveChallengeFirst)).unwrap();

        assert!(agent.pending.is_none());
    }

    #[test]
    fn test_rejected_move_rolls_back_the_solver_state() {
        let mut agent = agent("Player1");
        agent.solver = Box::new(CountingSolver::default());
        propose_move(&mut agent);

        agent.on_message(Message::ActionError(ActionError::CannotPassThroughWall)).unwrap();

        assert_eq!(propose_move(&mut agent), Action::MoveTo(Direction::Front));
    }

    #[test]
    fn test_each_agent_answers_challenge_with_its_own_modulo() {
        let secrets = TeamSecrets::default();
        let mut agent_a =
            Agent::new("Player1".into(), Algorithm::Tremeaux, Arc::clone(&secrets), 0);
        let mut agent_b =
            Agent::new("Player2".into(), Algorithm::Tremeaux, Arc::clone(&secrets), 0);
        agent_a.on_message(Message::Hint(Hint::Secret(25))).unwrap();

        let challenge = |m| Message::Challenge(Challenge::SecretSumModulo(m));
        let invalid = Message::ActionError(ActionError::InvalidChallengeSolution);
        let answers = [
            answer(agent_a.on_message(challenge(10)).unwrap()),
            answer(agent_b.on_message(challenge(7)).unwrap()),
            answer(agent_a.on_message(invalid).unwrap()),
        ];

        assert_eq!(answers, ["5", "4", "5"]);
    }

    #[test]
    fn test_radar_view_is_answered_with_a_move() {
        let mut agent = agent("Player1");

        let reply = agent
            .on_message(Message::RadarView(messages::RadarView("bKgGjsIyap8p8aa".to_string())))
            .unwrap();

        assert!(matches!(reply, Some(Action::MoveTo(_))));
    }

    #[test]
    fn test_move_is_applied_only_once_the_server_accepts_it() {
        let mut agent = agent("Player1");
        let radar = || Message::RadarView(messages::RadarView("bKgGjsIyap8p8aa".to_string()));

        agent.on_message(radar()).unwrap();
        let before_answer = agent.player.position;
        agent.on_message(radar()).unwrap();

        assert_eq!(before_answer, Player::new().position);
        assert_ne!(agent.player.position, Player::new().position);
    }

    #[test]
    fn test_compass_hint_is_stored_in_the_agent_frame() {
        let mut agent = agent("Player1");
        agent.player.heading = Heading::East;
        let mut turned = agent.player.clone();
        turned.turn(Direction::Right);
        agent.pending = Some(Pending {
            player: turned,
            graph: agent.graph.clone(),
            solver: agent.solver.clone(),
        });

        agent.on_message(Message::Hint(Hint::RelativeCompass { angle: 90.0 })).unwrap();

        assert_eq!(agent.player.exit_bearing, Some(180.0));
        assert_eq!(agent.pending.map(|p| p.player.exit_bearing), Some(Some(180.0)));
    }
}
