use std::collections::{HashMap, HashSet};

use shared::maze::Cell;
use shared::messages::{self, Direction, Message};
use shared::radar::{CellType, Passages, Radar};

use crate::data_structures::maze_graph::{CellStatus, MazeGraph};
use crate::maze_parser::Player;

pub fn tremeaux_solver(
    radar_view: &Radar,
    player: &mut Player,
    graph: &mut MazeGraph,
) -> messages::Action {
    let Some(player_cell) = graph.get_cell(player.position.clone()) else {
        return messages::Action::MoveTo(messages::Direction::Front);
    };
    let mut message: messages::Action = messages::Action::MoveTo(messages::Direction::Front);
    let neighbor_positions: HashSet<Cell> = player_cell.neighbors.clone();

    let mut visited: Vec<Cell> = Vec::new();

    for neighbor_position in neighbor_positions {
        let Some(neighbor_cell) = graph.get_cell(neighbor_position.clone()) else {
            continue;
        };

        if neighbor_cell.status == CellStatus::NotVisited {
            graph.update_cell_status(player.position, CellStatus::VISITED);
            let next_direction = player.get_next_direction(&neighbor_position);
            if next_direction == Direction::Left {
                player.turn_left();
                message = messages::Action::MoveTo(messages::Direction::Left);
            } else if next_direction == Direction::Right {
                player.turn_right();
                message = messages::Action::MoveTo(messages::Direction::Right);
            } else if next_direction == Direction::Back {
                player.turn_back();
                message = messages::Action::MoveTo(messages::Direction::Back);
            } else {
                message = messages::Action::MoveTo(messages::Direction::Front);
            }
            player.move_forward();
            return message;
        }

        if neighbor_cell.status == CellStatus::VISITED {
            visited.push(neighbor_position);
        }
    }

    let back_status = graph.get_cell_status(player.get_back_position());

    if back_status == CellStatus::DeadEnd {
        let next_direction = player.get_next_direction(&visited[0]);
        if next_direction == Direction::Left {
            player.turn_left();
            message = messages::Action::MoveTo(messages::Direction::Left);
        }
        if next_direction == Direction::Right {
            player.turn_right();
            message = messages::Action::MoveTo(messages::Direction::Right);
        }
    } else {
        graph.update_cell_status(player.position, CellStatus::DeadEnd);
        player.turn_back();
        message = messages::Action::MoveTo(messages::Direction::Back);
    }

    player.move_forward();
    message
}

pub fn right_hand_solver(radar_view: &Radar, player: &mut Player) -> messages::Action {
    let messages;

    if let Some(vertical) = radar_view.vertical.get(6) {
        if *vertical == Passages::OPEN {
            messages = messages::Action::MoveTo(messages::Direction::Right);
            player.turn_right();
            player.move_forward();
            return messages;
        }
    }

    if let Some(horizontal) = radar_view.horizontal.get(4) {
        if *horizontal == Passages::OPEN {
            messages = messages::Action::MoveTo(messages::Direction::Front);
            player.move_forward();
            return messages;
        }
    }

    if let Some(vertical) = radar_view.vertical.get(5) {
        if *vertical == Passages::OPEN {
            messages = messages::Action::MoveTo(messages::Direction::Left);
            player.turn_left();
            player.move_forward();
            return messages;
        }
    }

    if let Some(horizontal) = radar_view.horizontal.get(7) {
        if *horizontal == Passages::OPEN {
            messages = messages::Action::MoveTo(messages::Direction::Back);
            player.turn_back();
            player.move_forward();
            return messages;
        }
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

    if let Some(cell) = cells.get(index) {
        if *cell == CellType::OBJECTIVE {
            return true;
        }
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
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use super::*;
    use messages::RadarView;
    use shared::{
        maze::Cell,
        radar::{decode_base64, extract_data},
    };

    #[test]
    fn test_right_hand_solver() {
        let view = RadarView("swfGkIAyap8a8aa".to_owned());
        let mut player =
            Player { position: Cell { row: 0, column: 0 }, direction: messages::Direction::Front };
        let radar_view = extract_data(&decode_base64(&view.0));
        let result = right_hand_solver(&radar_view, &mut player);
        assert!(matches!(result, messages::Action::MoveTo(messages::Direction::Right)));
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
}
