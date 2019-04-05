use super::connection::{Connection, Cursor};
use super::errors::{Error, ErrorKind, Result};
use super::query::{self as r, Query};
use super::raw::{RawConnection, Wait};
use failure::{Compat, ResultExt};
use r2d2::{CustomizeConnection, ManageConnection, Pool, PooledConnection};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::net::{SocketAddr, ToSocketAddrs};
use std::ops::{Deref, DerefMut};
use std::result::Result as StdResult;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use std::vec::IntoIter as VecIntoIter;

#[derive(Debug, Clone)]
pub struct RethinkPool(Pool<ConnectionManager>);

#[derive(Debug, Clone)]
pub struct PoolOptions {
    pub max_size: u32,
    pub min_idle: Option<u32>,
}

impl Default for PoolOptions {
    fn default() -> Self {
        PoolOptions {
            max_size: 32,
            min_idle: Some(8),
        }
    }
}

impl RethinkPool {
    pub fn new(nodes: impl IntoIterator<Item = impl ToSocketAddrs>) -> Result<RethinkPool> {
        Self::with_options(nodes, PoolOptions::default())
    }

    pub fn with_options(
        nodes: impl IntoIterator<Item = impl ToSocketAddrs>,
        options: PoolOptions,
    ) -> Result<RethinkPool> {
        Ok(RethinkPool(
            Pool::builder()
                .connection_customizer(Box::new(ConnectionCustomizer))
                .max_size(options.max_size)
                .min_idle(options.min_idle)
                .build(ConnectionManager::new(nodes)?)
                .context(ErrorKind::Connection(
                    "failed to create connection pool".into(),
                ))?,
        ))
    }

    pub fn run<OutT, ItemT: DeserializeOwned>(
        &self,
        wait: Wait,
        query: Query<OutT, impl Serialize>,
    ) -> Result<ItemT> {
        let mut connection = self.get()?;
        connection.run(wait, query)
    }

    pub fn run_iter<OutT, ItemT: DeserializeOwned>(
        &self,
        wait: Wait,
        query: Query<OutT, impl Serialize>,
    ) -> Result<PoolCursor<ItemT>> {
        let mut connection = self.get()?;
        Ok(PoolCursor {
            cursor: connection.run_cursor(query)?,
            buffered: Vec::new().into_iter(),
            connection,
            wait,
            _phantom: PhantomData,
        })
    }

    pub fn get(&self) -> Result<PooledRethinkConnection> {
        Ok(PooledRethinkConnection(self.0.get().context(
            ErrorKind::Connection("failed to get pooled connection".into()),
        )?))
    }
}

#[derive(Debug)]
pub struct PoolCursor<ItemT: DeserializeOwned> {
    connection: PooledRethinkConnection,
    cursor: Cursor,
    wait: Wait,
    buffered: VecIntoIter<ItemT>,
    _phantom: PhantomData<*const ItemT>,
}

impl<ItemT: DeserializeOwned> Iterator for PoolCursor<ItemT> {
    type Item = Result<ItemT>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(buffered) = self.buffered.next() {
            Some(Ok(buffered))
        } else if self.cursor.exhausted() {
            None
        } else {
            let buffer: Vec<ItemT> = match self.connection.next_batch(self.wait, &mut self.cursor) {
                Err(error) => return Some(Err(error)),
                Ok(None) => {
                    if self.cursor.exhausted() {
                        return None;
                    } else {
                        return Some(Err(ErrorKind::IteratorTimeout.into()));
                    }
                }
                Ok(Some(buffer)) => buffer,
            };
            self.buffered = buffer.into_iter();
            return self.next();
        }
    }
}

#[derive(Debug)]
pub struct PooledRethinkConnection(PooledConnection<ConnectionManager>);

impl Deref for PooledRethinkConnection {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        self.0.deref()
    }
}

impl DerefMut for PooledRethinkConnection {
    fn deref_mut(&mut self) -> &mut Connection {
        self.0.deref_mut()
    }
}

#[derive(Debug)]
pub struct ConnectionManager {
    endpoints: Vec<SocketAddr>,
    next_endpoint_index: AtomicUsize,
}

impl ConnectionManager {
    pub fn new(nodes: impl IntoIterator<Item = impl ToSocketAddrs>) -> Result<Self> {
        let endpoints = nodes
            .into_iter()
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
        connection
            .run(Wait::For(Duration::from_secs(1)), r::expr(r::Null))
            .compat()
    }

    fn has_broken(&self, connection: &mut Connection) -> bool {
        !connection.is_open()
    }
}

#[derive(Debug)]
pub struct ConnectionCustomizer;
impl CustomizeConnection<Connection, Compat<Error>> for ConnectionCustomizer {
    fn on_acquire(&self, connection: &mut Connection) -> StdResult<(), Compat<Error>> {
        connection.invalidate();
        Ok(())
    }
}
