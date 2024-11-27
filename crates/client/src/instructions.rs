use shared::{
    messages::{self, RadarView},
    radar,
};

pub fn right_hand_solver(view: RadarView) -> messages::Action {
    let decoded = radar::decode(&view.0);
    let (horizontal, vertical, _cells) = radar::extract_data(&decoded);
    let messages;

    if let Some(vertical) = vertical.get(6) {
        if vertical == "open" {
            messages = messages::Action::MoveTo(messages::Direction::Right);
            return messages;
        }
    }

    if let Some(horizontal) = horizontal.get(4) {
        if horizontal == "open" {
            messages = messages::Action::MoveTo(messages::Direction::Front);
            return messages;
        }
    }

    if let Some(vertical) = vertical.get(5) {
        if vertical == "open" {
            messages = messages::Action::MoveTo(messages::Direction::Left);
            return messages;
        }
    }

    if let Some(horizontal) = horizontal.get(7) {
        if horizontal == "open" {
            messages = messages::Action::MoveTo(messages::Direction::Back);
            return messages;
        }
    }

    messages::Action::MoveTo(messages::Direction::Right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_right_hand_solver() {
        let view = RadarView("swfGkIAyap8a8aa".to_owned());
        let result = right_hand_solver(view);
        assert!(matches!(result, messages::Action::MoveTo(messages::Direction::Right)));
    }
}
