use chat_server::*;

fn main() -> Result<(), &'static str> {
    let mut app = App::new();
    app.load_users("users")?;
    app.load_convs("convs")?;
    app.load_msgs("msgs")?;
    app.load_rels("rels")?;

    // let me = app.get_user(Some("Curtis Jones"), None).unwrap();
    // let conv = app.get_conv(Some("Basic"), None).unwrap();

    // app.send_msg(me, conv, "Hello world!")?;

    // let users = app.get_user_mult(Some("s "), Some("mail"));
    // let convs = app.get_conv_mult(Some("bas"), None);

    let curtis = app.get_user("NAME","Curtis Jones").unwrap();
    println!("{:#?}", curtis);
    let sarah = app.get_user("NAME","Sarah Parsons").unwrap();

    println!("{:#?}", app.get_rel_status(curtis, sarah));

    TcpServer::listen(8080, &mut app);
    // app.add_user("Curtis Jones", "mail@curtisjones.ca")?;

    app.close("msgs", "convs", "users", "rels")
}
