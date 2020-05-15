use chat_server::*;

fn main() -> Result<(), &'static str> {
    let mut app = App::new();
    app.load_users("users")?;
    app.load_convs("convs")?;
    app.load_msgs("msgs")?;

    let me = app.get_user(Some("Curtis Jones"), None).unwrap();
    let conv = app.get_conv(Some("Basic"), None).unwrap();
    
    app.send_msg(me, conv, "Hello world!")?;

    println!("{:#?}", app);

    // app.add_user("Curtis Jones", "mail@curtisjones.ca")?;

    app.close("msgs", "convs", "users")
}
