use std::{env, mem::take, rc::Rc, time::Instant};

use connection::{get_msg, get_next_state};
use messages::StateObj;
use onitama_move_gen::tablebase::TableBase;
use tungstenite::{client::AutoStream, connect, WebSocket};

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
    let tablebase: Rc<TableBase> = TableBase::new(state.all_cards()).into();
    println!("tablebase took: {}", now1.elapsed().as_secs_f32());

    if state.index() != index {
        get_next_state(&mut state, &mut ws)?;
    }

    let mut agent = Agent::new(tablebase.clone());
    let mut node = agent.new_node(state.game());

    let mut runner = Runner {
        ws,
        state,
        token,
        match_id,
    };

    loop {
        let other_agent = Agent::new(tablebase.clone());

        runner.run(&agent, &mut node)?;

        let mut other_node = other_agent.copy(&node);
        agent = Agent::new(tablebase.clone());

        runner.run(&other_agent, &mut other_node)?;

        node = agent.copy(&other_node);
    }
}

struct Runner {
    ws: WebSocket<AutoStream>,
    state: StateObj,
    token: String,
    match_id: String,
}

impl Runner {
    fn run<'a>(&mut self, agent: &'a Agent, node: &mut Node<'a>) -> Option<()> {
        let now2 = Instant::now();
        loop {
            agent.bns(node);
            if now2.elapsed().as_millis() > 1000 || node.depth > 20 {
                break;
            }
        }
        *node = take(node.nodes.as_mut().unwrap().iter_mut().next().unwrap());

        let flip = self.state.current_turn == "red";
        let command = move_to_command(
            self.state.game(),
            node.game,
            &self.match_id,
            &self.token,
            flip,
        );
        self.ws.write_message(command.into()).unwrap();

        get_next_state(&mut self.state, &mut self.ws)?;
        get_next_state(&mut self.state, &mut self.ws)?;
        let cond = |n: &&mut Node| n.game == self.state.game();
        agent.expand(node);
        *node = take(node.nodes.as_mut().unwrap().iter_mut().find(cond).unwrap());

        Some(())
    }
}
