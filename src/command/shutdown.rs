pub fn exec() {
    let name = PathBuf::from(consts::COMMON_NAME);
    let is_already_exists = NamedPipeServer::is_exists(&name);
    if is_already_exists.is_err() || !is_already_exists.unwrap() {
        eprintln!("server has not launched yet..");
        return;
    }

    let mut client = NamedPipeClient::try_connect(name).unwrap();

    client.writeline("shutdown".to_string()).unwrap();
    loop {
        let line = client.readline().unwrap();
        if line == "done" {
            break;
        }
        println!("{line}");
    }
}
