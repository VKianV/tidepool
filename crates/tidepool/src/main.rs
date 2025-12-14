use riotpool::ThreadPool;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};
use tidepool::{bind_with_retry, handle_connection};

fn main() {
    let local_host = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7878);
    let timeout = Duration::from_secs(5);

    println!(
        "tidepool has started listening for connections on {}:{} address",
        local_host.ip(),
        local_host.port()
    );

    let listener = bind_with_retry(timeout, local_host)
        .expect("failed to bind to the local address. the port is probably busy");
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.expect("failed to read the stream");
        println!("incoming request");

        pool.execute(|| handle_connection(stream));
    }
}
