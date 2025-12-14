use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    net::{SocketAddrV4, TcpListener, TcpStream},
    thread,
    time::{Duration, Instant},
};

pub fn handle_connection(mut stream: TcpStream) {
    let request_line = BufReader::new(&stream)
        .lines()
        .next()
        .expect("failed to get the next item")
        .expect("failed to read from stream");

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "htmls/hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "htmls/hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "htmls/404.html"),
    };

    let body = fs::read_to_string(filename).expect("failed to read the html file.");
    let body_length = body.len();

    let header = format!("Content-Length: {body_length}");

    let response = format!("{status_line}\n{header}\n\n{body}");

    stream
        .write_all(response.as_bytes())
        .expect("failed to write to stream");
}

pub fn bind_with_retry(
    timeout: Duration,
    local_host: SocketAddrV4,
) -> Result<TcpListener, io::Error> {
    let start = Instant::now();
    loop {
        match TcpListener::bind(format!("{}:{}", local_host.ip(), local_host.port())) {
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
