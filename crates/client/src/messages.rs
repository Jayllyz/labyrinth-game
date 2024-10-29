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
pub enum SubscribeError {
    AlreadyRegistered,
    InvalidName,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SubscribeResult {
    Ok,
    Err(SubscribeError),
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
    fn test_welcome_message() {
        let welcome = Welcome { version: 1 };
        let msg = Message::Welcome(welcome);
        let serialized = serde_json::to_string(&msg).unwrap();
        assert_eq!(serialized, r#"{"Welcome":{"version":1}}"#);

        let deserialized: Message = serde_json::from_str(&serialized).unwrap();
        matches!(deserialized, Message::Welcome(_));
    }

    #[test]
    fn test_subscribe_message() {
        let subscribe = Subscribe { name: "Player1".to_string() };
        let msg = Message::Subscribe(subscribe);
        let serialized = serde_json::to_string(&msg).unwrap();
        assert_eq!(serialized, r#"{"Subscribe":{"name":"Player1"}}"#);
    }

    #[test]
    fn test_view_message() {
        let view = View { view: "game state".to_string() };
        let msg = Message::View(view);
        let serialized = serde_json::to_string(&msg).unwrap();
        assert_eq!(serialized, r#"{"View":{"view":"game state"}}"#);
    }

    #[test]
    fn test_action_message() {
        let action = Action::MoveTo(Direction::Right);
        let msg = Message::Action(action);
        let serialized = serde_json::to_string(&msg).unwrap();
        assert_eq!(serialized, r#"{"Action":{"MoveTo":"Right"}}"#);
    }

    #[test]
    fn test_action_result() {
        let results = vec![
            ActionResult::Ok,
            ActionResult::Completed,
            ActionResult::Err(ActionError::InvalidMove),
        ];

        for result in results {
            let msg = Message::ActionResult(result);
            let serialized = serde_json::to_string(&msg).unwrap();
            let deserialized: Message = serde_json::from_str(&serialized).unwrap();
            matches!(deserialized, Message::ActionResult(_));
        }
    }

    #[test]
    fn test_full_message_flow() {
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
