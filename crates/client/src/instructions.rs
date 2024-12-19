use crate::data_structures::maze_graph::{CellStatus, MazeGraph};
use crate::maze_parser::Player;
use shared::maze::Cell;
use shared::messages::{self, Direction};
use shared::radar::{CellType, Passages, Radar};
use std::collections::HashMap;

pub fn tremeaux_solver(player: &mut Player, graph: &mut MazeGraph) -> messages::Action {
    let mut message: messages::Action = messages::Action::MoveTo(messages::Direction::Front);

    let (neighbor_positions, parent) = {
        let Some(player_cell) = graph.get_cell(player.position) else {
            return messages::Action::MoveTo(messages::Direction::Front);
        };
        (player_cell.neighbors.clone(), player_cell.parent)
    };

    let mut visited: Vec<Cell> = Vec::new();

    for neighbor_position in neighbor_positions {
        let (walls, mut status, cell_type) = {
            let Some(neighbor_cell) = graph.get_cell(neighbor_position) else {
                continue;
            };
            (neighbor_cell.walls, neighbor_cell.status.clone(), neighbor_cell.cell_type.clone())
        };

        if walls == 3 && !(cell_type == CellType::OBJECTIVE || cell_type == CellType::HELP) {
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
    let next_direction = if parent_status == CellStatus::DeadEnd || parent == player.position {
        player.get_next_direction(&visited[0])
    } else {
        player.get_next_direction(&parent)
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

    graph.update_cell_status(player.position, CellStatus::DeadEnd);
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
        let radar_view = extract_data(&decode_base64(&view.0));
        let result = right_hand_solver(&radar_view, &mut player);
        assert!(matches!(result, messages::Action::MoveTo(messages::Direction::Right)));
    }

    #[test]
    fn test_tremeaux_solver() {
        let view = RadarView("begGkcIyap8p8pa".to_owned());
        let radar = extract_data(&decode_base64(&view.0));
        let mut player = Player::new();
        let mut graph = MazeGraph::new();
        maze_to_graph(&radar, &player, &mut graph);

        let result = tremeaux_solver(&mut player, &mut graph);

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
}
