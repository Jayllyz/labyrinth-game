//! What the TUI shows, built from the events agents send.

use crate::data_structures::maze_graph::MazeGraph;
use crate::maze_parser::Player;
use std::collections::HashMap;
use tracing::Level;

/// What agents tell the TUI. Agents only send; the TUI thread owns the state, so agents
/// never wait on a lock while it draws.
pub enum AgentEvent {
    Log { agent: String, level: Level, text: String },
    Update { agent: String, graph: MazeGraph, player: Player },
    ExitFound { agent: String },
    Finished,
}

pub struct AgentState {
    pub logs: Vec<(String, Level)>,
    pub graph: MazeGraph,
    pub player: Player,
    pub found_exit: bool,
}

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentState {
    pub fn new() -> Self {
        Self { logs: Vec::new(), graph: MazeGraph::new(), player: Player::new(), found_exit: false }
    }
}

pub struct GameState {
    pub(super) agents: HashMap<String, AgentState>,
    pub(super) selected_tab: usize,
    pub(super) finished: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        Self { agents: HashMap::new(), selected_tab: 0, finished: false }
    }

    pub fn mark_finished(&mut self) {
        self.finished = true;
    }

    pub fn mark_exit_found(&mut self, agent: &str) {
        if let Some(state) = self.agents.get_mut(agent) {
            state.found_exit = true;
        }
    }

    pub fn register_agent(&mut self, name: String) {
        self.agents.insert(name, AgentState::new());
    }

    pub fn add_log(&mut self, agent: &str, message: String, level: Level) {
        if let Some(state) = self.agents.get_mut(agent) {
            state.logs.push((message, level));
        }
    }

    pub fn update_state(&mut self, agent: &str, graph: MazeGraph, player: Player) {
        if let Some(state) = self.agents.get_mut(agent) {
            state.graph = graph;
            state.player = player;
        }
    }

    pub fn apply(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::Log { agent, level, text } => self.add_log(&agent, text, level),
            AgentEvent::Update { agent, graph, player } => self.update_state(&agent, graph, player),
            AgentEvent::ExitFound { agent } => self.mark_exit_found(&agent),
            AgentEvent::Finished => self.mark_finished(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::{heading::Heading, maze::Cell, radar::CellType};

    #[test]
    fn test_game_state_new() {
        let game_state = GameState::new();
        assert_eq!(game_state.agents.len(), 0);
        assert_eq!(game_state.selected_tab, 0);
    }

    #[test]
    fn test_game_state_register_agent() {
        let mut game_state = GameState::new();
        game_state.register_agent("agent1".to_string());
        game_state.register_agent("agent2".to_string());

        assert!(game_state.agents.contains_key("agent1"));
        assert!(game_state.agents.contains_key("agent2"));
        assert_eq!(game_state.agents.len(), 2);
    }

    #[test]
    fn test_game_state_add_log() {
        let mut game_state = GameState::new();
        game_state.register_agent("agent1".to_string());

        game_state.add_log("agent1", "Test log".to_string(), Level::INFO);
        game_state.add_log("agent1", "Warning log".to_string(), Level::WARN);

        let agent_state = &game_state.agents["agent1"];
        assert_eq!(agent_state.logs.len(), 2);
    }

    #[test]
    fn test_game_state_update_state() {
        let mut game_state = GameState::new();
        game_state.register_agent("agent1".to_string());

        let mut graph = MazeGraph::new();
        let cell = Cell { row: 1, column: 1 };
        graph.add(cell, CellType::NOTHING);

        let player = Player { position: cell, heading: Heading::North, exit_bearing: None };

        game_state.update_state("agent1", graph.clone(), player);

        let agent_state = &game_state.agents["agent1"];
        assert_eq!(agent_state.player.position, cell);
        assert!(agent_state.graph.contains(&cell));
    }
}
