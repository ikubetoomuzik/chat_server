// Library bit of the chat server.
// Started May 03, 2020.
//

// Imports
use chrono::{DateTime, Utc};
use colored::*;
use std::{fmt, fs::read_to_string, io, rc::Rc};
use uuid::Uuid;

// Interface-------------------------------------------------------------
// Command Struct
pub enum Command {}

fn get_input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line.");
    input
}

// System----------------------------------------------------------------
#[derive(Debug)]
pub struct App {
    pub users: Vec<User>,
    pub convs: Vec<Conversation>,
    pub msgs: Vec<Message>,
    start: DateTime<Utc>,
}

impl App {
    pub fn new() -> App {
        App {
            users: Vec::new(),
            convs: Vec::new(),
            msgs: Vec::new(),
            start: Utc::now(),
        }
    }

    pub fn load_users(&mut self, filename: &str) -> Result<(), &'static str> {
        for line in read_to_string(filename).unwrap().lines() {
            let mut line = line.split(';');
            let id = match line.next() {
                Some(id) => Uuid::parse_str(id).unwrap(),
                None => return Err("Invalid Users file."),
            };
            let user = match line.next() {
                Some(u) => u,
                None => return Err("Invalid Users file."),
            };
            let email = match line.next() {
                Some(e) => e,
                None => return Err("Invalid Users file."),
            };
            let create_time: DateTime<Utc> = match line.next() {
                Some(ct) => DateTime::from(DateTime::parse_from_rfc3339(ct).unwrap()),
                None => return Err("Invalid Users file."),
            };
            self.users
                .push(User::new(UserInfo::load(id, user, email, create_time)));
        }
        Ok(())
    }

    pub fn load_convs(&mut self, filename: &str) -> Result<(), &'static str> {
        for line in read_to_string(filename).unwrap().lines() {
            let mut line = line.split(';');
            let id = match line.next() {
                Some(id) => Uuid::parse_str(id).unwrap(),
                None => return Err("Invalid Users file."),
            };
            let name = match line.next() {
                Some(nm) => nm.to_string(),
                None => return Err("Invalid Users file."),
            };
            let mems: Vec<&str> = match line.next() {
                Some(m) => m.split(',').collect(),
                None => return Err("Invalid Users file."),
            };
            let start: DateTime<Utc> = match line.next() {
                Some(st) => DateTime::from(DateTime::parse_from_rfc3339(st).unwrap()),
                None => return Err("Invalid Users file."),
            };
            let last_msg: DateTime<Utc> = match line.next() {
                Some(lm) => DateTime::from(DateTime::parse_from_rfc3339(lm).unwrap()),
                None => return Err("Invalid Users file."),
            };
            let mut members = Vec::new();
            for mem in mems {
                let mem = Uuid::parse_str(mem).unwrap();
                if self.users.iter().any(|u| u.id() == mem) {
                    members.push(Rc::clone(
                        self.users.iter().find(|u| u.id() == mem).unwrap(),
                    ))
                }
            }
            self.convs.push(Conversation::new(ConvInfo::load(
                id, &name, members, start, last_msg,
            )));
        }
        Ok(())
    }

    pub fn load_msgs(&mut self, filename: &str) -> Result<(), &'static str> {
        for line in read_to_string(filename).unwrap().lines() {
            let mut line = line.split(';');
            let id = match line.next() {
                Some(id) => Uuid::parse_str(id).unwrap(),
                None => return Err("Invalid Users file."),
            };
            let text = match line.next() {
                Some(tx) => tx.to_string(),
                None => return Err("Invalid Users file."),
            };
            let time_stamp: DateTime<Utc> = match line.next() {
                Some(ts) => DateTime::from(DateTime::parse_from_rfc3339(ts).unwrap()),
                None => return Err("Invalid Users file."),
            };
            let user = Rc::clone(
                self.users
                    .iter()
                    .find(|u| u.id() == Uuid::parse_str(line.next().unwrap()).unwrap())
                    .unwrap(),
            );
            let conv = Rc::clone(
                self.convs
                    .iter()
                    .find(|c| c.id() == Uuid::parse_str(line.next().unwrap()).unwrap())
                    .unwrap(),
            );
            self.msgs.push(Message::new(MsgInfo::load(
                id, text, time_stamp, user, conv,
            )));
        }
        Ok(())
    }
}

// Message Struct
#[derive(Debug)]
pub struct MsgInfo {
    id: Uuid,
    text: String,
    time_stamp: DateTime<Utc>,
    user: User,
    conv: Conversation,
}

type Message = Rc<MsgInfo>;

impl MsgInfo {
    fn new(user: User, conv: Conversation, text: String) -> MsgInfo {
        MsgInfo {
            id: Uuid::new_v4(),
            text,
            time_stamp: Utc::now(),
            user,
            conv,
        }
    }

    fn load(
        id: Uuid,
        text: String,
        time_stamp: DateTime<Utc>,
        user: User,
        conv: Conversation,
    ) -> MsgInfo {
        MsgInfo {
            id,
            text,
            time_stamp,
            user,
            conv,
        }
    }
}

// User Struct
#[derive(Debug, PartialEq, Clone)]
pub struct UserInfo {
    id: Uuid,
    name: String,
    email: String,
    create_time: DateTime<Utc>,
}

type User = Rc<UserInfo>;

impl UserInfo {
    pub fn new(name: &str, email: &str) -> UserInfo {
        let name = name.to_string();
        let email = email.to_string();
        UserInfo {
            id: Uuid::new_v4(),
            name,
            email,
            create_time: Utc::now(),
        }
    }

    pub fn load(id: Uuid, name: &str, email: &str, create_time: DateTime<Utc>) -> UserInfo {
        let name = name.to_string();
        let email = email.to_string();
        UserInfo {
            id,
            name,
            email,
            create_time,
        }
    }

    pub fn send_msg(&self, app: &App) {
        let convos = app
            .convs
            .iter()
            .filter_map(|c| {
                let s = app.users.iter().find(|u| u.id() == self.id()).unwrap();
                if c.members.contains(s) {
                    Some(c.name())
                } else {
                    None
                }
            })
            .fold(String::new(), |acc, cn| format!("{}{} ", acc, cn));
        println!("What would you like to say?");
        let text = get_input();
        println!("To what convo: {}?", convos);
        let conv = get_input();
    }

    fn id(&self) -> Uuid {
        self.id
    }
}

// Conversation Struct
#[derive(Debug)]
pub struct ConvInfo {
    id: Uuid,
    name: String,
    members: Vec<User>,
    start: DateTime<Utc>,
    last_msg: DateTime<Utc>,
}

type Conversation = Rc<ConvInfo>;

impl ConvInfo {
    pub fn new(name: &str, members: Vec<User>) -> ConvInfo {
        let time = Utc::now();
        ConvInfo {
            id: Uuid::new_v4(),
            name: name.to_string(),
            members,
            start: time,
            last_msg: time,
        }
    }

    pub fn load(
        id: Uuid,
        name: &str,
        members: Vec<User>,
        start: DateTime<Utc>,
        last_msg: DateTime<Utc>,
    ) -> ConvInfo {
        ConvInfo {
            id,
            name: name.to_string(),
            members,
            start,
            last_msg,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }
}
