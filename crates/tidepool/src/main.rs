use riotpool::ThreadPool;
use std::{net::TcpListener, thread};
use tidepool::handle_connection;

fn main() {
    println!("tidepool has started listening for connections on 127.0.0.1:7878 address");
    // ToDo: try a couple more times to connect and don't panic for a few times.
    let listener = TcpListener::bind("127.0.0.1:7878")
        .expect("failed to bind to the local address. the port is probably busy");
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.expect("failed to read the stream");
        println!("incoming request");

        pool.execute(|| handle_connection(stream));
    }
}
