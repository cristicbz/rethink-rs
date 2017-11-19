#[macro_use]
extern crate failure;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate byteorder;
extern crate bufstream;
#[macro_use]
extern crate log;
extern crate ql2_proto;

use failure::ResultExt;
pub use failure::Error;
use std::time::Duration;
use byteorder::{LittleEndian, ByteOrder, BigEndian};
use std::io::{Read, Write, BufReader, Result as IoResult, ErrorKind as IoErrorKind};
use std::net::{TcpStream, ToSocketAddrs};
use serde::Serialize;

pub mod query;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Token(pub u64);

pub struct RawConnection {
    tcp: BufReader<TcpStream>,
    write_buffer: Vec<u8>,
    next_token: u64,
    options: GlobalOptions,
}

impl RawConnection {
    pub fn connect<A: ToSocketAddrs>(address: A) -> Result<Self, Error> {
        let mut tcp = TcpStream::connect_timeout(
            &address
                .to_socket_addrs()
                .context("resolving address error")?
                .next()
                .unwrap(),
            Duration::from_millis(CONNECTION_TIMEOUT_MS),
        ).context("connection error")?;
        tcp.set_read_timeout(Some(Duration::from_millis(MESSAGE_TIMEOUT_MS)))
            .context("set read timeout error")?;
        tcp.set_write_timeout(Some(Duration::from_millis(MESSAGE_TIMEOUT_MS)))
            .context("set write timeout error")?;
        tcp.set_nodelay(true).context("set nodelay error")?;
        tcp.write_all(HANDSHAKE_REQUEST)?;
        tcp.flush()?;

        let mut tcp = BufReader::new(tcp);
        let mut response = [0; HANDSHAKE_RESPONSE_LEN];
        tcp.read_exact(&mut response).context(
            "reading handshake error",
        )?;
        if response != HANDSHAKE_SUCCESS {
            return Err(format_err!("Handshake failed: {:?}", response));
        }

        Ok(RawConnection {
            tcp,
            write_buffer: Vec::with_capacity(32768),
            next_token: 1,
            options: GlobalOptions::default(),
        })
    }

    pub fn start_request<QueryT: Serialize>(&mut self, query: QueryT) -> Result<Token, Error> {
        self.write_buffer.clear();
        self.write_buffer.resize(REQUEST_HEADER_SIZE, 0u8);
        BigEndian::write_u64(&mut self.write_buffer, self.next_token);
        let token = Token(self.next_token);
        self.next_token += 1;

        serde_json::to_writer(&mut self.write_buffer, &(
            ql2_proto::mod_Query::QueryType::START as
                u8,
            query,
            &self.options,
        ))?;
        let request_size = self.write_buffer.len() - REQUEST_HEADER_SIZE;
        assert!(
            request_size < u32::max_value() as usize,
            "Request too large."
        );
        LittleEndian::write_u32(
            &mut self.write_buffer[REQUEST_LENGTH_OFFSET..REQUEST_HEADER_SIZE],
            request_size as u32,
        );
        debug!(
            "Sent request {:?}, size={:?}: {:?}",
            token,
            request_size,
            String::from_utf8_lossy(&self.write_buffer[REQUEST_HEADER_SIZE..]),
            );
        self.tcp.get_mut().write_all(&self.write_buffer)?;
        self.tcp.get_mut().flush()?;
        Ok(token)
    }

    pub fn continue_request(&mut self, token: Token) -> Result<(), Error> {
        let mut request = CONTINUE_REQUEST_TEMPLATE;
        BigEndian::write_u64(&mut request[..REQUEST_LENGTH_OFFSET], token.0);
        self.tcp.get_mut().write_all(&request)?;
        self.tcp.get_mut().flush()?;
        Ok(())
    }

    pub fn recv(&mut self, wait: Wait, result: &mut Vec<u8>) -> Result<Option<Token>, Error> {
        let mut header = [0u8; REQUEST_HEADER_SIZE];
        let header_read_result = match wait {
            Wait::Yes => self.tcp.read_exact(&mut header),
            Wait::No => {
                self.tcp.get_mut().set_nonblocking(true)?;
                let result = self.tcp.read_exact(&mut header);
                self.tcp.get_mut().set_nonblocking(false)?;
                result
            }
            Wait::For(duration) => {
                self.tcp.get_mut().set_read_timeout(Some(duration))?;
                let result = self.tcp.read_exact(&mut header);
                self.tcp.get_mut().set_read_timeout(Some(
                    Duration::from_millis(MESSAGE_TIMEOUT_MS),
                ))?;
                result
            }
        };
        debug!("Received header: {:?}", header);

        if let Err(error) = header_read_result {
            return if error.kind() == IoErrorKind::TimedOut {
                Ok(None)
            } else {
                Err(error.into())
            };
        }

        let (token, size) = header.split_at(REQUEST_LENGTH_OFFSET);
        let token = Token(BigEndian::read_u64(token));
        let size = LittleEndian::read_u32(size) as usize;

        result.clear();
        result.resize(size, 0u8);
        self.tcp.read_exact(result)?;
        Ok(Some(token))
    }
}

#[derive(Serialize, Default)]
pub struct GlobalOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_mode: Option<ReadMode>,
}

#[derive(Serialize)]
pub enum ReadMode {
    #[serde(rename = "single")]
    Single,
    #[serde(rename = "majority")]
    Majority,
    #[serde(rename = "outdated")]
    Outdated,
}

pub enum Wait {
    Yes,
    No,
    For(Duration),
}

#[must_use]
pub struct Sender<'a> {
    data: &'a mut Vec<u8>,
    tcp: &'a TcpStream,
    token: Token,
}

impl<'a> Sender<'a> {
    pub fn send(mut self) -> Result<Token, Error> {
        let request_size = self.data.len() - REQUEST_HEADER_SIZE;
        assert!(
            request_size < u32::max_value() as usize,
            "Request too large."
        );
        LittleEndian::write_u32(
            &mut self.data[REQUEST_LENGTH_OFFSET..REQUEST_HEADER_SIZE],
            request_size as u32,
        );
        debug!(
            "Sent request {:?}, size={:?}: {:?}",
            self.token,
            request_size,
            self.data
        );
        self.tcp.write_all(&*self.data).context("writing request")?;
        self.tcp.flush()?;
        Ok(self.token)
    }
}

impl<'a> Write for Sender<'a> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        Ok(self.data.write(buf).unwrap())
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

const CONTINUE_REQUEST_TEMPLATE: [u8; 15] = [0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, b'[', b'2', b']'];
const REQUEST_HEADER_SIZE: usize = 8 + 4;
const REQUEST_LENGTH_OFFSET: usize = 8;

const CONNECTION_TIMEOUT_MS: u64 = 5000;
const MESSAGE_TIMEOUT_MS: u64 = 30000;

const HANDSHAKE_REQUEST: &[u8] = &[
    0x20,
    0x2d,
    0x0c,
    0x40,
    0x00,
    0x00,
    0x00,
    0x00,
    0xc7,
    0x70,
    0x69,
    0x7e,
];
const HANDSHAKE_RESPONSE_LEN: usize = 8;
const HANDSHAKE_SUCCESS: [u8; HANDSHAKE_RESPONSE_LEN] =
    [0x53, 0x55, 0x43, 0x43, 0x45, 0x53, 0x53, 0x00];

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
