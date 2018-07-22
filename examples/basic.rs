extern crate env_logger;
extern crate rethink;

use rethink::{query as r, Connection, Error, RawConnection, Wait};
use std::time::Duration;

fn main() -> Result<(), Error> {
    env_logger::init();
    let mut connection = Connection::from_raw(RawConnection::connect("172.17.0.1:28015")?);
    let mut cursor = connection.run(
        r::db("default")
            .table("comment_cursors")
            .g("n")
            .map(|x| x.as_string().add("foo")),
    )?;
    let name: Vec<String> = connection
        .next(Wait::For(Duration::from_secs(1)), &mut cursor)?
        .unwrap();
    println!("{:?}", name);
    Ok(())
}
