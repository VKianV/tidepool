//! Tidepool core library
//!
//! Provides utilities for building a simple multithreaded web server:
//! - Connection handling with basic routing
//! - TCP listener binding with retry logic
//! - Initialization helpers
//!
//! This crate is intended to be used alongside the `riotpool` an in house thread pool.

use std::{
    path::Path,
    fs,
    io::{self, BufRead, BufReader, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    thread,
    time::{Duration, Instant},
};

/// Handles a single TCP connection by reading the request line,
/// determining the appropriate response, and writing it back to the client.
///
/// Currently, supports:
/// - `GET / HTTP/1.1` → serves `public/index.html`
/// - `GET /sleep HTTP/1.1` → sleeps for 5 seconds then serves `public/index.html`
/// - All other requests → serves files that match the files that exist in `public/` if not it returns a 404 error
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

    let full_path;
    let (status_line, filename) = if request_line.starts_with("GET ") && request_line.ends_with(" HTTP/1.1") {
        let path = request_line[4..request_line.len() - 9].trim(); // extract /path

        if path == "/" || path.is_empty() {
            ("HTTP/1.1 200 OK", "public/index.html")
        } else if path == "/sleep" {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "public/index.html")
        } else {
            // Serve any other file from the "public" directory
            let sanitized_path = if path.starts_with('/') { &path[1..] } else { path };

            // Basic security: prevent directory traversal
            if sanitized_path.contains("..") || sanitized_path.contains('\\') {
                ("HTTP/1.1 400 BAD REQUEST", "public/400.html")
            } else {
                 full_path = format!("public/{}", sanitized_path);

                // If file exists → serve it, else 404
                if Path::new(&full_path).exists() {
                    ("HTTP/1.1 200 OK", full_path.as_str())
                } else {
                    ("HTTP/1.1 404 NOT FOUND", "public/404.html")
                }
            }
        }
    } else {
        ("HTTP/1.1 400 BAD REQUEST", "public/400.html")
    };

    // Rest remains the same...
    let body = fs::read_to_string(filename).expect("failed to read the file");
    let body_length = body.len();
    let response = format!(
        "{status_line}\r\nContent-Length: {body_length}\r\n\r\n{body}"
    );

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