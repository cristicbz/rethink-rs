extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rethink;

use rethink::{query as r, Connection, Error, RawConnection, Wait};

fn run() -> Result<(), Error> {
    let mut connection = Connection::from_raw(RawConnection::connect("172.17.0.1:28015")?);
    let mut cursor = connection.run(r::db_list())?;
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
