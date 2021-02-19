use tungstenite::{client::AutoStream, Message, WebSocket};

use crate::messages::LitamaMsg;

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
