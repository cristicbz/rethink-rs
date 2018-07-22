use super::connection::Connection;
use super::errors::{Error, ErrorKind, Result};
use super::raw::{RawConnection, Wait};
use failure::{Compat, ResultExt};
use r2d2::{ManageConnection, Pool};
use std::net::{SocketAddr, ToSocketAddrs};
use std::result::Result as StdResult;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

pub type RethinkPool = Pool<ConnectionManager>;

pub struct ConnectionManager {
    endpoints: Vec<SocketAddr>,
    next_endpoint_index: AtomicUsize,
}

impl ConnectionManager {
    fn new(nodes: impl Iterator<Item = impl ToSocketAddrs>) -> Result<Self> {
        let endpoints = nodes
            .map(|address| {
                if let Some(address) = address
                    .to_socket_addrs()
                    .context(ErrorKind::AddressResolution)?
                    .next()
                {
                    return Ok(address);
                }
                Err(ErrorKind::AddressResolution.into())
            })
            .collect::<Result<Vec<SocketAddr>>>()?;
        if endpoints.is_empty() {
            return Err(ErrorKind::NoEndpoints.into());
        }
        Ok(ConnectionManager {
            endpoints,
            next_endpoint_index: AtomicUsize::new(0),
        })
    }
}

impl ManageConnection for ConnectionManager {
    type Connection = Connection;
    type Error = Compat<Error>;

    fn connect(&self) -> StdResult<Connection, Self::Error> {
        let index = self.next_endpoint_index.fetch_add(1, Ordering::SeqCst);
        Ok(Connection::from_raw(
            RawConnection::connect(self.endpoints[index % self.endpoints.len()]).compat()?,
        ))
    }

    fn is_valid(&self, connection: &mut Connection) -> StdResult<(), Self::Error> {
        let mut cursor = connection.run(::query::expr(::query::Null)).compat()?;
        connection
            .next(Wait::For(Duration::from_secs(1)), &mut cursor)
            .compat()?
            .ok_or(ErrorKind::Connection("is_valid timeout".into()).into())
            .compat()
    }

    fn has_broken(&self, connection: &mut Connection) -> bool {
        !connection.is_open()
    }
}
