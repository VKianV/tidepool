//! Tidepool core library
//!
//! Provides utilities for building a simple multithreaded web server:
//! - Connection handling with basic routing
//! - TCP listener binding with retry logic
//! - Initialization helpers
//!
//! This crate is intended to be used alongside the `riotpool` an in house thread pool.

use std::{
    collections::HashMap,
    fs,
    io::{self, BufRead, BufReader, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    path::Path,
    thread,
    time::{Duration, Instant},
};

#[derive(Debug)]
enum HttpMethod {
    Connect,
    Delete,
    Get,
    Head,
    Options,
    Patch,
    Post,
    Put,
    Trace,
}

#[derive(Debug)]
struct RequestLine {
    http_method: HttpMethod,
    request_uri: String,
    http_version: String,
}

impl RequestLine {
    fn new(request_line: String) -> Self {
        let  (mut phrases, http_method, request_uri, http_version);

        phrases = request_line.splitn(3, ' ');

        http_method = match phrases
            .next()
            .expect("failed to read http method")
            .to_lowercase()
            .as_str()
        {
            "get" => HttpMethod::Get,
            "post" => HttpMethod::Post,
            "connect" => HttpMethod::Connect,
            "patch" => HttpMethod::Patch,
            "delete" => HttpMethod::Delete,
            "head" => HttpMethod::Head,
            "put" => HttpMethod::Put,
            "options" => HttpMethod::Options,
            "trace" => HttpMethod::Trace,
            _ => panic!("invalid http method"),
        };

        request_uri = phrases
            .next()
            .expect("failed to read request uri")
            .to_string();

        http_version = phrases
            .next()
            .expect("failed to read http version")
            .to_string();

        Self {
            http_method,
            request_uri,
            http_version,
        }
    }
}

type HttpHeaders = HashMap<String, String>;

#[derive(Debug)]
struct Request {
    request_line: RequestLine,
    headers: HttpHeaders,
    body: String,
}

impl Request {
    fn new(stream: &TcpStream) -> Self {
        let (mut reader, request_line, headers, content_length, body);

        reader = BufReader::new(stream).lines();

        request_line = RequestLine::new(
            reader
                .next()
                .expect("failed to unwrap option")
                .expect("failed to unwrap result"),
        );

        headers = reader
            .by_ref()
            .map(|l| l.expect("read error"))
            .take_while(|l| !l.is_empty())
            .map(|line| {
                let (key, val) = line
                    .split_once(':')
                    .expect("invalid header format: missing a :");
                (key.trim().to_lowercase(), val.trim().to_string())
            })
            .collect::<HttpHeaders>();

        content_length = headers
            .get("content-length")
            .and_then(|val| val.parse::<usize>().ok())
            .unwrap_or(0);

        body = if content_length > 0 {
            reader
                .map(|l| l.unwrap())
                .take_while(|line| !line.is_empty())
                .collect()
        } else {
            String::from("")
        };

        Self {
            request_line,
            headers,
            body,
        }
    }
}

#[derive(Debug)]
struct ResponseLine {
    http_version: String,
    status_code: String,
    reason_phrase: String,
}

impl ResponseLine {
    fn new(line: String) -> Self {
        let mut parts = line.splitn(3, ' ');

        Self {
            http_version: parts.next().unwrap_or("").to_string(),
            status_code: parts.next().unwrap_or("").to_string(),
            reason_phrase: parts.next().unwrap_or("").to_string(),
        }
    }
    fn gen_req_line(&self) -> String {
        format!(
            "{} {} {}",
            self.http_version, self.status_code, self.reason_phrase
        )
    }
}

#[derive(Debug)]
struct Response {
    response_line: ResponseLine,
    headers: HttpHeaders,
    body: String,
}

impl Response {
    fn new(line: String) -> Self {
        Self {
            response_line: ResponseLine::new(line),
            headers: HashMap::new(),
            body: String::from(""),
        }
    }
    fn add_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    fn add_body(&mut self, content: String) {
        self.body = content;
    }

    fn headers_to_string(&self) -> String {
        self.headers
            .iter()
            .map(|(k, v)| format!("{k}: {v}\n"))
            .collect()
    }
    fn send(self, stream: &mut TcpStream) {
        let content = format!(
            "{}\n{}\n{}",
            self.response_line.gen_req_line(),
            self.headers_to_string(),
            self.body
        );
        stream
            .write_all(content.as_bytes())
            .expect("failed to send the response");
    }
}

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
    let (req, uri, mut resp, path);

    req = Request::new(&stream);
    uri = req.request_line.request_uri;

    path = if uri == "/" || uri == "/index" || uri == "/home" {
        "./public/pages/home/home.html".to_string()
    } else if uri == "/sleep" {
        thread::sleep(Duration::from_secs(5));
        "./public/pages/home/home.html".to_string()
    } else {
        "./public/pages/home/".to_string() + uri.trim_start_matches('/')
    };

    match fs::read_to_string(path) {
        Ok(content) => {
            resp = Response::new(String::from("HTTP/1.1 200 OK"));

            resp.add_header("Content-Length".to_string(), content.len().to_string());
            resp.add_header("Connection".to_string(), "close".to_string());

            resp.add_body(content);
        }
        Err(_) => {
            resp = Response::new("HTTP/1.1 404 Not Found".to_string());
            let file_404 = fs::read_to_string("public/pages/status code pages/400.html")
                .expect("couldn't extract the file");
            resp.add_body(file_404);
        }
    }

    resp.send(&mut stream);
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
/// let (addr, timeout, threads) = tidepool::initialize(7878, 8);
/// ```
pub fn initialize(port: u16, number_of_threads: usize) -> (SocketAddrV4, Duration, usize) {
    let local_host = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    let timeout = Duration::from_secs(5);

    println!(
        "tidepool has started listening for connections on {}:{} address",
        local_host.ip(),
        local_host.port()
    );

    (local_host, timeout, number_of_threads)
}
