extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rethink;

use rethink::{query as r, Connection, Error, RawConnection, Wait};
use std::time::Duration;

fn run() -> Result<(), Error> {
    let mut connection = Connection::from_raw(RawConnection::connect("172.17.0.1:28015")?);
    let mut cursor = connection.run(
        r::db("default")
            .table("comment_cursors")
            .g("n")
            .items_as::<r::StringOut>()
            .map(|x| x.add("foo"))
    )?;
    let name: Vec<String> = connection.next(Wait::For(Duration::from_secs(1)), &mut cursor)?.unwrap();
    println!("{:?}", name);
    Ok(())
}

fn main() {
    env_logger::init().unwrap();
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
