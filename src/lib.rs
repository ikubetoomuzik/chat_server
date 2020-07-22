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
    net::{SocketAddr, TcpListener},
    rc::Rc,
};
use uuid::Uuid;

// TCP server.

#[derive(Debug)]
pub struct TcpServer;

impl TcpServer {
    pub fn listen(port: u16, app: &mut App) {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(&addr).unwrap();
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut buffer = [0; 512];
            stream.read(&mut buffer).unwrap();
            let req = String::from_utf8_lossy(&buffer[..]);
            if req.starts_with("END") {
                break;
            } else {
                stream
                    .write(app.execute(req.to_string()).as_bytes())
                    .unwrap();
                stream.flush().unwrap();
            }
            println!("Request: {}", &req);
        }
    }
}

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
    users: Vec<User>,
    convs: Vec<Conversation>,
    msgs: Vec<Message>,
    rels: Vec<Relationship>,
    start: DateTime<Utc>,
}

enum ConvSearch {
    Name(String),
    Members(Vec<User>),
}

impl App {
    pub fn new() -> App {
        App {
            users: Vec::new(),
            convs: Vec::new(),
            msgs: Vec::new(),
            rels: Vec::new(),
            start: Utc::now(),
        }
    }

    pub fn execute(&mut self, req: String) -> String {
        let mut req = req.split(" ");
        match req.next().unwrap() {
            "GET" => match req.next().unwrap() {
                "USER" => {
                    let mut mult = false;
                    let option = match req.next() {
                        Some(o) => {
                            if !["ID", "NAME", "EMAIL", "MULT"].contains(&o) {
                                return String::from("INVALID GET USER <_> COMMAND");
                            }
                            if o == "MULT" {
                                mult = true;
                                match req.next() {
                                    Some(opt) => opt,
                                    None => {
                                        return String::from("NO OPTION OR SEARCH TERM PROVIDED")
                                    }
                                }
                            } else {
                                o
                            }
                        }
                        None => return String::from("NO OPTION PROVIDED"),
                    };
                    let search = match req.next() {
                        Some(s) => s,
                        None => return String::from("NO SEARCH TERM PROVIDED"),
                    };
                    if mult {
                        let users = self.get_user_mult(option, search.trim());
                        let users = match users {
                            Some(usrs) => usrs,
                            None => return String::from("NO USERS FOUND"),
                        };
                        users.iter().fold(String::new(), |acc, u| {
                            let u = u.borrow();
                            format!("{}{}", acc, u.to_string())
                        })
                    } else {
                        let user = self.get_user(option, search.trim());
                        let user = match user {
                            Some(usr) => usr,
                            None => return String::from("NO USER FOUND"),
                        };
                        let user = user.borrow();
                        user.to_string()
                    }
                }

                "CONV" => {
                    let option = req.next().unwrap_or("NONE");
                    let search = req.next().unwrap_or("NONE");
                    let extra = req.next().unwrap_or("NONE");
                    if option == "NONE" || search == "NONE" {
                        return String::from("INCOMPLETE COMMAND");
                    }
                    match option {
                        "NAME" => {
                            let search = ConvSearch::Name(String::from(search.trim()));
                            let result = match self.get_conv(search) {
                                Some(c) => c,
                                None => return String::from("NO CONVS FOUND"),
                            };
                            let result = result.borrow();
                            result.to_string()
                        }
                        "MEMBERS" => {
                            let mut search : Vec<&str> = search.split(|c| c == ',' || c == '\n').collect();
                            println!("{:#?}",search);
                            search.pop();
                            println!("{:#?}",search);
                            if search.len() == 0 {
                                return String::from("NO VALID USERS PROVIDED");
                            }
                            let search : Vec<Option<User>> = search.iter().map(|id| self.get_user("ID",id)).collect();
                            if search.contains(&None) {
                                return String::from("INVALID USER PROVIDED");
                            }
                            let search = search.iter().map(|u| User::clone(u.as_ref().unwrap())).collect();
                            let search = ConvSearch::Members(search);
                            let result = match self.get_conv(search) {
                                Some(c) => c,
                                None => return String::from("NO CONVS FOUND"),
                            };
                            let result = result.borrow();
                            result.to_string()
                        }
                        "MULT" => String::new(),
                        _ => String::new(),
                    }
                }
                "REL" => String::new(),
                "MSG" => String::new(),
                _ => panic!(),
            },

            _ => String::from("NOT GET"),
        }
    }

    pub fn load_rels(&mut self, filename: &str) -> Result<(), &'static str> {
        for line in read_to_string(filename).unwrap().lines() {
            let mut line = line.split(';');
            let mem1 = match line.next() {
                Some(id) => Uuid::parse_str(id).unwrap(),
                None => return Err("Invalid $mem1 in RELS file."),
            };
            let mem2 = match line.next() {
                Some(id) => Uuid::parse_str(id).unwrap(),
                None => return Err("Invalid $mem2 in RELS file."),
            };
            let status = match line.next() {
                Some(stat) => RelStatus::from_str(stat),
                None => return Err("Invalid $status in RELS file."),
            };
            self.rels.push(Relationship::new([mem1, mem2], status));
        }
        Ok(())
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
            let new_user = User::new(RefCell::new(UserInfo::load(id, user, email, create_time)));
            self.users.push(new_user);
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

    pub fn get_rel_status(&mut self, user1: User, user2: User) -> RelStatus {
        let user1 = user1.borrow().id();
        let user2 = user2.borrow().id();

        match self
            .rels
            .iter()
            .find(|r| r.members.contains(&user1) && r.members.contains(&user2))
        {
            Some(rel) => rel.status(),
            None => {
                self.rels.push(Relationship::new([user1, user2], None));
                RelStatus::Neutral
            }
        }
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

    pub fn get_user(&self, option: &str, search: &str) -> Option<User> {
        let mut users = self.users.iter();
        match option {
            "ID" => match users.find(|u| u.borrow().id().to_string() == search.trim()) {
                Some(ur) => Some(Rc::clone(ur)),
                None => None,
            },
            "NAME" => match users.find(|u| {
                u.borrow()
                    .name()
                    .to_lowercase()
                    .contains(&search.to_lowercase())
            }) {
                Some(ur) => Some(Rc::clone(ur)),
                None => None,
            },
            "EMAIL" => match users.find(|u| {
                u.borrow()
                    .email()
                    .to_lowercase()
                    .contains(&search.to_lowercase())
            }) {
                Some(ur) => Some(Rc::clone(ur)),
                None => None,
            },
            _ => None,
        }
    }

    pub fn get_user_mult(&self, option: &str, search: &str) -> Option<Vec<User>> {
        let mut list = Vec::new();
        let users = self.users.iter();
        match option {
            "EMAIL" => users
                .filter(|u| u.borrow().email().to_lowercase().contains(search))
                .for_each(|u| list.push(User::clone(u))),
            "NAME" => users
                .filter(|u| u.borrow().name().to_lowercase().contains(search))
                .for_each(|u| list.push(User::clone(u))),
            _ => {}
        }
        if list.is_empty() {
            None
        } else {
            Some(list)
        }
    }

    pub fn add_conv(&mut self, name: &str, members: Vec<User>) {
        self.convs
            .push(Conversation::new(RefCell::new(ConvInfo::new(
                name, members,
            ))))
    }

    fn get_conv(&self, search: ConvSearch) -> Option<Conversation> {
        match search {
            ConvSearch::Name(name) => {
                match self
                    .convs
                    .iter()
                    .find(|c| c.borrow().name().contains(&name))
                {
                    Some(c) => Some(Conversation::clone(c)),
                    None => None,
                }
            }
            ConvSearch::Members(users) => {
                match self.convs.iter().find(|c| {
                    let c = c.borrow();
                    let mems = c.members();
                    users.iter().all(|u| mems.contains(u))
                }) {
                    Some(c) => Some(Conversation::clone(c)),
                    None => None,
                }
            }
        }
    }

    pub fn get_conv_mult(
        &self,
        name: Option<&str>,
        members: Option<Vec<User>>,
    ) -> Option<Vec<Conversation>> {
        let mut list = Vec::new();
        if name == None && members == None {
            return None;
        } else if name == None && members != None {
            let members = members.unwrap();
            self.convs
                .iter()
                .filter(move |c| members.iter().all(move |m| c.borrow().members.contains(m)))
                .for_each(|c| list.push(Conversation::clone(c)));
        } else if name != None && members == None {
            self.convs
                .iter()
                .filter(|c| c.borrow().name().to_lowercase().contains(name.unwrap()))
                .for_each(|c| list.push(Conversation::clone(c)));
        } else {
            let members = members.unwrap();
            self.convs
                .iter()
                .filter(move |c| {
                    c.borrow().name().to_lowercase().contains(name.unwrap())
                        && members.iter().all(move |m| c.borrow().members.contains(m))
                })
                .for_each(|c| list.push(Conversation::clone(c)));
        }
        if list.is_empty() {
            None
        } else {
            Some(list)
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
        rel_file: &str,
    ) -> Result<(), &'static str> {
        let messages = self
            .msgs
            .drain(..)
            .fold(String::new(), |acc, m| acc + &m.borrow().to_string());
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
        let convs = self
            .convs
            .drain(..)
            .fold(String::new(), |acc, c| acc + &c.borrow().to_string());
        let mut conv_file = match OpenOptions::new().read(true).write(true).open(conv_file) {
            Ok(f) => f,
            Err(e) => {
                println!("{}", e);
                return Err("Error opening conversations file!");
            }
        };
        match conv_file.write_all(convs.as_bytes()) {
            Ok(_) => println!("Conversations file saved successfully!"),
            Err(e) => {
                println!("{}", e);
                return Err("Error writing conversations file!");
            }
        }
        let users = self
            .users
            .drain(..)
            .fold(String::new(), |acc, u| acc + &u.borrow().to_string());
        let mut user_file = match OpenOptions::new().read(true).write(true).open(user_file) {
            Ok(f) => f,
            Err(e) => {
                println!("{}", e);
                return Err("Error opening users file!");
            }
        };
        match user_file.write_all(users.as_bytes()) {
            Ok(_) => println!("Users file saved successfully!"),
            Err(e) => {
                println!("{}", e);
                return Err("Error writing users file!");
            }
        }
        let rels = self
            .rels
            .drain(..)
            .fold(String::new(), |acc, r| acc + &r.to_string());
        let mut rel_file = match OpenOptions::new().read(true).write(true).open(rel_file) {
            Ok(f) => f,
            Err(e) => {
                println!("{}", e);
                return Err("Error opening rels file!");
            }
        };
        match rel_file.write_all(rels.as_bytes()) {
            Ok(_) => println!("Rels file saved successfully!"),
            Err(e) => {
                println!("{}", e);
                return Err("Error writing users file!");
            }
        }
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
    fn to_string(&self) -> String {
        format!(
            "{};{};{};{};{}\n",
            self.id.to_string(),
            self.text,
            self.time_stamp.to_rfc3339(),
            self.user.borrow().id().to_string(),
            self.conv.borrow().id().to_string()
        )
    }

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

// Relationship Struct
//
#[derive(Debug, Copy, Clone)]
pub enum RelStatus {
    BestFriends,
    Friends,
    Neutral,
    Blocked(Uuid),
}

impl RelStatus {
    fn to_string(&self) -> String {
        match self {
            RelStatus::BestFriends => "BestFriends".to_owned(),
            RelStatus::Friends => "Friends".to_owned(),
            RelStatus::Neutral => "Neutral".to_owned(),
            RelStatus::Blocked(id) => format!("Blocked,{}", id.to_string()),
        }
    }

    fn from_str(input: &str) -> Option<RelStatus> {
        let mut input = input.split(',');
        match input.next().unwrap() {
            "BestFriends" => Some(RelStatus::BestFriends),
            "Friends" => Some(RelStatus::Friends),
            "Neutral" => Some(RelStatus::Neutral),
            "Blocked" => {
                let user = Uuid::parse_str(input.next().unwrap()).unwrap();
                Some(RelStatus::Blocked(user))
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Relationship {
    pub members: [Uuid; 2],
    status: RelStatus,
}

impl Relationship {
    fn to_string(&self) -> String {
        format!(
            "{};{};{},\n",
            self.members[0].to_string(),
            self.members[1].to_string(),
            self.status.to_string()
        )
    }

    fn new(members: [Uuid; 2], status: Option<RelStatus>) -> Relationship {
        Relationship {
            members,
            status: {
                match status {
                    Some(stat) => stat,
                    None => RelStatus::Neutral,
                }
            },
        }
    }

    fn status(&self) -> RelStatus {
        self.status
    }

    fn _change_status(&mut self, status: RelStatus) {
        self.status = status;
    }
}

// User Struct
//
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct UserInfo {
    id: Uuid,
    name: String,
    email: String,
    create_time: DateTime<Utc>,
}

pub type User = Rc<RefCell<UserInfo>>;

impl UserInfo {
    pub fn new(name: &str, email: &str) -> UserInfo {
        UserInfo {
            id: Uuid::new_v4(),
            name: name.to_string(),
            email: email.to_string(),
            create_time: Utc::now(),
        }
    }

    pub fn load(id: Uuid, name: &str, email: &str, create_time: DateTime<Utc>) -> UserInfo {
        UserInfo {
            id,
            name: name.to_string(),
            email: email.to_string(),
            create_time,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{};{};{};{}\n",
            self.id.to_string(),
            self.name,
            self.email,
            self.create_time.to_rfc3339()
        )
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

    pub fn to_string(&self) -> String {
        format!(
            "{};{};{};{};{}\n",
            self.id.to_string(),
            self.name,
            &self.members.iter().fold(String::new(), |acc, usr| {
                acc + "," + &usr.borrow().id().to_string()
            })[1..],
            self.start.to_rfc3339(),
            self.last_msg.to_rfc3339()
        )
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

    pub fn members(&self) -> &Vec<User> {
        &self.members
    }
}
