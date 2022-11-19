use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str;
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut name = String::new();
    let _ = stream.write(b"Name: ");
    let mut input = BufReader::new(stream.try_clone().unwrap());
    let mut output = BufWriter::new(stream);
    let _ = input.read_line(&mut name);
    let len = name.trim_matches(&['\r', '\n'][..]).len();
    name.truncate(len);

    println!("{} has joined.", name);
    let greeting = format!("Hello {}\n", name);
    let _ = output.write(greeting.as_bytes());
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
