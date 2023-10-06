use std::collections::HashMap;

use actix::prelude::*;
use uuid::Uuid;

use crate::PushData;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: Uuid,
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

pub struct NotifyServer {
    sessions: HashMap<Uuid, Recipient<Message>>,
}

impl NotifyServer {
    pub fn new() -> NotifyServer {
        NotifyServer {
            sessions: HashMap::new(),
        }
    }
}

impl Actor for NotifyServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        self.sessions.insert(msg.id, msg.addr);
    }
}

impl Handler<Disconnect> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);
    }
}

impl Handler<PushData> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: PushData, _: &mut Context<Self>) {
        let recipient = self.sessions.get(&msg.user_id);
        if recipient.is_some() {
            recipient.unwrap().do_send(Message(msg.data.to_owned()));
        }
    }
}
