use crate::data_structures::maze_graph::{CellStatus, MazeGraph};
use crate::maze_parser::Player;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Terminal,
};
use std::time::Instant;
use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

const UPDATE_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Clone)]
pub enum LogLevel {
    Info,
    Debug,
    Warning,
    Error,
}

pub struct AgentState {
    pub logs: Vec<(String, LogLevel)>,
    pub graph: MazeGraph,
    pub player: Player,
    pub last_update: Instant,
}

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentState {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            graph: MazeGraph::new(),
            player: Player::new(),
            last_update: Instant::now(),
        }
    }
}

pub struct AppState {
    agents: HashMap<String, AgentState>,
    selected_tab: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self { agents: HashMap::new(), selected_tab: 0 }
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
            let now = Instant::now();
            if now.duration_since(state.last_update) >= UPDATE_INTERVAL {
                state.graph = graph;
                state.player = player.clone();
                state.last_update = now;
            }
        }
    }

    pub fn needs_update(&self, agent: &str) -> bool {
        self.agents.get(agent).is_some_and(|state| state.last_update.elapsed() >= UPDATE_INTERVAL)
    }
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: Arc<Mutex<AppState>>,
}

impl Tui {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        let state = Arc::new(Mutex::new(AppState::new()));
        Ok(Self { terminal, state })
    }

    pub fn get_state(&self) -> Arc<Mutex<AppState>> {
        Arc::clone(&self.state)
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut last_draw = Instant::now();

        loop {
            if last_draw.elapsed() >= UPDATE_INTERVAL {
                self.draw()?;
                last_draw = Instant::now();
            }

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if let KeyCode::Char('q') = key.code {
                        break;
                    }

                    let mut state = self.state.lock().unwrap();
                    match key.code {
                        KeyCode::Right => {
                            let agent_count = state.agents.len();
                            if agent_count > 0 {
                                state.selected_tab = (state.selected_tab + 1) % agent_count;
                            }
                        }
                        KeyCode::Left => {
                            if state.selected_tab > 0 {
                                state.selected_tab -= 1;
                            }
                        }
                        _ => {}
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    }

    fn draw(&mut self) -> io::Result<()> {
        let terminal_size = self.terminal.size()?;
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

        let state = self.state.lock().unwrap();
        let agent_names: Vec<String> = state.agents.keys().cloned().collect();
        let selected_tab = state.selected_tab;

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
                (String::new(), None)
            }
        } else {
            (String::new(), None)
        };

        drop(state);

        self.terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(10)])
                .split(size);

            let tabs = Tabs::new(
                agent_names
                    .iter()
                    .enumerate()
                    .map(|(i, name)| {
                        if i == selected_tab {
                            Line::from(vec![Span::styled(
                                name.trim(),
                                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                            )])
                        } else {
                            Line::from(vec![Span::styled(name, Style::default().fg(Color::Green))])
                        }
                    })
                    .collect::<Vec<_>>(),
            )
            .block(Block::default().borders(Borders::ALL).title("Agents"))
            .select(selected_tab);

            f.render_widget(tabs, chunks[0]);

            if let Some((stats, logs)) = agent_data {
                let stats_widget = Paragraph::new(stats)
                    .block(Block::default().borders(Borders::ALL).title("Statistics"));
                f.render_widget(stats_widget, chunks[1]);

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
        })?;

        Ok(())
    }

    fn create_stats(&self, agent: &AgentState) -> String {
        let graph = &agent.graph;
        let cells = &graph.cell_map;

        let total_cells = cells.len();
        let visited_cells =
            cells.values().filter(|cell| cell.status == CellStatus::VISITED).count();
        let dead_ends = cells.values().filter(|cell| cell.status == CellStatus::DeadEnd).count();
        let not_visited =
            cells.values().filter(|cell| cell.status == CellStatus::NotVisited).count();

        let explored_percent = if total_cells > 0 {
            (visited_cells as f64 / total_cells as f64 * 100.0) as u32
        } else {
            0
        };

        format!(
            "Pos: [{}, {}] | Explored: {}% ({}/{}) | Dead ends: {} | Not visited: {}",
            agent.player.position.row,
            agent.player.position.column,
            explored_percent,
            visited_cells,
            total_cells,
            dead_ends,
            not_visited,
        )
    }

    fn create_maze_visualization(
        &self,
        graph: &MazeGraph,
        player: &Player,
        available_width: i16,
        available_height: i16,
    ) -> String {
        let mut visualization = String::new();
        let cells = &graph.cell_map;

        let bounds = cells.keys().fold(
            (i16::MAX, i16::MIN, i16::MAX, i16::MIN),
            |(min_row, max_row, min_col, max_col), cell| {
                (
                    min_row.min(cell.row),
                    max_row.max(cell.row),
                    min_col.min(cell.column),
                    max_col.max(cell.column),
                )
            },
        );

        let width = (available_width / 4).max(1);
        let height = (available_height / 2).max(1);

        let total_rows = bounds.1 - bounds.0 + 1;
        let total_cols = bounds.3 - bounds.2 + 1;

        let view_height = height.min(total_rows);
        let view_width = width.min(total_cols);

        let row_start = if total_rows <= view_height {
            bounds.0
        } else {
            let center_offset = view_height / 2;
            (player.position.row - center_offset).max(bounds.0).min(bounds.1 - view_height + 1)
        };

        let col_start = if total_cols <= view_width {
            bounds.2
        } else {
            let center_offset = view_width / 2;
            (player.position.column - center_offset).max(bounds.2).min(bounds.3 - view_width + 1)
        };

        let row_end = (row_start + view_height - 1).min(bounds.1);
        let col_end = (col_start + view_width - 1).min(bounds.3);

        visualization.push_str("    ");
        for _col in col_start..=col_end {
            visualization.push_str("───");
        }
        visualization.push('\n');

        for row in row_start..=row_end {
            if row % 5 == 0 {
                visualization.push_str(&format!("{:3} │", row));
            } else {
                visualization.push_str("    │");
            }

            for col in col_start..=col_end {
                let pos = shared::maze::Cell { row, column: col };

                if pos == player.position {
                    visualization.push_str(" 🔵 ");
                } else if let Some(cell) = cells.get(&pos) {
                    use shared::radar::CellType;

                    match cell.cell_type {
                        CellType::OBJECTIVE => visualization.push_str(" ✅ "),
                        CellType::ENEMY => visualization.push_str(" ⚠️ "),
                        CellType::HELP => visualization.push_str(" 🆘 "),
                        CellType::NOTHING => match cell.status {
                            CellStatus::VISITED => visualization.push_str(" · "),
                            CellStatus::DeadEnd => visualization.push_str(" 🔸"),
                            CellStatus::NotVisited => visualization.push_str("   "),
                        },
                        _ => visualization.push_str(" # "),
                    }

                    let right_pos = shared::maze::Cell { row, column: col + 1 };
                    if !cell.neighbors.contains(&right_pos) && col < col_end {
                        visualization.push('│');
                    } else {
                        visualization.push(' ');
                    }
                } else {
                    visualization.push_str("   ");
                    if col < col_end {
                        visualization.push(' ');
                    }
                }
            }
            visualization.push('\n');

            if row < row_end {
                visualization.push_str("    │");
                for col in col_start..=col_end {
                    let pos = shared::maze::Cell { row, column: col };
                    let below_pos = shared::maze::Cell { row: row + 1, column: col };

                    if let Some(cell) = cells.get(&pos) {
                        if !cell.neighbors.contains(&below_pos) {
                            visualization.push_str("───");
                        } else {
                            visualization.push_str("   ");
                        }

                        let right_pos = shared::maze::Cell { row, column: col + 1 };
                        if col < col_end {
                            if !cell.neighbors.contains(&right_pos) {
                                if !cell.neighbors.contains(&below_pos) {
                                    visualization.push('┼');
                                } else {
                                    visualization.push('│');
                                }
                            } else if !cell.neighbors.contains(&below_pos) {
                                visualization.push('─');
                            } else {
                                visualization.push(' ');
                            }
                        }
                    } else {
                        visualization.push_str("   ");
                        if col < col_end {
                            visualization.push(' ');
                        }
                    }
                }
                visualization.push('\n');
            }
        }

        visualization
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen,).unwrap();
    }
}
