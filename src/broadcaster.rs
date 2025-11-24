use actix::{Actor, Handler, Recipient};
use std::collections::HashSet;

use crate::messages::{BroadcastLog, Connect, Disconnect, WsMessage};

#[derive(Default)]
pub struct Broadcaster {
    subscribers: HashSet<Recipient<WsMessage>>,
}

impl Broadcaster {
    pub fn new() -> Self {
        Broadcaster {
            subscribers: HashSet::new(),
        }
    }
}

impl Actor for Broadcaster {
    type Context = actix::Context<Self>;
}

impl Handler<Connect> for Broadcaster {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) {
        self.subscribers.insert(msg.addr);
        println!(
            "New WS Client subscribed. Total: {}",
            self.subscribers.len()
        )
    }
}

impl actix::Handler<Disconnect> for Broadcaster {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) {
        self.subscribers.remove(&msg.addr);
        println!("WS Client unsubscribed. Total : {}", self.subscribers.len());
    }
}

impl actix::Handler<BroadcastLog> for Broadcaster {
    type Result = ();

    fn handle(&mut self, msg: BroadcastLog, _ctx: &mut Self::Context) {
        let log_line = format!(
            "[{}]: {}",
            if msg.is_error { "ERR" } else { "OUT" },
            msg.message
        );

        for client in &self.subscribers {
            client.do_send(WsMessage(log_line.clone()));
        }
    }
}
