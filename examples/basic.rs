#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rethink;

use rethink::{Connection, RawConnection, Error, Wait, query as r};

fn run() -> Result<(), Error> {
    let mut connection = Connection::from_raw(RawConnection::connect("172.17.0.1:28015")?);
    let mut cursor = connection.run(
        r::db("default")
            .table("comment_cursors")
            .between(r::MinVal, "B")
            .in_index("a")
            .right_bound("open"),
    )?;
    let name: Vec<String> = connection.next(Wait::Yes, &mut cursor)?.unwrap();
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
