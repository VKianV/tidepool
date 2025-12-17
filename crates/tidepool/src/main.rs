//! Tidepool: A simple multi-threaded web server in Rust.
//!
//! This binary crate starts a TCP server that listens for incoming connections
//! and serves static files using a thread pool for handling requests concurrently.
//!
//! Features:
//! - Basic GET request routing (`/`, `/sleep`, and /assets)
//! - 404 handling for unknown paths
//! - Graceful binding retry on startup
//! - Thread pool powered by the `riotpool` crate
//!
//! Example usage:
//! ```bash
//! cargo run
//! ```
//! Then visit `http://127.0.0.1:7878/` in your browser.

use riotpool::ThreadPool;
use tidepool::{bind_with_retry, handle_connection, initializing};

fn main() {
    // Initialize configuration: ip localhost(127.0.0.1), port 7878, 8 worker threads
    let (local_host, timeout, number_of_threads) = initializing(7878, 8);

    // Bind to the address with retry logic in case the port is temporarily busy
    let listener = bind_with_retry(timeout, local_host)
        .expect("failed to bind to the local address. the port is probably busy");

    // Create a thread pool to handle incoming connections concurrently
    let pool = ThreadPool::new(number_of_threads);

    // Accept incoming connections and dispatch them to the pool
    for stream in listener.incoming() {
        let stream = stream.expect("failed to read the stream");
        println!("incoming request");

        pool.execute(|| handle_connection(stream));
    }

    // This line is reached only if the listener stops (e.g., on error)
    println!("Shutting down.");
}