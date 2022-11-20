use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Sender};
use std::thread;

enum Message {
    Session {
        name: String,
        reader: BufReader<TcpStream>,
        writer: BufWriter<TcpStream>,
    },
}

fn handle_client(
    mut reader: BufReader<TcpStream>,
    mut writer: BufWriter<TcpStream>,
    tx: Sender<Message>,
) {
    // Get user's name (later: do some kind of auth)
    let mut name = String::new();
    writer.write(b"Name: ").unwrap();
    writer.flush().unwrap();
    reader.read_line(&mut name).unwrap();
    let len = name.trim_matches(&['\r', '\n'][..]).len();
    name.truncate(len);

    // Greet user
    println!("Thread says: {} has joined.", name);
    let greeting = format!("Hello {}\n", name);
    writer.write(greeting.as_bytes()).unwrap();
    writer.flush().unwrap();

    // Initialize session
    let session = Message::Session {
        name: name,
        reader: reader,
        writer: writer,
    };

    tx.send(session).unwrap();
}

fn listen(sock: TcpListener, tx: Sender<Message>) {
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

    for session in rx {
        match session {
            Message::Session {
                name,
                reader: _,
                writer: _,
            } => {
                println!("main says: {} has joined.", name);
            }
            _ => {
                unimplemented!("handle non-session messages")
            }
        }
    }
}
