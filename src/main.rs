use chat_server::*;
use colored::*;
use std::io;

fn main() -> Result<(), &'static str> {
    let mut app = App::new();
    app.load_users("users")?;
    app.load_convs("convs")?;
    app.load_msgs("msgs")?;

    println!("{:#?}", app);

    app.users[0].send_msg(&app);

    Ok(())
}
