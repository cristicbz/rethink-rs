extern crate rethink;

use rethink::RawConnection;

fn main() {
    let connection = RawConnection::new("localhost:28015");
}
