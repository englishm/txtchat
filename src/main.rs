use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Sender};
use std::thread;

fn handle_client(
    mut reader: BufReader<TcpStream>,
    mut writer: BufWriter<TcpStream>,
    tx: Sender<String>,
) {
    let mut name = String::new();
    let _ = writer.write(b"Name: ");
    writer.flush();
    let _ = reader.read_line(&mut name);
    let len = name.trim_matches(&['\r', '\n'][..]).len();
    name.truncate(len);

    println!("Thread says: {} has joined.", name);
    let greeting = format!("Hello {}\n", name);
    tx.send(name).unwrap();
    let _ = writer.write(greeting.as_bytes());
}

fn listen(sock: TcpListener, tx: Sender<String>) {
    for stream in sock.incoming() {
        let next_tx = tx.clone();
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    println!("in the thread, but not yet the fn?");
                    let reader = BufReader::new(stream.try_clone().unwrap());
                    let writer = BufWriter::new(stream);
                    handle_client(reader, writer, next_tx)
                });
            }
            Err(_e) => { /* connection failed */ }
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
