use std::{cmp::max, env, time::Instant};

use agent::Agent;
use connection::get_msg;
use onitama_move_gen::tablebase::TableBase;
use tungstenite::connect;

use crate::messages::{move_to_command, LitamaMsg, StateMsg};

pub mod agent;
mod connection;
mod messages;
mod transpose;

extern crate onitama_move_gen;
#[macro_use]
extern crate serde_derive;
extern crate tungstenite;

fn main() {
    let mut ws = connect("ws://litama.herokuapp.com").unwrap().0;

    let args: Vec<String> = env::args().collect();
    let (index, token, match_id) = if args.len() > 1 {
        ws.write_message(format!("join {} Omega", args[1]).into())
            .unwrap();

        let join = match get_msg(&mut ws) {
            LitamaMsg::Join(join) => join,
            msg => panic!(format!("expected join message: {:?}", msg)),
        };
        (join.index, join.token, args[1].clone())
    } else {
        ws.write_message("create Omega".to_string().into()).unwrap();

        let create = match get_msg(&mut ws) {
            LitamaMsg::Create(create) => create,
            msg => panic!(format!("expected create message: {:?}", msg)),
        };

        println!(
            "got match_id: https://git.io/onitama#spectate-{}",
            create.match_id
        );
        (create.index, create.token, create.match_id)
    };

    ws.write_message(format!("spectate {}", &match_id).into())
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

    let now = Instant::now();
    let mut agent = Agent::new(TableBase::new(state.all_cards()));
    println!("tablebase took: {}", now.elapsed().as_secs_f32());
    let mut depth = 0;

    'game: loop {
        if state.index() == index {
            let game = state.game();
            depth = max(depth, game.count_pieces() as usize);
            let now = Instant::now();
            let new_game = loop {
                let new_game = agent.search(game, depth);
                depth += 1;
                if now.elapsed().as_millis() > 1000 || depth > 22 {
                    break new_game;
                }
            };
            // println!("table: {}", agent.eval(game));
            println!("depth: {}", depth - game.count_pieces() as usize);
            let flip = state.current_turn == "red";
            let command = move_to_command(game, new_game, &match_id, &token, flip);
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
