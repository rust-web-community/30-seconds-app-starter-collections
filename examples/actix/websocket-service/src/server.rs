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
    pub path: String,
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

struct PathNode {
    path: String,
    clients: HashMap<Uuid, Recipient<Message>>,
    children: Vec<PathNode>,
}

impl PathNode {
    pub fn print(&self) {
        println!("{} {}", self.path, self.clients.len());
        for child in &self.children {
            child.print()
        }
    }

    pub fn remove_client(&mut self, user_id: &Uuid) {
        if self.clients.contains_key(user_id) {
            self.clients.remove(user_id);
        } else {
            for child in &mut self.children {
                child.remove_client(user_id)
            }
        }
    }

    pub fn find_node(&self, full_path: &str) -> (&PathNode, String) {
        let mut node = self;
        let mut path = full_path;
        let mut should_break = false;
        while path.len() > 0 && !should_break {
            should_break = true;
            for child in &node.children {
                let strip_prefix = path.strip_prefix(&["/", &child.path].join(""));
                println!("{} {} {}", path, &child.path, strip_prefix.is_some());
                if strip_prefix.is_some() {
                    node = &child;
                    path = strip_prefix.unwrap();
                    should_break = false;
                    break;
                }
            }
        }
        println!("{}, {}", node.path, path);
        return (node, path.to_owned());
    }

    pub fn add_node_to_path(
        &mut self,
        remaining_path: &str,
        user_id: Uuid,
        recipient: &Recipient<Message>,
    ) {
        for child in self.children.iter_mut() {
            let strip_prefix = remaining_path.strip_prefix(&child.path);
            if strip_prefix.is_some() {
                child.add_node_to_path(strip_prefix.unwrap(), user_id, recipient);
                return;
            }
        }
        let mut consumed_path = remaining_path;
        let next_path: &str;
        (consumed_path, next_path) = consumed_path
            .strip_prefix("/")
            .unwrap_or(consumed_path)
            .split_once("/")
            .unwrap_or((consumed_path, ""));
        let mut new_child = PathNode {
            path: consumed_path.to_owned(),
            clients: HashMap::new(),
            children: Vec::new(),
        };
        if consumed_path.len() > 0 {
            new_child.add_node_to_path(next_path, user_id, recipient);
            self.children.push(new_child);
        } else {
            self.clients.insert(user_id, recipient.clone());
            println!("{} {}", self.path, next_path)
        }
    }

    fn broadcast_rec(&self, msg_data: &str) {
        println!("BROADCAST {}", self.path);
        for recipient in self.clients.values() {
            recipient.do_send(Message(msg_data.to_owned()));
        }
        for child in &self.children {
            println!("{}", child.path);
            child.broadcast_rec(msg_data);
        }
    }
}

pub struct NotifyServer {
    sessions: PathNode,
}

impl NotifyServer {
    pub fn new() -> NotifyServer {
        NotifyServer {
            sessions: PathNode {
                path: "".to_owned(),
                clients: HashMap::new(),
                children: Vec::new(),
            },
        }
    }
}

impl Actor for NotifyServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        self.sessions.add_node_to_path(&msg.path, msg.id, &msg.addr);
    }
}

impl Handler<Disconnect> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove_client(&msg.id);
    }
}

impl Handler<PushData> for NotifyServer {
    type Result = ();

    fn handle(&mut self, msg: PushData, _: &mut Context<Self>) {
        self.sessions.print();
        let stripped_path = &msg.path.strip_suffix("/").unwrap_or(&msg.path);
        let (node, path) = self.sessions.find_node(stripped_path);
        if path.len() == 0 {
            if msg.post.user_id.is_some() {
                let opt_recpient = node.clients.get(&msg.post.user_id.unwrap());
                if opt_recpient.is_none() {
                    return;
                }
                opt_recpient
                    .unwrap()
                    .do_send(Message(msg.post.data.to_owned()));
            } else {
                // Broadcast recursively
                node.broadcast_rec(msg.post.data.as_str());
            }
        }
    }
}
