use std::time::Instant;

use agent::Agent;
use connection::get_msg;
use onitama_move_gen::tablebase::TableBase;
use tungstenite::connect;

use crate::messages::{move_to_command, LitamaMsg, StateMsg};

mod agent;
mod connection;
mod messages;
mod transpose;

extern crate onitama_move_gen;
#[macro_use]
extern crate serde_derive;
extern crate tungstenite;

fn main() {
    let mut ws = connect("ws://litama.herokuapp.com").unwrap().0;

    ws.write_message("create Omega".to_string().into()).unwrap();

    let create = match get_msg(&mut ws) {
        LitamaMsg::Create(create) => create,
        msg => panic!(format!("expected create message: {:?}", msg)),
    };

    println!(
        "got match_id: https://l0laapk3.github.io/Onitama-client/#spectate-{}",
        create.match_id
    );

    ws.write_message(format!("spectate {}", create.match_id).into())
        .unwrap();

    match get_msg(&mut ws) {
        LitamaMsg::Spectate => {}
        msg => panic!(format!("expected spectate message: {:?}", msg)),
    };

    let mut state = loop {
        match get_msg(&mut ws) {
            LitamaMsg::State(StateMsg::WaitingForPlayers) => {}
            LitamaMsg::State(StateMsg::InProgress(state)) => break state,
            msg => panic!(format!("expected state message: {:?}", msg)),
        }
    };

    let mut agent = Agent::new(TableBase::new(state.all_cards()));
    let mut depth = 0;

    'game: loop {
        if state.index() == create.index {
            let game = state.game();
            let now = Instant::now();
            let new_game = loop {
                let new_game = agent.search(game, depth);
                depth += 1;
                if now.elapsed().as_millis() > 3000 || depth > 32 {
                    break new_game;
                }
            };
            println!("depth: {}", depth);
            let flip = state.current_turn == "red";
            let command = move_to_command(game, new_game, &create.match_id, &create.token, flip);
            ws.write_message(command.into()).unwrap();
            depth = depth.saturating_sub(2);
        }

        state = loop {
            match get_msg(&mut ws) {
                LitamaMsg::Move => {}
                LitamaMsg::State(StateMsg::InProgress(new_state)) => {
                    if state != new_state {
                        break new_state;
                    }
                }
                LitamaMsg::State(StateMsg::Ended) => break 'game,
                LitamaMsg::Error(_) => {}
                msg => panic!(format!("expected state/move message: {:?}", msg)),
            }
        };
    }
}
