#[macro_use]
extern crate failure;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate byteorder;
extern crate arrayvec;
#[macro_use]
extern crate log;

pub mod query;
pub mod raw;
pub mod connection;
mod enums;
mod concatenator;

pub use raw::{RawConnection, Wait};
pub use connection::Connection;
pub use failure::Error;
