use std::collections::HashMap;

use shared::messages::{self};
use shared::radar::{CellType, Passages, RadarView};

use crate::maze_parser::Player;

pub fn right_hand_solver(radar_view: &RadarView, player: &mut Player) -> messages::Action {
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
