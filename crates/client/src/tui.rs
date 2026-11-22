use crate::data_structures::maze_graph::{CellStatus, MazeCell, MazeGraph};
use crate::maze_parser::Player;
use ratatui::crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    Frame, Terminal,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Tabs},
};
use shared::{logger::LogLevel, maze::Cell, radar::CellType};
use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub struct AgentState {
    pub logs: Vec<(String, LogLevel)>,
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
    agents: HashMap<String, AgentState>,
    selected_tab: usize,
    finished: bool,
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

    pub fn has_found_exit(&self, agent: &str) -> bool {
        self.agents.get(agent).is_some_and(|state| state.found_exit)
    }

    pub fn register_agent(&mut self, name: String) {
        self.agents.insert(name, AgentState::new());
    }

    pub fn add_log(&mut self, agent: &str, message: String, level: LogLevel) {
        if let Some(state) = self.agents.get_mut(agent) {
            state.logs.push((message, level));
        }
    }

    pub fn update_state(&mut self, agent: &str, graph: MazeGraph, player: Player) {
        if let Some(state) = self.agents.get_mut(agent) {
            state.graph = graph;
            state.player = player.clone();
        }
    }
}

#[cfg(not(test))]
pub struct Tui {
    terminal: Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    state: Arc<Mutex<GameState>>,
    refresh_rate: u64,
}

#[cfg(not(test))]
fn backend_err(e: io::Error) -> io::Error {
    e
}

#[cfg(test)]
fn backend_err(
    e: <ratatui::backend::TestBackend as ratatui::backend::Backend>::Error,
) -> io::Error {
    match e {}
}

#[cfg(test)]
pub struct Tui {
    terminal: Terminal<ratatui::backend::TestBackend>,
    state: Arc<Mutex<GameState>>,
    refresh_rate: u64,
}

impl Tui {
    #[cfg(not(test))]
    pub fn new(refresh_rate: u64) -> Result<Self, std::io::Error> {
        let terminal = Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stdout()))?;
        let state = Arc::new(Mutex::new(GameState::new()));
        Ok(Self { terminal, state, refresh_rate })
    }

    #[cfg(test)]
    pub fn new(refresh_rate: u64) -> Result<Self, std::io::Error> {
        let Ok(terminal) = Terminal::new(ratatui::backend::TestBackend::new(10, 10));
        let state = Arc::new(Mutex::new(GameState::new()));
        Ok(Self { terminal, state, refresh_rate })
    }

    pub fn enter(&self) -> Result<(), std::io::Error> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            std::io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?;
        Ok(())
    }

    pub fn exit(&self) -> Result<(), std::io::Error> {
        if crossterm::terminal::is_raw_mode_enabled()? {
            crossterm::execute!(
                std::io::stdout(),
                LeaveAlternateScreen,
                DisableMouseCapture,
                cursor::Show
            )?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    pub fn get_state(&self) -> Arc<Mutex<GameState>> {
        Arc::clone(&self.state)
    }

    pub fn run(&mut self) -> io::Result<()> {
        let update_interval = Duration::from_millis(self.refresh_rate);
        let mut last_draw = Instant::now() - update_interval;

        loop {
            if last_draw.elapsed() >= update_interval {
                self.draw()?;
                last_draw = Instant::now();
            }

            if event::poll(Duration::from_millis(100))?
                && let Event::Key(key) = event::read()?
            {
                if let KeyCode::Char('q') = key.code {
                    self.exit()?;
                    break;
                }

                let mut state = match self.state.lock() {
                    Ok(state) => state,
                    Err(_) => continue,
                };
                let agent_count = state.agents.len();
                match key.code {
                    KeyCode::Right => {
                        Self::select_agent_increment(&mut state, agent_count);
                    }
                    KeyCode::Left => {
                        Self::select_agent_decrement(&mut state, agent_count);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn select_agent_increment(state: &mut GameState, agent_count: usize) {
        if agent_count > 0 {
            state.selected_tab = (state.selected_tab + 1) % agent_count;
        } else {
            state.selected_tab = 0;
        }
    }

    fn select_agent_decrement(state: &mut GameState, agent_count: usize) {
        if state.selected_tab > 0 {
            state.selected_tab -= 1;
        } else {
            state.selected_tab = agent_count.saturating_sub(1);
        }
    }

    fn draw(&mut self) -> io::Result<()> {
        let terminal_size = self.terminal.size().map_err(backend_err)?;
        let area = ratatui::layout::Rect::new(0, 0, terminal_size.width, terminal_size.height);

        let full_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(10)])
            .split(area);

        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(full_layout[2]);

        let maze_area = content_layout[0];
        let view_height = (maze_area.height as i16).saturating_sub(2);
        let view_width = (maze_area.width as i16).saturating_sub(4);

        let state = self.state.lock().map_err(|_| io::Error::other("Failed to lock state"))?;
        let mut agents: Vec<(String, bool)> =
            state.agents.iter().map(|(name, agent)| (name.clone(), agent.found_exit)).collect();
        agents.sort();
        let agent_names: Vec<String> = agents.iter().map(|(name, _)| name.clone()).collect();
        let selected_tab = state.selected_tab;
        let result = state.finished.then(|| {
            let found = state.agents.values().filter(|agent| agent.found_exit).count();
            (found, state.agents.len())
        });

        let (maze_viz, agent_data) = if let Some(agent_name) = agent_names.get(selected_tab) {
            if let Some(agent) = state.agents.get(agent_name) {
                (
                    self.create_maze_visualization(
                        &agent.graph,
                        &agent.player,
                        view_width,
                        view_height,
                    ),
                    Some((self.create_stats(agent), agent.logs.clone())),
                )
            } else {
                (Vec::new(), None)
            }
        } else {
            (Vec::new(), None)
        };

        drop(state);

        self.terminal
            .draw(|f| {
                let size = f.area();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(10),
                    ])
                    .split(size);

                let tabs = Tabs::new(
                    agents
                        .iter()
                        .enumerate()
                        .map(|(i, (name, found_exit))| {
                            let label =
                                if *found_exit { format!("{name} ✓") } else { name.clone() };
                            let style = match (i == selected_tab, found_exit) {
                                (true, _) => {
                                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                                }
                                (false, true) => Style::default().fg(Color::Green),
                                (false, false) => Style::default().fg(Color::Gray),
                            };
                            Line::styled(label, style)
                        })
                        .collect::<Vec<_>>(),
                )
                .block(Block::default().borders(Borders::ALL).title("Agents"))
                .select(selected_tab);

                f.render_widget(tabs, chunks[0]);

                if let Some(((explored, stats), logs)) = agent_data {
                    let stats_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Length(30), Constraint::Min(10)])
                        .split(chunks[1]);
                    let progress = Gauge::default()
                        .block(Block::default().borders(Borders::ALL).title("Progress"))
                        .gauge_style(Style::default().fg(Color::Blue))
                        .ratio(explored.clamp(0.0, 1.0))
                        .label(format!("{:.0}% explored", explored * 100.0));
                    f.render_widget(progress, stats_chunks[0]);
                    let stats_widget = Paragraph::new(stats)
                        .block(Block::default().borders(Borders::ALL).title("Statistics"));
                    f.render_widget(stats_widget, stats_chunks[1]);

                    let content_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                        .split(chunks[2]);

                    let maze_widget = Paragraph::new(maze_viz)
                        .block(Block::default().borders(Borders::ALL).title("Maze"));
                    f.render_widget(maze_widget, content_chunks[0]);

                    let log_height = content_chunks[1].height.saturating_sub(2) as usize;
                    let log_lines: Vec<Line> = logs
                        .iter()
                        .rev()
                        .take(log_height)
                        .map(|(msg, level)| {
                            let (color, prefix) = match level {
                                LogLevel::Info => (Color::Green, "INFO "),
                                LogLevel::Debug => (Color::Blue, "DEBUG"),
                                LogLevel::Warning => (Color::Yellow, "WARN "),
                                LogLevel::Error => (Color::Red, "ERROR"),
                            };
                            Line::from(vec![
                                Span::styled(prefix, Style::default().fg(color)),
                                Span::raw(" │ "),
                                Span::styled(msg, Style::default().fg(Color::White)),
                            ])
                        })
                        .collect();

                    let logs_widget = Paragraph::new(log_lines)
                        .block(Block::default().borders(Borders::ALL).title("Logs"));
                    f.render_widget(logs_widget, content_chunks[1]);
                }

                if let Some((found, total)) = result {
                    Self::render_finished_popup(f, found, total);
                }
            })
            .map_err(backend_err)?;

        Ok(())
    }

    fn render_finished_popup(f: &mut Frame, found: usize, total: usize) {
        let (summary, color) = if found == total {
            ("All agents found the exit!".to_string(), Color::Green)
        } else {
            (format!("{}/{} agents found the exit", found, total), Color::Yellow)
        };

        let area = f.area().centered(Constraint::Length(40), Constraint::Length(5));
        let popup = Paragraph::new(vec![
            Line::styled(summary, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Line::from(""),
            Line::from("Press q to quit"),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Game finished"));

        f.render_widget(Clear, area);
        f.render_widget(popup, area);
    }

    fn create_stats(&self, agent: &AgentState) -> (f64, String) {
        let graph = &agent.graph;
        let cells = &graph.cell_map;

        let total_cells = cells.len();
        let visited_cells =
            cells.values().filter(|cell| cell.status == CellStatus::VISITED).count();
        let dead_ends = cells.values().filter(|cell| cell.status == CellStatus::DeadEnd).count();
        let not_visited =
            cells.values().filter(|cell| cell.status == CellStatus::NotVisited).count();

        let explored =
            if total_cells > 0 { visited_cells as f64 / total_cells as f64 } else { 0.0 };
        let text = format!(
            "Pos: [{}, {}] | Visited: {}/{} | Dead ends: {} | Not visited: {}",
            agent.player.position.row,
            agent.player.position.column,
            visited_cells,
            total_cells,
            dead_ends,
            not_visited,
        );
        (explored, text)
    }

    fn calculate_bounds(cells: &HashMap<Cell, MazeCell>) -> (i16, i16, i16, i16) {
        cells.keys().fold(
            (i16::MAX, i16::MIN, i16::MAX, i16::MIN),
            |(min_row, max_row, min_col, max_col), cell| {
                (
                    min_row.min(cell.row),
                    max_row.max(cell.row),
                    min_col.min(cell.column),
                    max_col.max(cell.column),
                )
            },
        )
    }

    fn calculate_view_bounds(
        bounds: (i16, i16, i16, i16),
        player: &Player,
        available_width: i16,
        available_height: i16,
    ) -> (i16, i16, i16, i16) {
        let width = (available_width / 4).max(1);
        let height = (available_height / 2).max(1);

        let total_rows = if bounds.1 >= bounds.0 {
            bounds.1.saturating_sub(bounds.0).saturating_add(1)
        } else {
            0
        };
        let total_cols = if bounds.3 >= bounds.2 {
            bounds.3.saturating_sub(bounds.2).saturating_add(1)
        } else {
            0
        };

        let view_height = height.min(total_rows);
        let view_width = width.min(total_cols);

        let row_start = if total_rows <= view_height {
            bounds.0
        } else {
            let center_offset = view_height.saturating_div(2);
            let player_offset = player.position.row.saturating_sub(bounds.0);
            let max_start = total_rows.saturating_sub(view_height);

            bounds.0.saturating_add(player_offset.saturating_sub(center_offset).clamp(0, max_start))
        };

        let col_start = if total_cols <= view_width {
            bounds.2
        } else {
            let center_offset = view_width / 2;
            bounds.2.saturating_add(
                (player.position.column - bounds.2)
                    .saturating_sub(center_offset)
                    .clamp(0, total_cols.saturating_sub(view_width)),
            )
        };

        let row_end = row_start.saturating_add(view_height.saturating_sub(1)).min(bounds.1);
        let col_end = col_start.saturating_add(view_width.saturating_sub(1)).min(bounds.3);

        (row_start, row_end, col_start, col_end)
    }

    fn render_cell(cell: &MazeCell) -> Span<'static> {
        let (symbol, style) = match cell.cell_type {
            CellType::OBJECTIVE | CellType::ObjectiveAlly | CellType::ObjectiveMonster => {
                (" ★ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            }
            CellType::ObjectiveEnemy => {
                (" ★ ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            }
            CellType::HELP => {
                (" ? ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            }
            CellType::ALLY => (" ● ", Style::default().fg(Color::Green)),
            CellType::ENEMY => (" ● ", Style::default().fg(Color::Red)),
            CellType::MONSTER => (" M ", Style::default().fg(Color::Magenta)),
            _ => match cell.status {
                CellStatus::VISITED => (" · ", Style::default().fg(Color::Blue)),
                CellStatus::DeadEnd => (" × ", Style::default().fg(Color::DarkGray)),
                CellStatus::NotVisited => ("   ", Style::default()),
            },
        };
        Span::styled(symbol, style)
    }

    /// The agent, pointing where it heads. Screen columns follow the graph's columns and
    /// `Front` moves toward lower columns.
    fn player_span(direction: &shared::messages::Direction) -> Span<'static> {
        use shared::messages::Direction as Heading;
        let symbol = match direction {
            Heading::Front => " ◄ ",
            Heading::Right => " ▼ ",
            Heading::Back => " ► ",
            Heading::Left => " ▲ ",
        };
        Span::styled(symbol, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    }

    fn render_horizontal_wall(
        cells: &HashMap<Cell, MazeCell>,
        row: i16,
        col: i16,
        col_end: i16,
    ) -> String {
        let mut wall = String::new();
        wall.push_str("    │");

        for current_col in col..=col_end {
            let pos = Cell { row, column: current_col };
            let below_pos = Cell { row: row + 1, column: current_col };

            if let Some(cell) = cells.get(&pos) {
                if !cell.neighbors.contains(&below_pos) {
                    wall.push_str("───");
                } else {
                    wall.push_str("   ");
                }

                let right_pos = Cell { row, column: current_col + 1 };
                if current_col < col_end {
                    if !cell.neighbors.contains(&right_pos) {
                        if !cell.neighbors.contains(&below_pos) {
                            wall.push('┼');
                        } else {
                            wall.push('│');
                        }
                    } else if !cell.neighbors.contains(&below_pos) {
                        wall.push('─');
                    } else {
                        wall.push(' ');
                    }
                }
            } else {
                wall.push_str("   ");
                if current_col < col_end {
                    wall.push(' ');
                }
            }
        }
        wall.push('\n');
        wall
    }

    fn create_maze_visualization(
        &self,
        graph: &MazeGraph,
        player: &Player,
        available_width: i16,
        available_height: i16,
    ) -> Vec<Line<'static>> {
        let wall = Style::default().fg(Color::DarkGray);
        let cells = &graph.cell_map;

        let bounds = Self::calculate_bounds(cells);
        let (row_start, row_end, col_start, col_end) =
            Self::calculate_view_bounds(bounds, player, available_width, available_height);

        let mut lines = Vec::new();
        let top = "────".repeat((col_start..=col_end).count());
        lines.push(Line::from(vec![Span::raw("    "), Span::styled(top, wall)]));

        for row in row_start..=row_end {
            let label = match row {
                r if r % 5 == 0 && r <= -100 => format!("{:3}", row),
                r if r % 5 == 0 => format!("{:3} ", row),
                _ => "    ".to_string(),
            };
            let mut spans = vec![Span::raw(label), Span::styled("│", wall)];

            for col in col_start..=col_end {
                let pos = Cell { row, column: col };
                let Some(cell) = cells.get(&pos) else {
                    spans.push(Span::raw(if col < col_end { "    " } else { "   " }));
                    continue;
                };
                spans.push(if pos == player.position {
                    Self::player_span(&player.direction)
                } else {
                    Self::render_cell(cell)
                });
                let right_pos = Cell { row, column: col + 1 };
                spans.push(if !cell.neighbors.contains(&right_pos) && col < col_end {
                    Span::styled("│", wall)
                } else {
                    Span::raw(" ")
                });
            }
            lines.push(Line::from(spans));

            if row < row_end {
                let below = Self::render_horizontal_wall(cells, row, col_start, col_end);
                lines.push(Line::styled(below.trim_end_matches('\n').to_string(), wall));
            }
        }

        lines
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        if let Err(e) = self.exit() {
            eprintln!("Failed to exit TUI: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::{logger::LogLevel, maze::Cell, messages::Direction, radar::CellType};

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

        game_state.add_log("agent1", "Test log".to_string(), LogLevel::Info);
        game_state.add_log("agent1", "Warning log".to_string(), LogLevel::Warning);

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

        let player = Player {
            position: cell,
            direction: shared::messages::Direction::Front,
            saved_state: None,
        };

        game_state.update_state("agent1", graph.clone(), player.clone());

        let agent_state = &game_state.agents["agent1"];
        assert_eq!(agent_state.player.position, cell);
        assert!(agent_state.graph.contains(&cell));
    }

    #[test]
    fn test_select_agent_increment() {
        let mut game_state = GameState::new();
        game_state.register_agent("agent1".to_string());
        game_state.register_agent("agent2".to_string());

        assert_eq!(game_state.selected_tab, 0);
        Tui::select_agent_increment(&mut game_state, 2);
        assert_eq!(game_state.selected_tab, 1);
        Tui::select_agent_increment(&mut game_state, 2);
        assert_eq!(game_state.selected_tab, 0);
    }

    #[test]
    fn test_select_agent_decrement() {
        let mut game_state = GameState::new();
        game_state.register_agent("agent1".to_string());
        game_state.register_agent("agent2".to_string());
        game_state.selected_tab = 1;

        assert_eq!(game_state.selected_tab, 1);
        Tui::select_agent_decrement(&mut game_state, 2);
        assert_eq!(game_state.selected_tab, 0);
        Tui::select_agent_decrement(&mut game_state, 2);
        assert_eq!(game_state.selected_tab, 1);
    }

    #[test]
    fn test_maze_visualization_empty() {
        let tui = Tui::new(100).unwrap();
        let graph = MazeGraph::new();
        let player = Player::new();

        let viz = tui.create_maze_visualization(&graph, &player, 10, 10);
        assert_eq!(lines_text(&viz), "    \n");
    }

    #[test]
    fn test_maze_visualization_2x2_grid() {
        let tui = Tui::new(100).unwrap();
        let mut graph = MazeGraph::new();
        let cells = [
            (0, 0, CellType::NOTHING),
            (0, 1, CellType::NOTHING),
            (1, 0, CellType::NOTHING),
            (1, 1, CellType::NOTHING),
        ];

        for (row, col, cell_type) in cells.iter() {
            let cell = Cell { row: *row, column: *col };
            graph.add(cell, cell_type.clone());
        }

        let player = Player {
            position: Cell { row: 0, column: 0 },
            direction: Direction::Front,
            saved_state: None,
        };

        let viz = tui.create_maze_visualization(&graph, &player, 10, 10);
        let expected = "    ────────\n  0 │ ◄ │    \n    │───┼───\n    │   │    \n";
        assert_eq!(lines_text(&viz), expected);
    }

    #[test]
    fn test_calculate_bounds() {
        let mut cells = HashMap::new();

        assert_eq!(Tui::calculate_bounds(&cells), (i16::MAX, i16::MIN, i16::MAX, i16::MIN));

        cells.insert(Cell { row: 0, column: 0 }, MazeCell::new(CellType::NOTHING));
        assert_eq!(Tui::calculate_bounds(&cells), (0, 0, 0, 0));

        cells.insert(Cell { row: -1, column: 2 }, MazeCell::new(CellType::NOTHING));
        cells.insert(Cell { row: 3, column: -2 }, MazeCell::new(CellType::NOTHING));
        assert_eq!(Tui::calculate_bounds(&cells), (-1, 3, -2, 2));
    }

    #[test]
    fn test_calculate_view_bounds() {
        let bounds = (-2, 2, -3, 3); // 5x7 maze
        let player = Player {
            position: Cell { row: 0, column: 0 },
            direction: Direction::Front,
            saved_state: None,
        };

        let (row_start, row_end, col_start, col_end) =
            Tui::calculate_view_bounds(bounds, &player, 40, 20);
        assert_eq!(row_start, -2);
        assert_eq!(row_end, 2);
        assert_eq!(col_start, -3);
        assert_eq!(col_end, 3);

        let (row_start, row_end, col_start, col_end) =
            Tui::calculate_view_bounds(bounds, &player, 12, 6);
        assert_eq!(row_start, -1);
        assert_eq!(row_end, 1);
        assert_eq!(col_start, -1);
        assert_eq!(col_end, 1);

        let edge_player = Player {
            position: Cell { row: 2, column: 3 },
            direction: Direction::Front,
            saved_state: None,
        };
        let (row_start, row_end, col_start, col_end) =
            Tui::calculate_view_bounds(bounds, &edge_player, 12, 6);
        assert_eq!(row_start, 0);
        assert_eq!(row_end, 2);
        assert_eq!(col_start, 1);
        assert_eq!(col_end, 3);
    }

    #[test]
    fn test_render_horizontal_wall() {
        let mut cells = HashMap::new();

        let wall = Tui::render_horizontal_wall(&cells, 0, 0, 2);
        assert_eq!(wall, "    │           \n");

        let mut cell1 = MazeCell::new(CellType::NOTHING);
        let mut cell2 = MazeCell::new(CellType::NOTHING);

        cell1.neighbors.insert(Cell { row: 0, column: 1 }); // Right neighbor
        cell1.neighbors.insert(Cell { row: 1, column: 0 }); // Bottom neighbor
        cell2.neighbors.insert(Cell { row: 0, column: 0 }); // Left neighbor
        cell2.neighbors.insert(Cell { row: 1, column: 1 }); // Bottom neighbor

        cells.insert(Cell { row: 0, column: 0 }, cell1);
        cells.insert(Cell { row: 0, column: 1 }, cell2);

        let wall = Tui::render_horizontal_wall(&cells, 0, 0, 1);
        assert_eq!(wall, "    │       \n");

        cells.clear();
        let cell1 = MazeCell::new(CellType::NOTHING);
        let cell2 = MazeCell::new(CellType::NOTHING);

        cells.insert(Cell { row: 0, column: 0 }, cell1);
        cells.insert(Cell { row: 0, column: 1 }, cell2);

        let wall = Tui::render_horizontal_wall(&cells, 0, 0, 1);
        assert_eq!(wall, "    │───┼───\n");
    }

    fn lines_text(lines: &[Line]) -> String {
        lines
            .iter()
            .map(|line| {
                line.spans.iter().map(|span| span.content.as_ref()).collect::<String>() + "\n"
            })
            .collect()
    }

    #[test]
    fn test_render_cell() {
        let symbol = |cell_type: CellType, status: CellStatus| {
            let mut cell = MazeCell::new(cell_type);
            cell.status = status;
            Tui::render_cell(&cell).content.into_owned()
        };

        assert_eq!(symbol(CellType::NOTHING, CellStatus::NotVisited), "   ");
        assert_eq!(symbol(CellType::NOTHING, CellStatus::DeadEnd), " × ");
        assert_eq!(symbol(CellType::OBJECTIVE, CellStatus::NotVisited), " ★ ");
        assert_eq!(symbol(CellType::ObjectiveEnemy, CellStatus::NotVisited), " ★ ");
    }

    #[test]
    fn test_player_symbol_shows_heading() {
        assert_eq!(Tui::player_span(&Direction::Front).content, " ◄ ");
        assert_eq!(Tui::player_span(&Direction::Right).content, " ▼ ");
        assert_eq!(Tui::player_span(&Direction::Back).content, " ► ");
        assert_eq!(Tui::player_span(&Direction::Left).content, " ▲ ");
    }

    #[test]
    fn test_tabs_mark_agents_that_found_the_exit() {
        let text = rendered_text(false, &["Player2"]);
        assert!(text.contains("Player2 ✓"));
        assert!(!text.contains("Player1 ✓"));
        assert!(text.find("Player1").unwrap() < text.find("Player2").unwrap());
    }

    #[test]
    fn test_statistics_show_explored_progress() {
        assert!(rendered_text(false, &[]).contains("0% explored"));
    }

    fn rendered_text(finished: bool, exits_found: &[&str]) -> String {
        let Ok(terminal) = Terminal::new(ratatui::backend::TestBackend::new(80, 24));
        let state = Arc::new(Mutex::new(GameState::new()));
        {
            let mut state = state.lock().unwrap();
            state.register_agent("Player1".to_string());
            state.register_agent("Player2".to_string());
            for agent in exits_found {
                state.mark_exit_found(agent);
            }
            if finished {
                state.mark_finished();
            }
        }

        let mut tui = Tui { terminal, state, refresh_rate: 0 };
        tui.draw().unwrap();
        tui.terminal.backend().buffer().content().iter().map(|c| c.symbol()).collect()
    }

    #[test]
    fn test_finished_popup_hidden_while_game_running() {
        assert!(!rendered_text(false, &["Player1", "Player2"]).contains("Game finished"));
    }

    #[test]
    fn test_finished_popup_when_all_agents_found_exit() {
        let text = rendered_text(true, &["Player1", "Player2"]);
        assert!(text.contains("All agents found the exit!"));
        assert!(text.contains("Press q to quit"));
    }

    #[test]
    fn test_finished_popup_reports_partial_result() {
        let text = rendered_text(true, &["Player2"]);
        assert!(text.contains("1/2 agents found the exit"));
        assert!(!text.contains("All agents found the exit!"));
    }
}
