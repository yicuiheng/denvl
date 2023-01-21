use nix::unistd;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

// UNIX FIFO
#[derive(Debug)]
pub struct NamedPipeServer {
    name: PathBuf,
}

#[derive(Debug)]
pub enum ServerCreationError {
    IO(std::io::Error),
    System(nix::Error),
}

impl From<std::io::Error> for ServerCreationError {
    fn from(value: std::io::Error) -> Self {
        ServerCreationError::IO(value)
    }
}

impl From<nix::Error> for ServerCreationError {
    fn from(value: nix::Error) -> Self {
        ServerCreationError::System(value)
    }
}

impl NamedPipeServer {
    pub fn is_exists(name: &Path) -> Result<bool, nix::Error> {
        let PipenamePair {
            server2client: pipename1,
            client2server: pipename2,
        } = make_pipename_pair(name);

        let stat = nix::sys::stat::stat(pipename1.as_path())?;
        let is_pipename1_exists = stat.st_mode & nix::sys::stat::SFlag::S_IFMT.bits()
            == nix::sys::stat::SFlag::S_IFIFO.bits();
        let stat = nix::sys::stat::stat(pipename2.as_path())?;
        let is_pipename2_exists = stat.st_mode & nix::sys::stat::SFlag::S_IFMT.bits()
            == nix::sys::stat::SFlag::S_IFIFO.bits();
        Ok(is_pipename1_exists && is_pipename2_exists)
    }

    pub fn create(name: PathBuf) -> Result<Self, ServerCreationError> {
        let PipenamePair {
            server2client,
            client2server,
        } = make_pipename_pair(&name);
        unistd::mkfifo(server2client.as_path(), nix::sys::stat::Mode::S_IRWXU)?;
        unistd::mkfifo(client2server.as_path(), nix::sys::stat::Mode::S_IRWXU)?;
        Ok(Self { name })
    }

    pub fn readline(&mut self) -> Result<String, std::io::Error> {
        let pipename = make_client_to_server_pipename(&self.name);
        let mut file = File::options().read(true).write(false).open(pipename)?;
        let result = readline(&mut file)?;
        Ok(result)
    }

    pub fn writeline(&mut self, line: String) -> Result<(), std::io::Error> {
        let pipename = make_server_to_client_pipename(&self.name);
        let mut file = File::options()
            .read(false)
            .write(true)
            .append(true)
            .open(pipename)?;
        writeline(&mut file, line)?;
        Ok(())
    }
}

impl Drop for NamedPipeServer {
    fn drop(&mut self) {
        let PipenamePair {
            server2client: pipename1,
            client2server: pipename2,
        } = make_pipename_pair(&self.name);
        std::fs::remove_file(pipename1).unwrap();
        std::fs::remove_file(pipename2).unwrap();
    }
}

pub struct NamedPipeClient {
    name: PathBuf,
}

#[derive(Debug)]
pub enum ClientConnectionError {
    System(nix::Error),
    ServerNotFound,
}

impl From<nix::Error> for ClientConnectionError {
    fn from(value: nix::Error) -> Self {
        ClientConnectionError::System(value)
    }
}

impl NamedPipeClient {
    pub fn try_connect(name: PathBuf) -> Result<NamedPipeClient, ClientConnectionError> {
        if !NamedPipeServer::is_exists(&name)? {
            return Err(ClientConnectionError::ServerNotFound);
        }
        Ok(NamedPipeClient { name })
    }

    pub fn readline(&mut self) -> Result<String, std::io::Error> {
        let pipename = make_server_to_client_pipename(&self.name);
        let mut file = File::options().read(true).write(false).open(pipename)?;
        let result = readline(&mut file)?;
        Ok(result)
    }

    pub fn writeline(&mut self, line: String) -> Result<(), std::io::Error> {
        let pipename = make_client_to_server_pipename(&self.name);
        let mut file = File::options()
            .read(false)
            .write(true)
            .append(true)
            .open(pipename)?;
        writeline(&mut file, line)?;
        Ok(())
    }
}

struct PipenamePair {
    pub server2client: PathBuf,
    pub client2server: PathBuf,
}

fn make_pipename_pair(name: &Path) -> PipenamePair {
    std::fs::create_dir_all(name).unwrap();
    PipenamePair {
        server2client: make_server_to_client_pipename(name),
        client2server: make_client_to_server_pipename(name),
    }
}

pub fn make_server_to_client_pipename(name: &Path) -> PathBuf {
    name.join("server_to_client")
}

fn make_client_to_server_pipename(name: &Path) -> PathBuf {
    name.join("client_to_server")
}

pub fn readline(file: &mut File) -> Result<String, std::io::Error> {
    let mut result = String::new();
    loop {
        let mut buf: [char; 1] = [' '];
        unsafe {
            file.read_exact(std::mem::transmute(buf.as_mut_slice()))?;
        }
        if buf[0] == '\n' {
            break;
        } else {
            result.push(buf[0]);
        }
    }
    Ok(result)
}

pub fn writeline(file: &mut File, line: String) -> Result<(), std::io::Error> {
    let line = format!("{line}\n");
    file.write_all(line.as_bytes())?;
    Ok(())
}
