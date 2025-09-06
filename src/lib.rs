pub mod http;
pub mod threadpool;

use std::{
    collections::HashMap, fs, io::{self, prelude::*, BufReader}, net::{TcpListener, TcpStream}, sync::mpsc::{self, TryRecvError}, thread, time::Duration
};
use http::{HTTPRequest, HTTPResponse};
use threadpool::ThreadPool;

pub fn run() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let (tx, rx) = mpsc::channel();

    let dispatcher_handle = thread::spawn(move || dispatcher(rx));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        tx.send(stream).expect("Couldn't send connection to dispatcher");
    }

    dispatcher_handle.join().unwrap();
}

fn dispatcher(rx: mpsc::Receiver<TcpStream>) {
    let mut pending_connections = HashMap::new();
    let mut thread_pool = ThreadPool::<HTTPRequest, HTTPResponse>::new(
        8,
        |req| respond(req)
    );

    let mut active_jobs = 0;

    loop {
        if active_jobs > 0 {
            match rx.try_recv() {
                Ok(stream) => {
                    let Some(key) = start_connection(&stream, &mut thread_pool) else { continue };
                    pending_connections.insert(key, stream);
                    active_jobs += 1;
                },
                Err(e) if e == TryRecvError::Empty => (),
                Err(_) => panic!("Socket closed")
            }
        } else {
            let stream = rx.recv().expect("Socket closed");
            let Some(key) = start_connection(&stream, &mut thread_pool) else { continue };
            pending_connections.insert(key, stream);
            active_jobs += 1;
        }
        
        for (key, res) in thread_pool.poll() {
            let mut stream = pending_connections.remove(&key).expect("Invalid key");
            stream.write_all(res.to_string().as_bytes()).unwrap();
            active_jobs -= 1;
        }
    }
}

fn start_connection(
    stream: &TcpStream, 
    thread_pool: &mut ThreadPool<HTTPRequest, HTTPResponse>
) -> Option<u32> {
    let buf_reader = BufReader::new(stream);
    let http_request = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty());

    let Some(http_request) = HTTPRequest::new(http_request) else {
        eprintln!("Malformed request");
        return None;
    };

    Some(thread_pool.submit(http_request))
}

fn respond(req: HTTPRequest) -> HTTPResponse {
    match req.uri().unwrap() {
        "/" => {
            let status = "200 OK";
            let contents = fs::read_to_string("src/hello.html").unwrap();
            HTTPResponse::new(status, Vec::new().iter(), contents).unwrap()
        }
        "/wait" => {
            thread::sleep(Duration::from_secs(10));
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