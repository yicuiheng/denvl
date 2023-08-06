// NamedPipe の Client 側と Server 側のペア
// クライアントから見てもサーバから見ても読み書きできるパイプを提供
use std::path::PathBuf;

use super::error::Result;
use super::named_pipe::NamedPipe;

const COMMON_CLIENT2SERVER_PIPE_PATH: PathBuf =
    PathBuf::from("/tmp/denvl/common.client2server.pipe");
const COMMON_SERVER2CLIENT_PIPE_PATH: PathBuf =
    PathBuf::from("/tmp/denvl/common.server2client.pipe");

pub struct NamedPipePair {
    client2server: NamedPipe,
    server2client: NamedPipe,
}

pub struct Stream<'a> {
    pub input: &'a mut NamedPipe,
    pub output: &'a mut NamedPipe,
}

impl NamedPipePair {
    pub fn to_client_stream<'a>(&'a mut self) -> Stream<'a> {
        Stream {
            input: &mut self.server2client,
            output: &mut self.client2server,
        }
    }
    pub fn to_server_stream<'a>(&'a mut self) -> Stream<'a> {
        Stream {
            input: &mut self.client2server,
            output: &mut self.server2client,
        }
    }

    // サーバから呼ばれる
    pub fn create_common() -> Result<NamedPipePair> {
        let client2server = NamedPipe::create(PathBuf::from(COMMON_CLIENT2SERVER_PIPE_PATH));
        let server2client = NamedPipe::create(PathBuf::from(COMMON_SERVER2CLIENT_PIPE_PATH));
        let (client2server, server2client) = match (client2server, server2client) {
            (Ok(client2server), Ok(server2client)) => (client2server, server2client),
            (Ok(pipe), Err(err)) | (Err(err), Ok(pipe)) => {
                pipe.remove()?;
                return Err(err);
            }
            (Err(err), _) => {
                return Err(err);
            }
        };
        Ok(NamedPipePair {
            client2server,
            server2client,
        })
    }

    // クライアントから呼ばれる
    pub fn check_common_existence() -> Result<Option<NamedPipePair>> {
        let client2server = NamedPipe::check_existence(COMMON_CLIENT2SERVER_PIPE_PATH)?;
        let server2client = NamedPipe::check_existence(COMMON_SERVER2CLIENT_PIPE_PATH)?;
        match (client2server, server2client) {
            (Some(client2server), Some(server2client)) => Ok(Some(NamedPipePair {
                client2server,
                server2client,
            })),
            (None, None) => Ok(None),
            (_, _) => unreachable!(),
        }
    }
}
