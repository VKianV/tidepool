//! Tidepool core library
//!
//! Provides utilities for building a simple multi-threaded web server:
//! - Connection handling with basic routing
//! - TCP listener binding with retry logic
//! - Initialization helpers
//!
//! This crate is intended to be used alongside the `riotpool` an in house thread pool.

use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    thread,
    time::{Duration, Instant},
};

/// Handles a single TCP connection by reading the request line,
/// determining the appropriate response, and writing it back to the client.
///
/// Currently supports:
/// - `GET / HTTP/1.1` → serves `assets/hello.html`
/// - `GET /sleep HTTP/1.1` → sleeps for 5 seconds then serves `assets/hello.html`
/// - All other requests → serves files that match the files that exist in `assets/` if not it returns a 404 error
///
/// # Panics
///
/// This function panics if:
/// - It fails to read the request line from the stream
/// - It fails to read the requested HTML file from disk
/// - It fails to write the response to the stream
///
/// # Examples
///
/// ```no_run
/// use std::net::TcpStream;
/// tidepool::handle_connection(stream);
/// ```
pub fn handle_connection(mut stream: TcpStream) {
    let request_line = BufReader::new(&stream)
        .lines()
        .next()
        .expect("failed to get the next item")
        .expect("failed to read from stream");

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "assets/hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "assets/hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "assets/404.html"),
    };

    let body = fs::read_to_string(filename).expect("failed to read the html file.");
    let body_length = body.len();
    let header = format!("Content-Length: {body_length}");
    let response = format!("{status_line}\r\n{header}\r\n\r\n{body}");

    stream
        .write_all(response.as_bytes())
        .expect("failed to write to stream");
}

/// Attempts to bind a `TcpListener` to the given address, retrying every 300ms
/// until the timeout expires.
///
/// Useful when the port might be temporarily held by a previous process.
///
/// # Returns
///
/// - `Ok(TcpListener)` on successful bind
/// - `Err(io::Error)` if the timeout is reached
///
/// # Examples
///
/// ```no_run
/// use std::time::Duration;
/// use std::net::SocketAddrV4;
/// use std::net::Ipv4Addr;
///
/// let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7878);
/// let listener = tidepool::bind_with_retry(Duration::from_secs(5), addr).unwrap();
/// ```
pub fn bind_with_retry(
    timeout: Duration,
    local_host: SocketAddrV4,
) -> Result<TcpListener, io::Error> {
    let start = Instant::now();
    loop {
        match TcpListener::bind(local_host) {
            Ok(listener) => return Ok(listener),
            Err(e) => {
                if start.elapsed() >= timeout {
                    return Err(e);
                }
                thread::sleep(Duration::from_millis(300));
            }
        }
    }
}

/// Initializes the server configuration and prints a startup message.
///
/// Returns a tuple of:
/// - The local socket address to bind to
/// - The bind retry timeout duration
/// - The number of worker threads to use
///
/// # Examples
///
/// ```no_run
/// let (addr, timeout, threads) = tidepool::initializing(7878, 8);
/// ```
pub fn initializing(port: u16, number_of_threads: usize) -> (SocketAddrV4, Duration, usize) {
    let local_host = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    let timeout = Duration::from_secs(5);

    println!(
        "tidepool has started listening for connections on {}:{} address",
        local_host.ip(),
        local_host.port()
    );

    (local_host, timeout, number_of_threads)
}