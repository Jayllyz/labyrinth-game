use crate::data_structures::maze_graph::{CellStatus, MazeGraph};
use crate::maze_parser::Player;
use core::time;
use shared::maze::Cell;
use shared::messages::{self, Direction};
use shared::radar::{CellType, Passages, Radar};
use std::collections::{HashMap, HashSet, VecDeque};
use std::thread::sleep;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
#[value(rename_all = "PascalCase")]
pub enum Algorithm {
    Tremeaux,
    WallFollower,
    Alian,
}

// TODO:
// [X] For each currently visited maze node, there is occupied flag is set to true so that other agents know they cannot
// go there.
// [X] If there is a neighboring maze node that has not yet been visited by any agent (white), the agent should go there.
// [X] If there are several such nodes, the agent should choose one at random.
// [X] If there is no node that has not been visited by any agent yet, the agent should prefer a node that has not been
// visited by it yet.
// [X] If there are several such nodes, the agent should choose one at random.
// [X] If the agent has already traveled in all possible directions, the agent will choose the node that has been visited
// the least by it.
// [ ] If there is only one neighboring maze node that can be visited, but it is currently occupied, the agent should
// wait at its current maze node until it becomes free
// [X] If there is only one way to leave the current node, and all other directional are obstacles or black nodes, the
// agent marks the current node as a black node so that other agents cannot move there.

/// The parent to backtrack to, or any open neighbor when the parent is not adjacent
/// (unset or stale), since walking toward a non-neighbor bumps into a wall forever.
fn backtrack_target(neighbors: &HashSet<Cell>, parent: Cell) -> Cell {
    if neighbors.contains(&parent) {
        return parent;
    }
    neighbors.iter().next().copied().unwrap_or(parent)
}

pub fn alian_solver(
    player: &mut Player,
    graph: &mut MazeGraph,
    thread_name: &str,
) -> messages::Action {
    let mut message: messages::Action = messages::Action::MoveTo(messages::Direction::Front);

    let (neighbor_positions, parent, player_position) = {
        let Some(player_cell) = graph.get_cell(player.position) else {
            return messages::Action::MoveTo(messages::Direction::Front);
        };
        (player_cell.neighbors.clone(), player_cell.parent, player.position)
    };

    let mut visited: Vec<Cell> = Vec::new();
    let mut visited_by_self: Vec<(Cell, u8)> = Vec::new();
    let mut not_visited_by_self: Vec<Cell> = Vec::new();

    let mut dead_ends_cell = 0;
    for pos in &neighbor_positions {
        let status = graph.get_cell_status(*pos);
        if status == CellStatus::DeadEnd {
            dead_ends_cell += 1;
        }
    }

    let player_walls = graph.get_cell(player_position).map_or(0, |cell| cell.walls);
    if player_walls + dead_ends_cell == 3 {
        graph.update_cell_status(player_position, CellStatus::DeadEnd);
    }

    for &neighbor_position in &neighbor_positions {
        let (walls, mut status, cell_type, visited_by) = {
            let Some(neighbor_cell) = graph.get_cell(neighbor_position) else {
                continue;
            };
            (
                neighbor_cell.walls,
                neighbor_cell.status.clone(),
                neighbor_cell.cell_type.clone(),
                neighbor_cell.visited_by.clone(),
            )
        };

        if walls == 3
            && !(cell_type == CellType::OBJECTIVE
                || cell_type == CellType::HELP
                || cell_type == CellType::ObjectiveAlly
                || cell_type == CellType::ObjectiveEnemy
                || cell_type == CellType::ObjectiveMonster)
        {
            graph.update_cell_status(neighbor_position, CellStatus::DeadEnd);
            status = CellStatus::DeadEnd;
        }

        if status == CellStatus::NotVisited {
            if cell_type == CellType::ENEMY {
                let five_millis: std::time::Duration = time::Duration::from_millis(500);
                sleep(five_millis);
                graph.update_cell_type(neighbor_position, CellType::NOTHING)
            }
            graph.update_cell_status(player_position, CellStatus::VISITED);
            graph.set_visited(player_position, thread_name);
            let next_direction = player.get_next_direction(&neighbor_position);

            message = match next_direction {
                Direction::Left => {
                    player.turn_left();
                    messages::Action::MoveTo(messages::Direction::Left)
                }
                Direction::Right => {
                    player.turn_right();
                    messages::Action::MoveTo(messages::Direction::Right)
                }
                Direction::Back => {
                    player.turn_back();
                    messages::Action::MoveTo(messages::Direction::Back)
                }
                _ => messages::Action::MoveTo(messages::Direction::Front),
            };

            graph.set_parent(neighbor_position, player_position);
            player.move_forward();
            return message;
        }

        if status == CellStatus::VISITED {
            visited.push(neighbor_position);

            if !visited_by.contains_key(thread_name) {
                not_visited_by_self.push(neighbor_position);
            } else if let Some(&count) = visited_by.get(thread_name) {
                visited_by_self.push((neighbor_position, count));
            }
        }
    }

    // Trie ascendant
    visited_by_self.sort_by_key(|a| a.1);

    let parent_status = graph.get_cell_status(parent);
    let next_direction = if (parent_status == CellStatus::DeadEnd || parent == player_position)
        && !not_visited_by_self.is_empty()
    {
        player.get_next_direction(&not_visited_by_self[0])
    } else if (parent_status == CellStatus::DeadEnd || parent == player_position)
        && !visited.is_empty()
    {
        player.get_next_direction(&visited_by_self[0].0)
    } else {
        player.get_next_direction(&backtrack_target(&neighbor_positions, parent))
    };

    message = match next_direction {
        Direction::Left => {
            player.turn_left();
            messages::Action::MoveTo(messages::Direction::Left)
        }
        Direction::Right => {
            player.turn_right();
            messages::Action::MoveTo(messages::Direction::Right)
        }
        Direction::Back => {
            player.turn_back();
            messages::Action::MoveTo(messages::Direction::Back)
        }
        _ => message,
    };

    graph.update_cell_status(player_position, CellStatus::DeadEnd);
    player.move_forward();
    message
}

/// First step of the shortest known path to the nearest unexplored cell.
///
/// Backtracking along parents breaks once the agent is pushed off its path (by an
/// opponent), so the path goes through every known cell, dead ends included, except
/// `avoid` (a cell an opponent holds).
fn first_step_to_unexplored(graph: &MazeGraph, from: Cell, avoid: Option<Cell>) -> Option<Cell> {
    let mut came_from: HashMap<Cell, Cell> = HashMap::from([(from, from)]);
    let mut queue = VecDeque::from([from]);

    while let Some(cell) = queue.pop_front() {
        for &next in &graph.cell_map.get(&cell)?.neighbors {
            if came_from.contains_key(&next) || !graph.contains(&next) || Some(next) == avoid {
                continue;
            }
            came_from.insert(next, cell);
            if graph.get_cell_status(next) == CellStatus::NotVisited {
                let mut step = next;
                while came_from[&step] != from {
                    step = came_from[&step];
                }
                return Some(step);
            }
            queue.push_back(next);
        }
    }
    None
}

pub fn tremeaux_solver(
    player: &mut Player,
    graph: &mut MazeGraph,
    avoid: Option<Cell>,
) -> messages::Action {
    let mut message: messages::Action = messages::Action::MoveTo(messages::Direction::Front);

    let (neighbor_positions, parent) = {
        let Some(player_cell) = graph.get_cell(player.position) else {
            return messages::Action::MoveTo(messages::Direction::Front);
        };
        (player_cell.neighbors.clone(), player_cell.parent)
    };

    let mut visited: Vec<Cell> = Vec::new();

    for &neighbor_position in &neighbor_positions {
        if Some(neighbor_position) == avoid {
            continue;
        }
        let (walls, mut status, cell_type) = {
            let Some(neighbor_cell) = graph.get_cell(neighbor_position) else {
                continue;
            };
            (neighbor_cell.walls, neighbor_cell.status.clone(), neighbor_cell.cell_type.clone())
        };

        if walls == 3
            && !(cell_type == CellType::OBJECTIVE
                || cell_type == CellType::HELP
                || cell_type == CellType::ObjectiveAlly
                || cell_type == CellType::ObjectiveEnemy
                || cell_type == CellType::ObjectiveMonster)
        {
            graph.update_cell_status(neighbor_position, CellStatus::DeadEnd);
            status = CellStatus::DeadEnd;
        }

        if status == CellStatus::NotVisited {
            graph.update_cell_status(player.position, CellStatus::VISITED);
            let next_direction = player.get_next_direction(&neighbor_position);

            message = match next_direction {
                Direction::Left => {
                    player.turn_left();
                    messages::Action::MoveTo(messages::Direction::Left)
                }
                Direction::Right => {
                    player.turn_right();
                    messages::Action::MoveTo(messages::Direction::Right)
                }
                Direction::Back => {
                    player.turn_back();
                    messages::Action::MoveTo(messages::Direction::Back)
                }
                _ => messages::Action::MoveTo(messages::Direction::Front),
            };

            graph.set_parent(neighbor_position, player.position);
            player.move_forward();
            return message;
        }

        if status == CellStatus::VISITED {
            visited.push(neighbor_position);
        }
    }

    let parent_status = graph.get_cell_status(parent);
    let target = if let Some(step) = first_step_to_unexplored(graph, player.position, avoid) {
        step
    } else if (parent_status == CellStatus::DeadEnd || parent == player.position)
        && !visited.is_empty()
    {
        visited[0]
    } else {
        backtrack_target(&neighbor_positions, parent)
    };
    let next_direction = player.get_next_direction(&target);

    message = match next_direction {
        Direction::Left => {
            player.turn_left();
            messages::Action::MoveTo(messages::Direction::Left)
        }
        Direction::Right => {
            player.turn_right();
            messages::Action::MoveTo(messages::Direction::Right)
        }
        Direction::Back => {
            player.turn_back();
            messages::Action::MoveTo(messages::Direction::Back)
        }
        _ => message,
    };

    graph.update_cell_status(player.position, CellStatus::DeadEnd);
    player.move_forward();
    message
}

/// Steps away from a cell an opponent is holding, preferring the way we came, so the
/// opponent can pass.
///
/// Retreating to the parent reopens the current cell so the solver comes back to it
/// later. Retreating sideways into an unexplored cell links it back to the current one,
/// so backtracking from it returns here.
pub fn yield_to_opponent(
    player: &mut Player,
    graph: &mut MazeGraph,
    blocked: Cell,
) -> Option<messages::Action> {
    let position = player.position;
    let cell = graph.get_cell(position)?;
    let is_open = |c: &Cell| *c != blocked && *c != position;

    let retreat = if cell.neighbors.contains(&cell.parent) && is_open(&cell.parent) {
        if cell.status == CellStatus::VISITED {
            cell.status = CellStatus::NotVisited;
        }
        cell.parent
    } else {
        let side = *cell.neighbors.iter().find(|c| is_open(c))?;
        if let Some(side_cell) = graph.get_cell(side)
            && side_cell.status == CellStatus::NotVisited
        {
            side_cell.parent = position;
        }
        side
    };

    Some(player.step_toward(&retreat))
}

/// Where a wall follower has already been, to notice when it circles a loop of walls.
#[derive(Default)]
pub struct WallFollowerState {
    seen: HashSet<(Cell, Direction)>,
    looping: bool,
}

/// Follows the right-hand wall until the agent is back on a cell facing the same way.
/// In a maze with loops that means it circles walls not connected to the exit forever,
/// so it explores with Trémaux from then on.
pub fn wall_follower_solver(
    radar_view: &Radar,
    player: &mut Player,
    graph: &mut MazeGraph,
    state: &mut WallFollowerState,
    avoid: Option<Cell>,
) -> messages::Action {
    if !state.looping && !state.seen.insert((player.position, player.direction.clone())) {
        state.looping = true;
    }
    if state.looping {
        return tremeaux_solver(player, graph, avoid);
    }
    right_hand_solver(radar_view, player)
}

pub fn right_hand_solver(radar_view: &Radar, player: &mut Player) -> messages::Action {
    let messages;

    if let Some(vertical) = radar_view.vertical.get(6)
        && *vertical == Passages::OPEN
    {
        messages = messages::Action::MoveTo(messages::Direction::Right);
        player.turn_right();
        player.move_forward();
        return messages;
    }

    if let Some(horizontal) = radar_view.horizontal.get(4)
        && *horizontal == Passages::OPEN
    {
        messages = messages::Action::MoveTo(messages::Direction::Front);
        player.move_forward();
        return messages;
    }

    if let Some(vertical) = radar_view.vertical.get(5)
        && *vertical == Passages::OPEN
    {
        messages = messages::Action::MoveTo(messages::Direction::Left);
        player.turn_left();
        player.move_forward();
        return messages;
    }

    if let Some(horizontal) = radar_view.horizontal.get(7)
        && *horizontal == Passages::OPEN
    {
        messages = messages::Action::MoveTo(messages::Direction::Back);
        player.turn_back();
        player.move_forward();
        return messages;
    }

    messages::Action::MoveTo(messages::Direction::Right)
}

pub fn check_win_condition(cells: &[CellType], direction: messages::Action) -> bool {
    let index = match direction {
        messages::Action::MoveTo(messages::Direction::Right) => 5,
        messages::Action::MoveTo(messages::Direction::Left) => 3,
        messages::Action::MoveTo(messages::Direction::Front) => 1,
        messages::Action::MoveTo(messages::Direction::Back) => 7,
        _ => return false,
    };

    if let Some(cell) = cells.get(index)
        && (*cell == CellType::OBJECTIVE || *cell == CellType::ObjectiveAlly)
    {
        return true;
    }

    false
}

pub fn solve_sum_modulo<S: ::std::hash::BuildHasher>(
    secret_sum: u128,
    secrets: &HashMap<std::thread::ThreadId, u128, S>,
) -> String {
    (secrets.values().sum::<u128>() % secret_sum).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze_parser::maze_to_graph;
    use messages::RadarView;
    use shared::radar::{decode_base64, extract_data};
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    #[test]
    fn test_right_hand_solver() {
        let view = RadarView("swfGkIAyap8a8aa".to_owned());
        let mut player = Player::new();
        let radar_view = extract_data(&decode_base64(&view.0)).unwrap();
        let result = right_hand_solver(&radar_view, &mut player);
        assert!(matches!(result, messages::Action::MoveTo(messages::Direction::Right)));
    }

    #[test]
    fn test_tremeaux_solver() {
        let view = RadarView("begGkcIyap8p8pa".to_owned());
        let radar = extract_data(&decode_base64(&view.0)).unwrap();
        let mut player = Player::new();
        let mut graph = MazeGraph::new();
        maze_to_graph(&radar, &player, &mut graph);

        let result = tremeaux_solver(&mut player, &mut graph, None);

        assert!(matches!(
            result,
            messages::Action::MoveTo(messages::Direction::Front | messages::Direction::Back)
        ));
    }

    #[test]
    fn test_check_win_condition_right() {
        let cells = vec![
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::OBJECTIVE,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
        ];
        let direction = messages::Action::MoveTo(messages::Direction::Right);
        assert!(check_win_condition(&cells, direction));
    }

    #[test]
    fn test_check_win_condition_left() {
        let cells = vec![
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::OBJECTIVE,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
        ];
        let direction = messages::Action::MoveTo(messages::Direction::Left);
        assert!(check_win_condition(&cells, direction));
    }

    #[test]
    fn test_check_win_condition_front() {
        let cells = vec![
            CellType::NOTHING,
            CellType::OBJECTIVE,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
        ];
        let direction = messages::Action::MoveTo(messages::Direction::Front);
        assert!(check_win_condition(&cells, direction));
    }

    #[test]
    fn test_check_win_condition_back() {
        let cells = vec![
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::OBJECTIVE,
            CellType::NOTHING,
        ];
        let direction = messages::Action::MoveTo(messages::Direction::Back);
        assert!(check_win_condition(&cells, direction));
    }

    #[test]
    fn test_check_win_condition_no_objective() {
        let cells = vec![
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
            CellType::NOTHING,
        ];
        let direction = messages::Action::MoveTo(messages::Direction::Right);
        assert!(!check_win_condition(&cells, direction));
    }

    #[test]
    fn test_sum_modulo() {
        let secrets_arc = Arc::new(Mutex::new(HashMap::new()));
        let mut handles = vec![];

        let values = vec![2667360881372235285, 7064968778338382540, 8653237798568263501];

        for value in values {
            let secrets = Arc::<
                std::sync::Mutex<std::collections::HashMap<std::thread::ThreadId, u128>>,
            >::clone(&secrets_arc);
            let handle = std::thread::spawn(move || {
                let thread_id = std::thread::current().id();
                secrets.lock().unwrap().insert(thread_id, value);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let secrets = secrets_arc.lock().unwrap();

        let result = solve_sum_modulo(1524576388644652385, &secrets);
        assert_eq!(result, "90650794543052706");
    }

    const CURRENT: Cell = Cell { row: 5, column: 5 };
    const BLOCKED: Cell = Cell { row: 5, column: 4 };
    const BEHIND: Cell = Cell { row: 5, column: 6 };
    const SIDE: Cell = Cell { row: 6, column: 5 };

    fn graph_around_player(neighbors: &[Cell], parent: Cell) -> (Player, MazeGraph) {
        let mut graph = MazeGraph::new();
        graph.add(CURRENT, CellType::NOTHING);
        for neighbor in neighbors {
            graph.add(*neighbor, CellType::NOTHING);
            graph.add_neighbor(&CURRENT, neighbor);
        }
        graph.set_parent(CURRENT, parent);
        graph.update_cell_status(CURRENT, CellStatus::VISITED);

        let mut player = Player::new();
        player.position = CURRENT;
        (player, graph)
    }

    #[test]
    fn test_tremeaux_heads_back_to_unexplored_cell_after_being_pushed_aside() {
        // Pushed onto CURRENT, whose parent BEHIND leads away from the unexplored cell
        // still waiting next to SIDE.
        let unexplored = Cell { row: 7, column: 5 };
        let (mut player, mut graph) = graph_around_player(&[BEHIND, SIDE], BEHIND);
        graph.update_cell_status(BEHIND, CellStatus::VISITED);
        graph.update_cell_status(SIDE, CellStatus::VISITED);
        graph.add(unexplored, CellType::NOTHING);
        graph.add_neighbor(&SIDE, &unexplored);
        graph.add_neighbor(&unexplored, &SIDE);

        tremeaux_solver(&mut player, &mut graph, None);
        assert_eq!(player.position, SIDE);

        tremeaux_solver(&mut player, &mut graph, None);
        assert_eq!(player.position, unexplored);
    }

    #[test]
    fn test_tremeaux_routes_around_cell_held_by_opponent() {
        let unexplored = Cell { row: 7, column: 5 };
        let (mut player, mut graph) = graph_around_player(&[BLOCKED, SIDE], BEHIND);
        graph.update_cell_status(SIDE, CellStatus::VISITED);
        graph.add(unexplored, CellType::NOTHING);
        graph.add_neighbor(&SIDE, &unexplored);
        graph.add_neighbor(&unexplored, &SIDE);

        tremeaux_solver(&mut player, &mut graph, Some(BLOCKED));

        assert_eq!(player.position, SIDE);
    }

    #[test]
    fn test_wall_follower_switches_to_tremeaux_when_back_where_it_was() {
        let (mut player, mut graph) = graph_around_player(&[SIDE], BEHIND);
        let mut state = WallFollowerState::default();
        state.seen.insert((CURRENT, player.direction.clone()));
        let no_walls_known =
            Radar { horizontal: Vec::new(), vertical: Vec::new(), cells: Vec::new() };

        wall_follower_solver(&no_walls_known, &mut player, &mut graph, &mut state, None);

        assert_eq!(player.position, SIDE);
    }

    #[test]
    fn test_tremeaux_without_parent_moves_to_a_neighbor() {
        let (mut player, mut graph) = graph_around_player(&[SIDE], CURRENT);
        graph.update_cell_status(SIDE, CellStatus::DeadEnd);

        tremeaux_solver(&mut player, &mut graph, None);

        assert_eq!(player.position, SIDE);
    }

    #[test]
    fn test_alian_without_parent_moves_to_a_neighbor() {
        let (mut player, mut graph) = graph_around_player(&[SIDE], CURRENT);
        graph.update_cell_status(SIDE, CellStatus::DeadEnd);

        alian_solver(&mut player, &mut graph, "Player1");

        assert_eq!(player.position, SIDE);
    }

    #[test]
    fn test_tremeaux_heads_for_exit_held_by_opponent() {
        let (mut player, mut graph) = graph_around_player(&[BEHIND], BEHIND);
        graph.update_cell_status(BEHIND, CellStatus::VISITED);
        graph.add(BLOCKED, CellType::ObjectiveEnemy);
        graph.update_walls(BLOCKED, 3);
        graph.add_neighbor(&CURRENT, &BLOCKED);

        tremeaux_solver(&mut player, &mut graph, None);

        assert_eq!(player.position, BLOCKED);
    }

    #[test]
    fn test_yield_retreats_to_parent_and_reopens_current_cell() {
        let (mut player, mut graph) = graph_around_player(&[BLOCKED, BEHIND, SIDE], BEHIND);

        let action = yield_to_opponent(&mut player, &mut graph, BLOCKED);

        assert_eq!(action, Some(messages::Action::MoveTo(Direction::Back)));
        assert_eq!(player.position, BEHIND);
        assert_eq!(graph.get_cell_status(CURRENT), CellStatus::NotVisited);
    }

    #[test]
    fn test_yield_uses_another_neighbor_without_parent() {
        let (mut player, mut graph) = graph_around_player(&[BLOCKED, SIDE], CURRENT);

        let action = yield_to_opponent(&mut player, &mut graph, BLOCKED);

        assert_eq!(action, Some(messages::Action::MoveTo(Direction::Right)));
        assert_eq!(player.position, SIDE);
        // Backtracking from the side cell must lead back here, without rewriting our parent.
        assert_eq!(graph.get_cell(SIDE).map(|c| c.parent), Some(CURRENT));
        assert_eq!(graph.get_cell_status(CURRENT), CellStatus::VISITED);
    }

    #[test]
    fn test_yield_without_alternative_keeps_player_in_place() {
        let (mut player, mut graph) = graph_around_player(&[BLOCKED], CURRENT);

        assert_eq!(yield_to_opponent(&mut player, &mut graph, BLOCKED), None);
        assert_eq!(player.position, CURRENT);
        assert_eq!(graph.get_cell_status(CURRENT), CellStatus::VISITED);
    }
}
