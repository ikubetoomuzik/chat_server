use chat_server::*;

fn main() -> Result<(), &'static str> {
    let mut app = App::new();
    app.load_users("users")?;
    app.load_convs("convs")?;
    app.load_msgs("msgs")?;

    println!("{:#?}", app);

    app.users[0].borrow().send_msg(&app);

    Ok(())
}
