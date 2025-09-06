pub mod http;

use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, thread, time::Duration,
};
use http::{HTTPRequest, HTTPResponse};

pub fn run() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let http_request = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty());

    let Some(http_request) = HTTPRequest::new(http_request) else {
        eprintln!("Malformed request");
        return;
    };

    let response = respond(http_request);
    
    stream.write_all(response.to_string().as_bytes()).unwrap();
}

fn respond(req: HTTPRequest) -> HTTPResponse {
    match req.uri().unwrap() {
        "/" => {
            let status = "200 OK";
            let contents = fs::read_to_string("src/hello.html").unwrap();
            HTTPResponse::new(status, Vec::new().iter(), contents).unwrap()
        }
        "/wait" => {
            thread::sleep(Duration::from_secs(1));
            let status = "200 OK";
            let contents = fs::read_to_string("src/hello.html").unwrap();
            HTTPResponse::new(status, Vec::new().iter(), contents).unwrap()
        }
        _ => not_found()
    }
}

fn not_found() -> HTTPResponse {
    let contents = fs::read_to_string("src/404.html").unwrap();
    HTTPResponse::new("404 NOT FOUND", Vec::new().iter(), contents).unwrap()
}