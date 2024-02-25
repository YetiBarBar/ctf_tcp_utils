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
    /// Create a new `TcpHandler` wit a default 1s read timeout.
    ///
    /// # Errors
    ///
    /// May fail if server:port is unavaible or read timeout cannot be set.
    pub fn new(server_url: &str, port: u16) -> Result<Self, CtfTcpHandlerError> {
        let stream = {
            let connection_uri = format!("{server_url}:{port}");
            TcpStream::connect(connection_uri).map_err(|_| CtfTcpHandlerError::ConnectionError)
        }?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_millis(1000)))
            .map_err(|_| CtfTcpHandlerError::ConfigurationError)?;

        Ok(Self { stream })
    }

    /// Read TCP stream until read timeout is reached. Always produces a String using UTF-8 lossy conversion.
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

    /// Read TCP stream until read timeout is reached. Always produces a String using UTF-8 lossy conversion.
    pub fn write_answer(&mut self, answer: &str) {
        let data = format!("{answer}\n");
        let _size = self.stream.write(data.as_bytes());
    }

    /// Set read timeout.
    ///
    /// timeout are milliseconds, given with a `u64`
    ///
    /// # Errors
    ///
    /// Fail if read timeout cannot be set.
    pub fn set_timeout(&mut self, timeout: u64) -> Result<(), CtfTcpHandlerError> {
        self.stream
            .set_read_timeout(Some(std::time::Duration::from_millis(timeout)))
            .map_err(|_| CtfTcpHandlerError::ConfigurationError)?;

        Ok(())
    }
}

/// Connect to a CTF TCP server and process the same function in loop.
///
/// # Errors
///
/// Fails if connection fail
pub fn run_function_loop(
    url: &str,
    port: u16,
    function: impl Fn(&str) -> Option<String>,
) -> Result<String, CtfTcpHandlerError> {
    let mut tcp_handle =
        TcpHandler::new(url, port).map_err(|_| CtfTcpHandlerError::ConnectionError)?;
    loop {
        let input = tcp_handle.read_to_string();
        println!("{input}");
        if let Some(answer) = function(&input) {
            println!("{answer}");
            tcp_handle.write_answer(&answer);
        } else {
            break;
        }
    }
    Ok(tcp_handle.read_to_string())
}
