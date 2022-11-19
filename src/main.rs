use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 140];
    let _ = stream.write(b"Name: ");
    let _ = stream.read(&mut buf);
    let mut name = str::from_utf8(&buf).unwrap().to_owned();
    println!("initial length: {}", name.len());
    println!("Initial value: {:?}", name);
    let len = name.trim_matches(&['\r', '\n'][..]).len();
    name.truncate(len - 3);
    println!("new length: {}", name.len());
    println!("New value: {:?}", name);

    println!("{} has joined.", name);
    let greeting = format!("Hello {}\n", name);
    let _ = stream.write(greeting.as_bytes());
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:2323").unwrap();
    println!("Starting up...");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
            Err(e) => { /* connection failed */ }
        }
    }
    drop(listener);
}
