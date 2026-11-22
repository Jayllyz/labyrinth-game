use crate::data_structures::maze_graph::MazeGraph;
use crate::instructions::{self, Algorithm, WallFollowerState};
use crate::maze_parser::Player;
use shared::{maze::Cell, messages::Action, radar::Radar};

/// What a solver sees, and may change, to choose one move.
pub struct Turn<'a> {
    pub player: &'a mut Player,
    pub graph: &'a mut MazeGraph,
    pub radar: &'a Radar,
    /// A cell an opponent holds, to route around.
    pub avoid: Option<Cell>,
}

/// An exploration algorithm. It keeps its own state between turns; the agent clones it
/// with the pending move, so a move the server rejects also undoes the solver's changes.
pub trait Solver: CloneSolver + Send {
    fn next_move(&mut self, turn: &mut Turn) -> Action;
}

pub trait CloneSolver {
    fn clone_box(&self) -> Box<dyn Solver>;
}

impl<T: Solver + Clone + 'static> CloneSolver for T {
    fn clone_box(&self) -> Box<dyn Solver> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Solver> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl Algorithm {
    pub fn solver(self, agent_name: &str) -> Box<dyn Solver> {
        match self {
            Self::Tremeaux => Box::new(Tremeaux),
            Self::WallFollower => Box::new(WallFollower::default()),
            Self::Alian => Box::new(Alian { agent_name: agent_name.to_string() }),
        }
    }
}

#[derive(Clone)]
struct Tremeaux;

impl Solver for Tremeaux {
    fn next_move(&mut self, turn: &mut Turn) -> Action {
        instructions::tremeaux_solver(turn.player, turn.graph, turn.avoid)
    }
}

#[derive(Clone, Default)]
struct WallFollower {
    state: WallFollowerState,
}

impl Solver for WallFollower {
    fn next_move(&mut self, turn: &mut Turn) -> Action {
        instructions::wall_follower_solver(
            turn.radar,
            turn.player,
            turn.graph,
            &mut self.state,
            turn.avoid,
        )
    }
}

#[derive(Clone)]
struct Alian {
    /// Alian marks the cells each agent visited.
    agent_name: String,
}

impl Solver for Alian {
    fn next_move(&mut self, turn: &mut Turn) -> Action {
        instructions::alian_solver(turn.player, turn.graph, &self.agent_name)
    }
}
