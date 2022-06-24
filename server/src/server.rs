//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use serde::{Serialize, Deserialize};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use crate::session;
#[derive(Serialize)]
pub struct QuestionStruct {
    pub question_value: i32,
}

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// Message for chat server communications

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub code: String,
}
/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Start {
    pub id: usize,
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct UserResponse {
    pub id: usize,
    pub reponse: i32,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// Send message to specific room
#[derive(Deserialize, Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: String,
}

/// List of available rooms
pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

/// `ChatServer` manages chat rooms and responsible for coordinating chat session.
///
/// Implementation is very naïve.
#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
    visitor_count: Arc<AtomicUsize>,
}

impl ChatServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> ChatServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("Main".to_owned(), HashSet::new());

        ChatServer {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
            visitor_count,
        }
    }
}

impl ChatServer {
    /// Send message to all users in the room
    fn send_message(&self, room: &str, message: &str, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        let _ = addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for ChatServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // notify all users in same room
        self.send_message("Main", "Someone joined", 0);

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // auto join session to Main room
        self.rooms
            .entry("Main".to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);

        let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        self.send_message("Main", &format!("Total visitors {count}"), 0);

        // send id back
        id
    }
}
//Start message
impl Handler<Start> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Start, _: &mut Context<Self>) {
        println!("Start questions");

        let mut room = None;

        for (key, val) in self.rooms.iter() {
            if val.contains(&msg.id) {
                room = Some(key);
                break;
            }
        }
        let question = QuestionStruct { question_value: 0 };
        if let Some(sessions) = self.rooms.get(room.unwrap()) {
            for id in sessions {
                if let Some(addr) = self.sessions.get(id) {
                    let _ = addr.do_send(Message(serde_json::to_string(&question).unwrap()));
                }
            }
        }
    }
}
// reponse utilisateur message
impl Handler<UserResponse> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: UserResponse, _: &mut Context<Self>) {
        println!("l'utilisateur à bien répondu");

        let reponses = session::ResponseStruct {
            id_question: 0,
            response_value: 1,
        };
    }
}
// user join
impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        println!("l'utilisateur à bien join");
    }
}


/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnectedL");

        let mut rooms: Vec<String> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
        // send message to other users
        for room in rooms {
            self.send_message(&room, "Someone disconnected", 0);
        }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(&msg.room, msg.msg.as_str(), msg.id);
    }
}

/// Handler for `ListRooms` message.
impl Handler<ListRooms> for ChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.rooms.keys() {
            rooms.push(key.to_owned())
        }

        MessageResult(rooms)
    }
}


