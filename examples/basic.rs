#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rethink;

use rethink::{RawConnection, Error, Wait, query as r};

fn run() -> Result<(), Error> {
    let mut connection = RawConnection::connect("172.17.0.1:28015")?;
    let token = connection.start_request(
        r::db("default")
            .table("comment_cursors")
            .get("summary-updater")
            .assert_not_null()
            .i("e"),
    )?;
    let mut result = Vec::new();
    let recv_token = connection.recv(Wait::Yes, &mut result)?;
    assert_eq!(Some(token), recv_token);
    println!("{}", String::from_utf8_lossy(&result));
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
