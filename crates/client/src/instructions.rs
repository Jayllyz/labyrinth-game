use shared::messages::{self};
use shared::radar::{Cells, Passages};

pub fn right_hand_solver(horizontal: Vec<Passages>, vertical: Vec<Passages>) -> messages::Action {
    let messages;

    if let Some(vertical) = vertical.get(6) {
        if *vertical == Passages::OPEN {
            messages = messages::Action::MoveTo(messages::Direction::Right);
            return messages;
        }
    }

    if let Some(horizontal) = horizontal.get(4) {
        if *horizontal == Passages::OPEN {
            messages = messages::Action::MoveTo(messages::Direction::Front);
            return messages;
        }
    }

    if let Some(vertical) = vertical.get(5) {
        if *vertical == Passages::OPEN {
            messages = messages::Action::MoveTo(messages::Direction::Left);
            return messages;
        }
    }

    if let Some(horizontal) = horizontal.get(7) {
        if *horizontal == Passages::OPEN {
            messages = messages::Action::MoveTo(messages::Direction::Back);
            return messages;
        }
    }

    messages::Action::MoveTo(messages::Direction::Right)
}

pub fn check_win_condition(cells: Vec<Cells>, direction: messages::Action) -> bool {
    let index = match direction {
        messages::Action::MoveTo(messages::Direction::Right) => 5,
        messages::Action::MoveTo(messages::Direction::Left) => 3,
        messages::Action::MoveTo(messages::Direction::Front) => 1,
        messages::Action::MoveTo(messages::Direction::Back) => 7,
        _ => return false,
    };

    if let Some(cell) = cells.get(index) {
        if *cell == Cells::OBJECTIVE {
            return true;
        }
    }

    false
}

pub fn solve_sum_modulo(
    secret_sum: u128,
    secrets: &std::collections::HashMap<std::thread::ThreadId, u128>,
) -> String {
    let sum = secrets.values().sum::<u128>();
    let result = sum % secret_sum;

    return result.to_string();
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use super::*;
    use messages::RadarView;
    use shared::radar::{decode_base64, extract_data};

    #[test]
    fn test_right_hand_solver() {
        let view = RadarView("swfGkIAyap8a8aa".to_owned());
        let (horizontal, vertical, _cells) = extract_data(&decode_base64(&view.0));
        let result = right_hand_solver(horizontal, vertical);
        assert!(matches!(result, messages::Action::MoveTo(messages::Direction::Right)));
    }

    #[test]
    fn test_check_win_condition_right() {
        let cells = vec![
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::OBJECTIVE,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
        ];
        let direction = messages::Action::MoveTo(messages::Direction::Right);
        assert!(check_win_condition(cells, direction));
    }

    #[test]
    fn test_check_win_condition_left() {
        let cells = vec![
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::OBJECTIVE,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
        ];
        let direction = messages::Action::MoveTo(messages::Direction::Left);
        assert!(check_win_condition(cells, direction));
    }

    #[test]
    fn test_check_win_condition_front() {
        let cells = vec![
            Cells::NOTHING,
            Cells::OBJECTIVE,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
        ];
        let direction = messages::Action::MoveTo(messages::Direction::Front);
        assert!(check_win_condition(cells, direction));
    }

    #[test]
    fn test_check_win_condition_back() {
        let cells = vec![
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::OBJECTIVE,
            Cells::NOTHING,
        ];
        let direction = messages::Action::MoveTo(messages::Direction::Back);
        assert!(check_win_condition(cells, direction));
    }

    #[test]
    fn test_check_win_condition_no_objective() {
        let cells = vec![
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
            Cells::NOTHING,
        ];
        let direction = messages::Action::MoveTo(messages::Direction::Right);
        assert!(!check_win_condition(cells, direction));
    }

    #[test]
    fn test_sum_modulo() {
        let secrets_arc = Arc::new(Mutex::new(HashMap::new()));
        let mut handles = vec![];

        let values = vec![2667360881372235285, 7064968778338382540, 8653237798568263501];

        for value in values {
            let secrets = secrets_arc.clone();
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
