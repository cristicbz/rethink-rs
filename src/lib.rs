#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate failure;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate byteorder;
extern crate bufstream;

use failure::Error;
use std::time::Duration;
use byteorder::{WriteBytesExt, ReadBytesExt, LittleEndian};
use std::io::{Read, Write, BufRead};
use std::net::{TcpStream, ToSocketAddrs};
use bufstream::BufStream;

pub struct RawConnection {
    tcp: BufStream<TcpStream>,
    bytes: Vec<u8>,
}

const CONNECTION_TIMEOUT_MS: u64 = 1000;
const MESSAGE_TIMEOUT_MS: u64 = 1000;
const RETHINK_MAGIC: u32 = 0x34c2bdc3;

impl RawConnection {
    pub fn new<A: ToSocketAddrs>(address: A) -> Result<Self, Error> {
        let tcp = TcpStream::connect_timeout(
            &address.to_socket_addrs()?.next().unwrap(),
            Duration::from_millis(CONNECTION_TIMEOUT_MS),
        )?;
        tcp.set_read_timeout(
            Some(Duration::from_millis(MESSAGE_TIMEOUT_MS)),
        )?;
        tcp.set_write_timeout(
            Some(Duration::from_millis(MESSAGE_TIMEOUT_MS)),
        )?;
        tcp.set_nodelay(true)?;

        let mut tcp = BufStream::new(tcp);
        let mut bytes = Vec::with_capacity(32768);
        tcp.write_u32::<LittleEndian>(RETHINK_MAGIC)?;
        tcp.read_until(b'\0', &mut bytes)?;
        bytes.clear();
        let handshake = serde_json::from_slice::<HandshakeResponse>(&bytes)?;

        if !handshake.success {
            return Err(format_err!("Handhake failed: {:?}", handshake));
        }
        debug!("Handshake response: {:?}", handshake);

        Ok(RawConnection { tcp, bytes })
    }
}

#[derive(Deserialize, Debug)]
struct HandshakeResponse {
    success: bool,
    min_protocol_version: u32,
    max_protocol_version: u32,
    server_version: String,
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
