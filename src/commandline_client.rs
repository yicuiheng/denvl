use crate::consts;
use crate::named_pipe::{self, NamedPipeClient, NamedPipeServer};
use std::path::PathBuf;

pub fn run(filename: &str) {
    let name = PathBuf::from(consts::COMMON_NAME);
    let is_already_exists = NamedPipeServer::is_exists(&name);
    if is_already_exists.is_err() || !is_already_exists.unwrap() {
        println!("launching server..");
        let exe_path = std::env::current_exe().unwrap();
        let status = std::process::Command::new(exe_path)
            .arg("__server")
            .status()
            .unwrap();
        if !status.success() {
            panic!("failed to launch server..");
        }

        // サーバーの起動が完了したことを確認
        let server2client = named_pipe::make_server_to_client_pipename(&name);
        let mut file = std::fs::File::options()
            .read(true)
            .write(false)
            .open(&server2client)
            .unwrap();
        let line = named_pipe::readline(&mut file).unwrap();
        assert_eq!(consts::SERVER_STARTING_HEADER, line);
        println!("done.");
    }

    let mut input_filepath = std::env::current_dir().unwrap();
    input_filepath = input_filepath.join(filename);
    let input_filepath = input_filepath.into_os_string().into_string().unwrap();

    let mut client = NamedPipeClient::try_connect(name).unwrap();

    client.writeline(input_filepath).unwrap();
    loop {
        let line = client.readline().unwrap();
        if line == "done" {
            break;
        }
        eprintln!("{line}");
    }
}

pub fn shutdown() {
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

#[allow(dead_code)]
fn create_random_pipename() -> PathBuf {
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    let rng = thread_rng();
    let random_hash: String = rng
        .sample_iter(Alphanumeric)
        .take(16)
        .map(|c| c as char)
        .collect();
    let base = PathBuf::from(consts::COMMON_NAME);
    base.join(random_hash)
}
