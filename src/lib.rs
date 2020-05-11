// Library bit of the chat server.
// Started May 03, 2020.
//

// Imports
use std::fmt;
use uuid::Uuid;
use colored::*;
use chrono::{DateTime, Utc};

// Interface-------------------------------------------------------------
// Command Struct
pub enum Command {
}

// System----------------------------------------------------------------

// Message Struct
#[derive(Debug)]
struct Message {
    id: Uuid,
    text: String,
    time_stamp: DateTime<Utc>,
    user: User,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({} {}) --> [\"{}\"]",
            "FROM:".red(),
            self.user.name().cyan(),
            self.text.bright_yellow()
        )
    }
}

impl Message {
    fn new(user: User, text: String) -> Message {
        Message {
            id: Uuid::new_v4(),
            text,
            time_stamp: Utc::now(),
            user,
        }
    }
}

// User Struct
#[derive(Debug, PartialEq, Clone)]
pub struct User {
    id: Uuid,
    name: String,
    email: String,
    create_time: DateTime<Utc>,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} --> {})", self.name.cyan(), self.email.red())
    }
}

impl User {
    pub fn new(name: &str, email: &str) -> User {
        let name = name.to_string();
        let email = email.to_string();
        User {
            id: Uuid::new_v4(),
            name,
            email,
            create_time: Utc::now(),
        }
    }

    pub fn id(&self) -> String {
        self.id.to_string()
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }

    pub fn send_msg(&self, conversation: &mut Conversation, text: String) {
        let message = Message::new(self.clone(), text);
        conversation.add_msg(self, message);
    }
}

// Conversation Struct
#[derive(Debug)]
pub struct Conversation {
    id: Uuid,
    members: Vec<User>,
    messages: Vec<Message>,
    start: DateTime<Utc>,
    last_msg: DateTime<Utc>,
}

impl fmt::Display for Conversation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut users = String::new();
        for m in &self.members {
            users += &format!("{} ", m);
        }
        write!(
            f,
            "Users: {}\nLast Message: {}",
            users,
            self.messages.last().unwrap()
        )
    }
}

impl Conversation {
    pub fn new(members: Vec<User>) -> Conversation {
        let time = Utc::now();
        Conversation {
            id: Uuid::new_v4(),
            members,
            messages: Vec::new(),
            start: time,
            last_msg: time,
        }
    }

    fn add_msg(&mut self, user: &User, msg: Message) {
        if self.members.contains(user) {
            self.last_msg = msg.time_stamp;
            self.messages.push(msg);
        }
    }

    fn add_member(&mut self, member: User) {
        if !self.members.contains(&member) {
            self.members.push(member);
        }
    }
}
