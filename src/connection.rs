use failure::Error;
use byteorder::{ByteOrder, BigEndian};
use serde::Serialize;
use serde_json;
use super::raw::{RawConnection, Token, Wait};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use serde::de::DeserializeOwned;
use std::str::{self, FromStr};

pub struct Connection {
    raw: RawConnection,
    connection_id: ConnectionId,
    num_resets: usize,
    responses: HashMap<Token, Vec<u8>>,
    buffers: Vec<Vec<u8>>,
}

impl Connection {
    pub fn from_raw(raw: RawConnection) -> Self {
        Connection {
            raw,
            connection_id: ConnectionId::new(),
            num_resets: 0,
            buffers: Vec::new(),
            responses: HashMap::new(),
        }
    }

    pub fn run<QueryT: Serialize>(&mut self, query: QueryT) -> Result<Cursor, Error> {
        Ok(Cursor {
            token: self.raw.start_request(query)?,
            connection_id: self.connection_id,
            num_resets: 0,
            buffer: None,
            position: 0,
            exhausted: false,
        })
    }

    pub fn next<PayloadT: DeserializeOwned>(
        &mut self,
        wait: Wait,
        cursor: &mut Cursor,
    ) -> Result<Option<PayloadT>, Error> {
        assert_eq!(
            cursor.connection_id,
            self.connection_id,
            "Used a cursor from a different connection."
        );
        if cursor.exhausted || cursor.num_resets != self.num_resets {
            return Err(format_err!("Cursor was closed."));
        }

        if cursor.buffer.is_none() {
            cursor.buffer = self.recv(cursor.token, wait)?;
        }

        let mut buffer_exhausted = false;
        let result = if let Some(buffer) = cursor.buffer.as_mut() {
            let buffer = &buffer[cursor.position..];
            let size = BigEndian::read_u32(&buffer) as usize;
            let content_start = cursor.position + 4;
            let content_end = content_start + size;
            buffer_exhausted = content_end == buffer.len();
            cursor.position = content_end;
            if size > buffer.len() {
                Err(format_err!("Buffer underrun."))
            } else {
                match extract_from_response(&buffer[content_start..content_end]) {
                    Ok((payload, complete)) => {
                        if complete == Complete::Yes {
                            cursor.exhausted = true;
                        }
                        Ok(Some(payload))
                    }
                    Err(error) => Err(error.into()),
                }
            }
        } else {
            Ok(None)
        };

        if result.is_err() || buffer_exhausted || cursor.exhausted {
            let mut buffer = cursor.buffer.take().unwrap();
            buffer.clear();
            reclaim(&mut self.buffers, buffer);
        }

        result
    }

    fn recv(&mut self, token: Token, wait: Wait) -> Result<Option<Vec<u8>>, Error> {
        if let Some(response_buffer) = self.responses.remove(&token) {
            return Ok(Some(response_buffer));
        }

        let mut buffer = self.buffers.pop().unwrap_or_else(Vec::new);
        // TODO(cristicbz): wait is incorrect.
        let response_token = {
            let responses = &mut self.responses;
            self.raw.recv(
                wait,
                |response_token| if response_token == token {
                    &mut buffer
                } else {
                    responses.entry(response_token).or_insert_with(Vec::new)
                },
            )
        };

        match response_token {
            Ok(Some(response_token)) if response_token == token => Ok(Some(buffer)),
            Ok(_) => {
                reclaim(&mut self.buffers, buffer);
                Ok(None)
            }
            Err(error) => {
                reclaim(&mut self.buffers, buffer);
                for (_, mut buffer) in self.responses.drain() {
                    buffer.clear();
                    reclaim(&mut self.buffers, buffer);
                }
                self.num_resets += 1;
                Err(error.into())
            }
        }
    }
}

#[derive(PartialEq, Eq)]
enum Complete {
    Yes,
    No,
}

fn extract_from_response<PayloadT: DeserializeOwned>(
    buffer: &[u8],
) -> Result<(PayloadT, Complete), Error> {
    debug!("Response: {}", String::from_utf8_lossy(buffer));
    if !buffer.starts_with(RESPONSE_PREFIX) {
        return Err(format_err!(
            "Unexpected start of response: {}",
            String::from_utf8_lossy(&buffer[..RESPONSE_TYPE_START])
        ));
    }
    let comma_position = match buffer[RESPONSE_TYPE_START..].iter().position(
        |&x| x == b',',
    ) {
        Some(comma_position) => comma_position + RESPONSE_TYPE_START,
        None => {
            return Err(format_err!(
                "Comma missing in response: {}",
                String::from_utf8_lossy(&buffer[RESPONSE_TYPE_START..])
            ));
        }
    };
    let response_type = u32::from_str(str::from_utf8(
        &buffer[RESPONSE_TYPE_START..comma_position],
    )?)?;
    match response_type {
        SUCCESS_ATOM => {
            let response: AtomResponse<PayloadT> = serde_json::from_slice(buffer)?;
            Ok((response.payload.0, Complete::Yes))
        }
        SUCCESS_PARTIAL => {
            let response: SequenceResponse<PayloadT> = serde_json::from_slice(buffer)?;
            Ok((response.payload, Complete::No))
        }
        SUCCESS_SEQUENCE => {
            let response: SequenceResponse<PayloadT> = serde_json::from_slice(buffer)?;
            Ok((response.payload, Complete::Yes))
        }
        CLIENT_ERROR | RUNTIME_ERROR | COMPILE_ERROR => {
            let response: ErrorResponse = serde_json::from_slice(buffer)?;

            Err(format_err!(
                "{} error code={:?} span={:?}: {}",
                match response_type {
                    CLIENT_ERROR => "Client",
                    RUNTIME_ERROR => "Runtime",
                    COMPILE_ERROR => "Compile",
                    _ => unreachable!(),
                },
                response.error_code,
                response.span,
                response.message.0
            ))
        }
        _ => {
            Err(format_err!(
                "Unexpected response type: {}",
                response_type,
            ))
        }
    }
}

const RESPONSE_PREFIX: &[u8] = b"{\"t\":";
const RESPONSE_TYPE_START: usize = 5; // After the prefix.


const SUCCESS_ATOM: u32 = 1;
const SUCCESS_SEQUENCE: u32 = 2;
const SUCCESS_PARTIAL: u32 = 3;
//const WAIT_COMPLETE: u32 = 4;
//const SERVER_INFO: u32 = 5;
const CLIENT_ERROR: u32 = 16;
const COMPILE_ERROR: u32 = 17;
const RUNTIME_ERROR: u32 = 18;


#[derive(Deserialize)]
struct AtomResponse<PayloadT> {
    #[serde(rename = "r")]
    payload: (PayloadT,),
}

#[derive(Deserialize)]
struct SequenceResponse<PayloadT> {
    #[serde(rename = "r")]
    payload: PayloadT,
}

#[derive(Deserialize)]
struct ErrorResponse {
    #[serde(rename = "e")]
    error_code: Option<u32>,

    #[serde(rename = "b")]
    span: Vec<u32>,

    #[serde(rename = "r")]
    message: (String,),
}


pub struct Cursor {
    token: Token,
    connection_id: ConnectionId,
    exhausted: bool,
    buffer: Option<Vec<u8>>,
    position: usize,
    num_resets: usize,
}

fn reclaim(buffers: &mut Vec<Vec<u8>>, buffer: Vec<u8>) {
    assert!(buffer.is_empty());
    if buffer.capacity() <= MAX_BUFFER_CAPACITY && buffers.len() < MAX_NUM_BUFFERS {
        buffers.push(buffer);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ConnectionId(usize);

impl ConnectionId {
    pub fn new() -> Self {
        ConnectionId(NEXT_CONNECTION_ID.fetch_add(1, Ordering::SeqCst))
    }
}

static NEXT_CONNECTION_ID: AtomicUsize = ATOMIC_USIZE_INIT;
static MAX_BUFFER_CAPACITY: usize = 4096;
static MAX_NUM_BUFFERS: usize = 32;
