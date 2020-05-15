// Library bit of the chat server.
// Started May 03, 2020.
//

// Imports
use chrono::{DateTime, Utc};
use std::{
    cell::RefCell,
    fs::{read_to_string, OpenOptions},
    io,
    io::prelude::*,
    rc::Rc,
};
use uuid::Uuid;

// Interface-------------------------------------------------------------
// Command Struct
pub enum Command {}

fn _get_input() -> String {
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
                None => return Err("Invalid $id in USERS file."),
            };
            let user = match line.next() {
                Some(u) => u,
                None => return Err("Invalid $name in USERS file."),
            };
            let email = match line.next() {
                Some(e) => e,
                None => return Err("Invalid $email in USERS file."),
            };
            let create_time: DateTime<Utc> = match line.next() {
                Some(ct) => DateTime::from(DateTime::parse_from_rfc3339(ct).unwrap()),
                None => return Err("Invalid $create_time in USERS file."),
            };
            self.users.push(User::new(RefCell::new(UserInfo::load(
                id,
                user,
                email,
                create_time,
            ))));
        }
        Ok(())
    }

    pub fn load_convs(&mut self, filename: &str) -> Result<(), &'static str> {
        for line in read_to_string(filename).unwrap().lines() {
            let mut line = line.split(';');
            let id = match line.next() {
                Some(id) => Uuid::parse_str(id).unwrap(),
                None => return Err("Invalid $id in CONVS file."),
            };
            let name = match line.next() {
                Some(nm) => nm.to_string(),
                None => return Err("Invalid $name in CONVS file."),
            };
            let mems: Vec<&str> = match line.next() {
                Some(m) => m.split(',').collect(),
                None => return Err("Invalid $mems list in CONVS file."),
            };
            let start: DateTime<Utc> = match line.next() {
                Some(st) => DateTime::from(DateTime::parse_from_rfc3339(st).unwrap()),
                None => return Err("Invalid $start time in CONVS file."),
            };
            let last_msg: DateTime<Utc> = match line.next() {
                Some(lm) => DateTime::from(DateTime::parse_from_rfc3339(lm).unwrap()),
                None => return Err("Invalid $last_msg time in CONVS file."),
            };
            let mut members = Vec::new();
            for mem in mems {
                let mem = Uuid::parse_str(mem).unwrap();
                if self.users.iter().any(|u| u.borrow().id() == mem) {
                    members.push(User::clone(
                        self.users.iter().find(|u| u.borrow().id() == mem).unwrap(),
                    ))
                }
            }
            self.convs
                .push(Conversation::new(RefCell::new(ConvInfo::load(
                    id, &name, members, start, last_msg,
                ))));
        }
        Ok(())
    }

    pub fn load_msgs(&mut self, filename: &str) -> Result<(), &'static str> {
        for line in read_to_string(filename).unwrap().lines() {
            let mut line = line.split(';');
            let id = match line.next() {
                Some(id) => Uuid::parse_str(id).unwrap(),
                None => return Err("Invalid $id in MSGS file."),
            };
            let text = match line.next() {
                Some(tx) => tx.to_string(),
                None => return Err("Invalid $text in MSGS file."),
            };
            let time_stamp: DateTime<Utc> = match line.next() {
                Some(ts) => DateTime::from(DateTime::parse_from_rfc3339(ts).unwrap()),
                None => return Err("Invalid $time_stamp in MSGS file."),
            };
            let user = User::clone(
                self.users
                    .iter()
                    .find(|u| u.borrow().id() == Uuid::parse_str(line.next().unwrap()).unwrap())
                    .unwrap(),
            );
            let conv = Conversation::clone(
                self.convs
                    .iter()
                    .find(|c| c.borrow().id() == Uuid::parse_str(line.next().unwrap()).unwrap())
                    .unwrap(),
            );
            self.msgs.push(Message::new(RefCell::new(MsgInfo::load(
                id, text, time_stamp, user, conv,
            ))));
        }
        Ok(())
    }

    pub fn add_user(&mut self, name: &str, email: &str) -> Result<(), &'static str> {
        if !self
            .users
            .iter()
            .any(|u| u.borrow().name() == name && u.borrow().email() == email)
        {
            self.users
                .push(User::new(RefCell::new(UserInfo::new(name, email))));
            Ok(())
        } else {
            return Err("User already exists with that info!");
        }
    }

    pub fn get_user(&self, name: Option<&str>, email: Option<&str>) -> Option<User> {
        if name == None && email == None {
            None
        } else if name == None && email != None {
            match self
                .users
                .iter()
                .find(|u| u.borrow().email() == email.unwrap())
            {
                Some(ur) => Some(User::clone(ur)),
                None => None,
            }
        } else if name != None && email == None {
            match self
                .users
                .iter()
                .find(|u| u.borrow().name() == name.unwrap())
            {
                Some(ur) => Some(User::clone(ur)),
                None => None,
            }
        } else {
            match self.users.iter().find(|u| {
                u.borrow().name() == name.unwrap() && u.borrow().email() == email.unwrap()
            }) {
                Some(ur) => Some(User::clone(ur)),
                None => None,
            }
        }
    }

    pub fn add_conv(&mut self, name: &str, members: Vec<User>) {
        self.convs
            .push(Conversation::new(RefCell::new(ConvInfo::new(
                name, members,
            ))))
    }

    pub fn get_conv(&self, name: Option<&str>, members: Option<Vec<User>>) -> Option<Conversation> {
        if name == None && members == None {
            None
        } else if name != None && members == None {
            match self
                .convs
                .iter()
                .find(|c| c.borrow().name() == name.unwrap())
            {
                Some(cr) => Some(Conversation::clone(cr)),
                None => None,
            }
        } else if name == None && members != None {
            let members = members.unwrap();
            match self
                .convs
                .iter()
                .find(move |c| members.iter().all(move |m| c.borrow().members.contains(m)))
            {
                Some(cr) => Some(Conversation::clone(cr)),
                None => None,
            }
        } else {
            let members = members.unwrap();
            match self.convs.iter().find(move |c| {
                c.borrow().name() == name.unwrap()
                    && members.iter().all(move |m| c.borrow().members.contains(m))
            }) {
                Some(cr) => Some(Conversation::clone(cr)),
                None => None,
            }
        }
    }

    pub fn send_msg(
        &mut self,
        from: User,
        to: Conversation,
        text: &str,
    ) -> Result<(), &'static str> {
        if to.borrow().members.contains(&from) {
            to.borrow_mut().new_msg();
            self.msgs
                .push(Message::new(RefCell::new(MsgInfo::new(from, to, text))));
            Ok(())
        } else {
            return Err("User not in that conv.");
        }
    }

    pub fn close(
        &mut self,
        msg_file: &str,
        conv_file: &str,
        user_file: &str,
    ) -> Result<(), &'static str> {
        let messages = self.msgs.drain(..).fold(String::new(), |mut acc, m| {
            let m = m.borrow();
            acc += &m.id.to_string();
            acc += ";";
            acc += &m.text;
            acc += ";";
            acc += &m.time_stamp.to_rfc3339();
            acc += ";";
            acc += &m.user.borrow().id().to_string();
            acc += ";";
            acc += &m.conv.borrow().id().to_string();
            acc += "\n";
            acc
        });
        let mut msg_file = match OpenOptions::new().read(true).write(true).open(msg_file) {
            Ok(f) => f,
            Err(e) => {
                println!("{}", e);
                return Err("Error opening messages file!");
            }
        };
        match msg_file.write_all(messages.as_bytes()) {
            Ok(_) => println!("Messages file saved successfully!"),
            Err(e) => {
                println!("{}", e);
                return Err("Error writing messages file!");
            }
        }

        let convs = self.convs.drain(..).fold(String::new(), |mut acc, c| {
            let mut c = c.borrow_mut();
            acc += &c.id().to_string();
            acc += ";";
            acc += &c.name();
            acc += ";";
            acc += &c.members.drain(..).fold(String::new(), |acc, m| {
                acc + &m.borrow().id().to_string() + ","
            });
            acc.pop();
            acc += ";";
            acc += &c.start.to_rfc3339();
            acc += ";";
            acc += &c.last_msg.to_rfc3339();
            acc += "\n";
            acc
        });
        let mut conv_file = match OpenOptions::new().read(true).write(true).open(conv_file) {
            Ok(f) => f,
            Err(e) => {
                println!("{}", e);
                return Err("Error opening conversations file!")
        }};
        match conv_file.write_all(convs.as_bytes()) {
            Ok(_) => println!("Conversations file saved successfully!"),
            Err(e) => {
                println!("{}", e);
                return Err("Error writing conversations file!")
        }}

        let users = self.users.drain(..).fold(String::new(), |mut acc, u| {
            let u = u.borrow();
            acc += &u.id().to_string();
            acc += ";";
            acc += u.name();
            acc += ";";
            acc += u.email();
            acc += ";";
            acc += &u.time().to_rfc3339();
            acc += "\n";
            acc
        });
        let mut user_file = match OpenOptions::new().read(true).write(true).open(user_file) {
            Ok(f) => f,
            Err(e) => {
                println!("{}", e);
                return Err("Error opening users file!")
        }};
        match user_file.write_all(users.as_bytes()) {
            Ok(_) => println!("Users file saved successfully!"),
            Err(e) => {
                println!("{}", e);
                return Err("Error writing users file!")
        }}

        println!("Goodbye! :)");
        Ok(())
    }
}

// Message Struct
#[derive(Debug)]
pub struct MsgInfo {
    pub id: Uuid,
    pub text: String,
    pub time_stamp: DateTime<Utc>,
    pub user: User,
    pub conv: Conversation,
}

pub type Message = Rc<RefCell<MsgInfo>>;

impl MsgInfo {
    fn new(user: User, conv: Conversation, text: &str) -> MsgInfo {
        MsgInfo {
            id: Uuid::new_v4(),
            text: text.to_string(),
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

pub type User = Rc<RefCell<UserInfo>>;

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

    pub fn change_name(&mut self, name: &str) -> Result<(), &'static str> {
        if self.name != name {
            self.name = name.to_string()
        } else {
            return Err("That is already this user's name!");
        }
        Ok(())
    }

    pub fn change_email(&mut self, email: &str) -> Result<(), &'static str> {
        if self.email != email {
            self.email = email.to_string()
        } else {
            return Err("That is already this user's email!");
        }
        Ok(())
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn time(&self) -> DateTime<Utc> {
        self.create_time
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

pub type Conversation = Rc<RefCell<ConvInfo>>;

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

    pub fn new_msg(&mut self) {
        self.last_msg = Utc::now()
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
