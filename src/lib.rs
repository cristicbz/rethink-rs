#![forbid(unsafe_code)]
#![deny(warnings)]

#[macro_use]
extern crate failure;

extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate arrayvec;
extern crate byteorder;

#[macro_use]
extern crate log;

extern crate r2d2;

pub mod connection;
pub mod pool;
pub mod query;
pub mod raw;

mod enums;
mod errors;

pub use connection::Connection;
pub use failure::Error;
pub use pool::{PooledRethinkConnection, RethinkPool};
pub use raw::{RawConnection, Wait};
