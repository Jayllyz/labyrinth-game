use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Hello,
    Welcome(Welcome),
    Subscribe(Subscribe),
    SubscribeResult(SubscribeResult),
    View(View),
    Action(Action),
    ActionResult(ActionResult),
    MessageError(MessageError),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hello;

#[derive(Serialize, Deserialize, Debug)]
pub struct Welcome {
    pub version: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageError {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscribe {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SubscribeResult {
    Ok,
    Err(SubscribeError),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SubscribeError {
    AlreadyRegistered,
    InvalidName,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct View {
    pub view: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ViewModel {
    pub view: View,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    #[serde(rename = "MoveTo")]
    MoveTo(Direction),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionError {
    InvalidMove,
    OutOfMap,
    Blocked,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ActionResult {
    Ok,
    Completed,
    Err(ActionError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_hello_message() {
        let msg = Message::Hello;
        let serialized = serde_json::to_string(&msg).unwrap();
        assert_eq!(serialized, r#""Hello""#);

        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        matches!(deserialized, Message::Hello);
    }

    #[test]
    fn test_all_messages() {
        let messages = vec![
            Message::Hello,
            Message::Welcome(Welcome { version: 1 }),
            Message::Subscribe(Subscribe { name: "Player1".to_string() }),
            Message::SubscribeResult(SubscribeResult::Ok),
            Message::View(View { view: "Initial state".to_string() }),
            Message::Action(Action::MoveTo(Direction::Right)),
            Message::ActionResult(ActionResult::Ok),
            Message::MessageError(MessageError { message: "Error".to_string() }),
        ];

        for msg in messages {
            let serialized = serde_json::to_string(&msg).unwrap();
            let deserialized: Message = serde_json::from_str(&serialized).unwrap();
            matches!(
                deserialized,
                Message::Hello
                    | Message::Welcome(_)
                    | Message::Subscribe(_)
                    | Message::SubscribeResult(_)
                    | Message::View(_)
                    | Message::Action(_)
                    | Message::ActionResult(_)
                    | Message::MessageError(_)
            );
        }
    }
}
