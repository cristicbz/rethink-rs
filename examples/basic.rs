extern crate env_logger;
extern crate failure;
#[macro_use]
extern crate log;
extern crate rethink;
extern crate serde_json;

use failure::Error;
use rethink::{query as r, RethinkPool, Wait};
use serde_json::Value;

use std::time::Duration;

fn run() -> Result<(), Error> {
    let pool = RethinkPool::new(&["localhost:28015"])?;
    let name = pool
        .run_iter(
            Wait::For(Duration::from_secs(1)),
            r::db("default")
                .table("users")
                .get_all("2cbc68d8b6b514e0")
                .in_index("id")
                .limit(1),
        )?
        .collect::<Result<Vec<Value>, _>>()?;
    println!("{:?}", name);
    Ok(())
}

fn main() {
    env_logger::init();
    if let Err(error) = run() {
        error!("Error: {}", error);
        let mut cause = error.cause();
        loop {
            error!("caused by: {}", cause);
            if let Some(new_cause) = cause.cause() {
                cause = new_cause;
            } else {
                break;
            }
        }
        error!("Backtrace:\n{}", error.backtrace());
    }
}
