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

    /// Create a new `TcpHandler` wit a default 1s read timeout.
    ///
    /// # Errors
    ///
    /// May fail if server:port is unavaible or read timeout cannot be set.
    pub fn new_with_timeout(
        server_url: &str,
        port: u16,
        timeout: u64,
    ) -> Result<Self, CtfTcpHandlerError> {
        let stream = {
            let connection_uri = format!("{server_url}:{port}");
            TcpStream::connect(connection_uri).map_err(|_| CtfTcpHandlerError::ConnectionError)
        }?;
        stream
            .set_read_timeout(Some(std::time::Duration::from_millis(timeout)))
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
}

type BoxedFunction = Box<dyn Fn(&str) -> Option<String>>;

/// `CtfLoopResponder` is a Builder pattern like to build a loop responder.
///
/// The main function connect to the server and run the same routine on every incoming message.
pub struct CtfLoopResponder<'a> {
    url: Option<&'a str>,
    port: Option<u16>,
    timeout: Option<u64>,
    responder_func: Option<BoxedFunction>,
}

impl<'a> Default for CtfLoopResponder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> CtfLoopResponder<'a> {
    #[must_use]
    /// Build a new empty `CtfLoopResponder`
    pub fn new() -> Self {
        Self {
            url: None,
            port: None,
            timeout: None,
            responder_func: None,
        }
    }

    /// Build a `CtfLoopResponder` for localhost on given port.
    #[must_use]
    pub fn localhost(port: u16) -> Self {
        Self::new().url("localhost").port(port)
    }

    #[must_use]
    /// Set url
    pub fn url(self, url: &'a str) -> Self {
        Self {
            url: Some(url),
            ..self
        }
    }

    #[must_use]
    /// Set port
    pub fn port(self, port: u16) -> Self {
        Self {
            port: Some(port),
            ..self
        }
    }

    #[must_use]
    /// Set timeout
    pub fn timeout(self, timeout: u64) -> Self {
        Self {
            timeout: Some(timeout),
            ..self
        }
    }

    /// Set the responder routine runned on each server's message.
    #[must_use]
    pub fn responder_func(self, responder_func: impl Fn(&str) -> Option<String> + 'static) -> Self {
        Self {
            responder_func: Some(Box::new(responder_func)),
            ..self
        }
    }

    /// Connect to the server and use the struct routine to answer each incoming message.
    ///
    /// # Errors
    ///
    /// The function will fail if either url, port or responder routine is not defined.
    /// It may also fails if TCP connection fail.
    pub fn connect_and_work(&self) -> Result<String, CtfTcpHandlerError> {
        let url = self.url.ok_or(CtfTcpHandlerError::ConfigurationError)?;
        let port = self.port.ok_or(CtfTcpHandlerError::ConfigurationError)?;
        let responder = self
            .responder_func
            .as_ref()
            .ok_or(CtfTcpHandlerError::ConfigurationError)?;
        let mut tcp_handler = self
            .timeout
            .map_or_else(
                || TcpHandler::new(url, port),
                |timeout| TcpHandler::new_with_timeout(url, port, timeout),
            )
            .map_err(|_| CtfTcpHandlerError::ConnectionError)?;

        let mut input = loop {
            let input = tcp_handler.read_to_string();
            log::debug!("Received:\n{input}");
            if let Some(answer) = responder(&input) {
                log::debug!("Answered: {answer}");
                tcp_handler.write_answer(&answer);
            } else {
                break input;
            }
        };
        input.push_str(&tcp_handler.read_to_string());
        Ok(input)
    }
}
