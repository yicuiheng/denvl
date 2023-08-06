// UNIX FIFO のラッパー

use std::fs::File;
use std::path::{Path, PathBuf};

use nix::errno::Errno;

use super::error::{Error, Result};

#[derive(Debug, PartialEq, Eq)]
pub struct NamedPipe {
    path: PathBuf,
}

impl NamedPipe {
    pub fn is_exists(path: &Path) -> Result<bool> {
        match nix::sys::stat::stat(path) {
            Ok(stat) => {
                let mode = stat.st_mode & nix::sys::stat::SFlag::S_IFMT.bits();
                if mode == nix::sys::stat::SFlag::S_IFIFO.bits() {
                    Ok(true)
                } else {
                    Err(Error::UnexpectedFileMode)
                }
            }
            Err(errno)
                if errno == Errno::EACCES || errno == Errno::ENOENT || errno == Errno::ENOTDIR =>
            {
                Ok(false)
            }
            Err(errno)
                if errno == Errno::EFAULT
                    || errno == Errno::ELOOP
                    || errno == Errno::ENAMETOOLONG
                    || errno == Errno::ENOMEM
                    || errno == Errno::EOVERFLOW =>
            {
                Err(Error::System(errno))
            }
            Err(_) => unreachable!(),
        }
    }

    pub fn check_existence(path: PathBuf) -> Result<Option<Self>> {
        if NamedPipe::is_exists(path.as_path())? {
            Ok(Some(NamedPipe { path }))
        } else {
            Ok(None)
        }
    }

    pub fn create(path: PathBuf) -> Result<Self> {
        nix::unistd::mkfifo(path.as_path(), nix::sys::stat::Mode::S_IRWXU)?;
        Ok(NamedPipe { path })
    }

    pub fn remove(self) -> Result<()> {
        std::fs::remove_file(self.path).map_err(Error::Io)
    }
}

impl std::io::Read for NamedPipe {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut file = File::options()
            .read(true)
            .write(false)
            .open(self.path.as_path())?;
        file.read(buf)
    }
}

impl std::io::Write for NamedPipe {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut file = File::options()
            .read(false)
            .write(true)
            .append(true)
            .open(self.path.as_path())?;
        file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
