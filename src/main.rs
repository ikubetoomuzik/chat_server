use chat_server::*;

fn main() -> Result<(), &'static str> {
    let mut app = App::new();
    app.load_users("files/users")?;
    app.load_convs("files/convs")?;
    app.load_msgs("files/msgs")?;
    app.load_rels("files/rels")?;

    // let me = app.get_user(Some("Curtis Jones"), None).unwrap();
    // let conv = app.get_conv(Some("Basic"), None).unwrap();

    // app.send_msg(me, conv, "Hello world!")?;

    // let users = app.get_user_mult(Some("s "), Some("mail"));
    // let convs = app.get_conv_mult(Some("bas"), None);

    let curtis = app.get_user("NAME", "Curtis Jones").unwrap();
    println!("{:#?}", curtis);
    let sarah = app.get_user("NAME", "Sarah Parsons").unwrap();

    println!("{:#?}", app.get_rel_status(curtis, sarah));

    TcpServer::listen(8080, &mut app);
    // app.add_user("Curtis Jones", "mail@curtisjones.ca")?;

    app.close("files/msgs", "files/convs", "files/users", "files/rels")
}
