use chat_server::*;
use colored::*;
use std::io;

fn get_input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line.");
    input
}

fn get_users() -> Vec<User> {
    let mut users = Vec::new();
    println!("Enter names and email seperated by commas, then <{}> when done.", "exit".red().bold().on_cyan());
    loop {
        println!("User {}: ", users.len() + 1);
        let input = get_input();
        if input.trim() == "exit" {
            break;
        } else {
            let input: Vec<&str> = input.split(',').collect();
            users.push(User::new(
                input[0].trim(),
                input[1].trim(),
            ));
        }
    }
    users
}
    
fn main() {
    let users = get_users();
    let mut convo = Conversation::new(users.clone());

    println!("Conversation created with all users!");
    println!("Enter text to send here: ");

    users[0].send_msg(&mut convo, get_input().trim().to_string());
    println!("{}", convo);
}
