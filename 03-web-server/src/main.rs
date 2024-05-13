use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use web_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878")
        .expect("Connection to port failed! Maybe is already being used?");
    let pool = ThreadPool::build(4).unwrap_or_else(|_| panic!("Error creating the pool"));

    // Stop after receiving 2 requests for demonstration purposes
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    println!("Request: {request_line}");

    let mut words = request_line.split_whitespace();
    let _method = words.next().unwrap();
    let resource = words.next().unwrap();

    let (status_line, filename) = match resource {
        "/" => ("200 OK", "hello.html"),
        "/favicon.ico" => ("200 OK", "rust-logo.svg"),
        "/sleep" => {
            thread::sleep(Duration::from_secs(5));
            ("200 OK", "hello.html")
        }
        _ => ("404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
