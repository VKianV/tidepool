use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    // ToDo: try a couple more times to connect and don't panic for a few times.
    let listener = TcpListener::bind("127.0.0.1:7878")
        .expect("failed to bind to the local address. the port is probably busy");

    for stream in listener.incoming() {
        let stream = stream.expect("failed to read the stream");

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.expect("Failed to read http request lines"))
        .take_while(|line| !line.is_empty())
        .collect();

    let status_line = "HTTP/1.1 200 OK";

    let body = fs::read_to_string("htmls/hello.html").expect("failed to read the html file.");
    let body_length = body.len();

    let header = format!("Content-Length: {body_length}");
    
    let response = format!("{status_line}\n{header}\n\n{body}");

    stream
        .write_all(response.as_bytes())
        .expect("failed to write to stream");
}
