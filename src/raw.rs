use byteorder::{BigEndian, ByteOrder, LittleEndian};
use errors::{ErrorKind, Result};
use failure::ResultExt;
use serde::Serialize;
use serde_json;
use std::io::{BufReader, ErrorKind as IoErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Token(pub u64);

#[derive(Debug)]
pub struct RawConnection {
    endpoint: SocketAddr,
    tcp: BufReader<TcpStream>,
    write_buffer: Vec<u8>,
    next_token: u64,
    options: GlobalOptions,
}

impl RawConnection {
    pub fn connect(endpoint: SocketAddr) -> Result<Self> {
        Ok(RawConnection {
            endpoint,
            tcp: handshake(&endpoint)?,
            write_buffer: Vec::with_capacity(4096),
            next_token: 1,
            options: GlobalOptions::default(),
        })
    }

    pub fn close(&self) -> Result<()> {
        match self.tcp.get_ref().shutdown(Shutdown::Both) {
            Err(ref error) if error.kind() == IoErrorKind::NotConnected => Ok(()),
            result @ _ => {
                result.context(ErrorKind::Connection("failed to close socket".into()))?;
                Ok(())
            }
        }
    }

    pub fn reset(&mut self) -> Result<()> {
        self.next_token = 1;
        match handshake(&self.endpoint) {
            Ok(tcp) => {
                self.tcp = tcp;
                Ok(())
            }
            Err(error) => {
                let _ = self.tcp.get_mut().shutdown(Shutdown::Both);
                Err(error.into())
            }
        }
    }

    pub fn is_open(&mut self) -> bool {
        let tcp = self.tcp.get_mut();
        tcp.set_nonblocking(true)
            .and_then(|_| {
                let is_ok = match tcp.read(&mut []) {
                    Ok(_) => true,
                    Err(error) => error.kind() == IoErrorKind::WouldBlock,
                };
                tcp.set_nonblocking(false).map(|_| is_ok)
            })
            .unwrap_or(false)
    }

    pub fn start_request<QueryT: Serialize>(&mut self, query: QueryT) -> Result<Token> {
        self.write_buffer.clear();
        self.write_buffer.resize(REQUEST_HEADER_SIZE, 0u8);
        BigEndian::write_u64(&mut self.write_buffer, self.next_token);
        let token = Token(self.next_token);
        self.next_token += 1;

        serde_json::to_writer(
            &mut self.write_buffer,
            &(::enums::query::START, query, &self.options),
        ).context(ErrorKind::Connection("failed to serialize request".into()))?;
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
        self.tcp
            .get_mut()
            .write_all(&self.write_buffer)
            .context(ErrorKind::Connection("failed to send start request".into()))?;
        self.tcp.get_mut().flush().context(ErrorKind::Connection(
            "failed to flush start request".into(),
        ))?;
        Ok(token)
    }

    pub fn continue_request(&mut self, token: Token) -> Result<()> {
        let mut request = CONTINUE_REQUEST_TEMPLATE;
        BigEndian::write_u64(&mut request[..REQUEST_LENGTH_OFFSET], token.0);
        self.tcp
            .get_mut()
            .write_all(&request)
            .context(ErrorKind::Connection(
                "failed to send continue request".into(),
            ))?;
        self.tcp.get_mut().flush().context(ErrorKind::Connection(
            "failed to flush continue request".into(),
        ))?;
        Ok(())
    }

    pub fn recv<'a, F: FnOnce(Token) -> &'a mut Vec<u8>>(
        &mut self,
        wait: Wait,
        buffer: F,
    ) -> Result<Option<Token>> {
        let mut header = [0u8; REQUEST_HEADER_SIZE];
        let header_read_result = match wait {
            Wait::Yes => self.tcp.read_exact(&mut header),
            Wait::No => {
                self.tcp
                    .get_mut()
                    .set_nonblocking(true)
                    .context(ErrorKind::Connection("failed to set nonblocking".into()))?;
                let result = self.tcp.read_exact(&mut header);
                self.tcp
                    .get_mut()
                    .set_nonblocking(false)
                    .context(ErrorKind::Connection("failed to unset nonblocking".into()))?;
                result
            }
            Wait::For(duration) => {
                self.tcp
                    .get_mut()
                    .set_read_timeout(Some(duration))
                    .context(ErrorKind::Connection("failed to set read timeout".into()))?;
                let result = self.tcp.read_exact(&mut header);
                self.tcp
                    .get_mut()
                    .set_read_timeout(Some(Duration::from_millis(MESSAGE_TIMEOUT_MS)))
                    .context(ErrorKind::Connection("failed to reset read timeout".into()))?;
                result
            }
        };
        debug!("Received header: {:?}", header);

        match header_read_result {
            Err(ref error) if error.kind() == IoErrorKind::WouldBlock => {
                return Ok(None);
            }
            result @ _ => result.context(ErrorKind::Connection("failed to read header".into()))?,
        }

        let (token, size) = header.split_at(REQUEST_LENGTH_OFFSET);
        let token = Token(BigEndian::read_u64(token));
        let size = LittleEndian::read_u32(size);
        debug!("Header: token={:?} size={}", token, size);

        let buffer = buffer(token);
        let initial_buffer_len = buffer.len();
        let final_buffer_len = initial_buffer_len + 4 + size as usize;
        buffer.resize(final_buffer_len, 0u8);
        BigEndian::write_u32(&mut buffer[initial_buffer_len..], size);
        self.tcp
            .read_exact(&mut buffer[initial_buffer_len + 4..])
            .context(ErrorKind::Connection("failed to read response body".into()))?;
        Ok(Some(token))
    }
}

fn handshake(endpoint: &SocketAddr) -> Result<BufReader<TcpStream>> {
    let mut tcp =
        TcpStream::connect_timeout(&endpoint, Duration::from_millis(CONNECTION_TIMEOUT_MS))
            .context(ErrorKind::Connection("connection error".into()))?;
    tcp.set_read_timeout(Some(Duration::from_millis(MESSAGE_TIMEOUT_MS)))
        .context(ErrorKind::Connection("set read timeout error".into()))?;
    tcp.set_write_timeout(Some(Duration::from_millis(MESSAGE_TIMEOUT_MS)))
        .context(ErrorKind::Connection("set write timeout error".into()))?;
    tcp.set_nodelay(true)
        .context(ErrorKind::Connection("set nodelay error".into()))?;
    tcp.write_all(HANDSHAKE_REQUEST)
        .context(ErrorKind::Connection("error sending handshake".into()))?;
    tcp.flush()
        .context(ErrorKind::Connection("error flushing handshake".into()))?;

    let mut tcp = BufReader::new(tcp);
    let mut response = [0; HANDSHAKE_RESPONSE_LEN];
    tcp.read_exact(&mut response)
        .context(ErrorKind::Connection(
            "error reading handshake response".into(),
        ))?;

    if response == HANDSHAKE_SUCCESS {
        Ok(tcp)
    } else {
        Err(
            ErrorKind::Connection(format!("handshake failed, response: {:?}", response).into())
                .into(),
        )
    }
}

#[derive(Debug, Serialize, Default)]
pub struct GlobalOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_mode: Option<ReadMode>,
}

#[derive(Debug, Serialize)]
pub enum ReadMode {
    #[serde(rename = "single")]
    Single,
    #[serde(rename = "majority")]
    Majority,
    #[serde(rename = "outdated")]
    Outdated,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Wait {
    Yes,
    No,
    For(Duration),
}

const CONTINUE_REQUEST_TEMPLATE: [u8; 15] = [0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, b'[', b'2', b']'];
const REQUEST_HEADER_SIZE: usize = 8 + 4;
const REQUEST_LENGTH_OFFSET: usize = 8;

const CONNECTION_TIMEOUT_MS: u64 = 5000;
const MESSAGE_TIMEOUT_MS: u64 = 30000;

const HANDSHAKE_REQUEST: &[u8] = &[
    0x20, 0x2d, 0x0c, 0x40, 0x00, 0x00, 0x00, 0x00, 0xc7, 0x70, 0x69, 0x7e,
];
const HANDSHAKE_RESPONSE_LEN: usize = 8;
const HANDSHAKE_SUCCESS: [u8; HANDSHAKE_RESPONSE_LEN] =
    [0x53, 0x55, 0x43, 0x43, 0x45, 0x53, 0x53, 0x00];
