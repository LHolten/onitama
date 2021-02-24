use std::{env, time::Instant};

use connection::{get_msg, get_next_state};
use onitama_move_gen::tablebase::TableBase;
use tungstenite::connect;

use crate::{
    messages::{move_to_command, LitamaMsg, StateMsg},
    node::{Agent, Node},
};

mod connection;
mod messages;
pub mod node;

extern crate onitama_move_gen;
#[macro_use]
extern crate serde_derive;
extern crate tungstenite;

fn main() {
    run_loop();
}

fn run_loop() -> Option<()> {
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

    let now1 = Instant::now();
    let mut agent = Agent(TableBase::new(state.all_cards()));
    println!("tablebase took: {}", now1.elapsed().as_secs_f32());

    if state.index() != index {
        state = get_next_state(state, &mut ws)?;
    }
    let mut node = agent.new_node(state.game());

    loop {
        let now2 = Instant::now();
        loop {
            agent.bns(&mut node);
            if now2.elapsed().as_millis() > 1000 || node.depth > 20 {
                break;
            }
        }
        // println!(
        //     "{}, {}, {}, {}",
        //     node.lower,
        //     node.nodes[0].lower,
        //     node.nodes[0].nodes[0].lower,
        //     node.nodes[0].nodes[0].nodes[0].lower
        // );
        // println!("table: {}", agent.eval(game));
        let flip = state.current_turn == "red";
        node = node.nodes.unwrap().into_iter().next().unwrap();
        let command = move_to_command(state.game(), node.game, &match_id, &token, flip);
        ws.write_message(command.into()).unwrap();

        // let lower = node.lower;
        state = get_next_state(state, &mut ws)?;
        state = get_next_state(state, &mut ws)?;
        let cond = |n: &Node| n.game == state.game();
        node = node.nodes.unwrap().into_iter().find(cond).unwrap();
        // node.lower = lower;
    }
}
