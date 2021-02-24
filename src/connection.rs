use tungstenite::{client::AutoStream, Message, WebSocket};

use crate::messages::{LitamaMsg, StateMsg, StateObj};

pub fn get_msg(ws: &mut WebSocket<AutoStream>) -> LitamaMsg {
    let msg = loop {
        match ws.read_message().unwrap() {
            Message::Text(msg) => break msg,
            Message::Ping(val) => ws.write_message(Message::Pong(val)).unwrap(),
            msg => panic!(format!("unexpected message: {}", msg)),
        }
    };
    // println!("got message {:?}", &msg);
    serde_json::from_str::<LitamaMsg>(&msg).unwrap()
}

pub fn get_next_state(state: StateObj, ws: &mut WebSocket<AutoStream>) -> Option<StateObj> {
    loop {
        match get_msg(ws) {
            LitamaMsg::Move => {}
            LitamaMsg::State(StateMsg::InProgress(new_state)) => {
                if state != new_state {
                    break Some(new_state);
                }
            }
            LitamaMsg::State(StateMsg::Ended) => break None,
            LitamaMsg::Error(_) => {}
            msg => panic!(format!("expected state/move message: {:?}", msg)),
        }
    }
}
