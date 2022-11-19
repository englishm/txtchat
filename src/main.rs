use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::mpsc::{channel, Sender};

fn handle_client(mut stream: TcpStream, tx: Sender<String>) {
    let mut name = String::new();
    let _ = stream.write(b"Name: ");
    let mut input = BufReader::new(stream.try_clone().unwrap());
    let mut output = BufWriter::new(stream);
    let _ = input.read_line(&mut name);
    let len = name.trim_matches(&['\r', '\n'][..]).len();
    name.truncate(len);

    println!("Thread says: {} has joined.", name);
    let greeting = format!("Hello {}\n", name);
    tx.send(name).unwrap();
    let _ = output.write(greeting.as_bytes());
}

fn listen(sock: TcpListener, tx: Sender<String>){
    for stream in sock.incoming() {
        let next_tx = tx.clone();
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream, next_tx));
            }
            Err(e) => { /* connection failed */ }
        }
    }
    drop(sock);
}

fn main() {
    let (tx, rx) = channel();
    let listener = TcpListener::bind("127.0.0.1:2323").unwrap();
    println!("Starting up...");
    thread::spawn(move || listen(listener, tx));

    for name in rx {
        println!("main says: {} has joined.", name);
    }
}
