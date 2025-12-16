use riotpool::ThreadPool;
use tidepool::{bind_with_retry, handle_connection, initializing};

fn main() {
    let (local_host, timeout, number_of_threads) = initializing(7878, 8);

    let listener = bind_with_retry(timeout, local_host)
        .expect("failed to bind to the local address. the port is probably busy");
    let pool = ThreadPool::new(number_of_threads);

    for stream in listener.incoming() {
        let stream = stream.expect("failed to read the stream");
        println!("incoming request");

        pool.execute(|| handle_connection(stream));
    }
}
