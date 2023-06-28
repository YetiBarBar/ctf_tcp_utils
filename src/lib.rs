use std::io::{Read, Write};
use std::net::TcpStream;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CtfTcpHandlerError {
    #[error("Unable to connect to remote")]
    ConnectionError,
    #[error("Unable to set timeout")]
    ConfigurationError,
    #[error("Read error")]
    ReadError,
}

pub struct TcpHandler {
    stream: TcpStream,
}

impl TcpHandler {
    pub fn new(url: &str, port: u16) -> Result<Self, CtfTcpHandlerError> {
        let stream = {
            let uri = format!("{url}:{port}");
            TcpStream::connect(uri).map_err(|_| CtfTcpHandlerError::ConnectionError)
        }?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_millis(1000)))
            .map_err(|_| CtfTcpHandlerError::ConfigurationError)?;

        Ok(Self { stream })
    }

    pub fn read_to_string(&mut self) -> String {
        let mut res = String::new();
        let mut buf = vec![0; 4096];

        loop {
            let size = self.stream.read(&mut buf).unwrap_or(0);
            if size == 0 {
                break;
            }
            let my_str = std::str::from_utf8(&buf[..size]).unwrap_or_default();
            res = format!("{res}{my_str}");
        }
        res
    }

    pub fn write_answer(&mut self, answer: &str) {
        let data = format!("{}\n", answer);
        let _size = self.stream.write(data.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(true)
    }
}
