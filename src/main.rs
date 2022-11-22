use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

enum ClientToServerMessage {
    RegisterClient {
        name: String,
        tx: Sender<ServerToClientMessage>,
    },
    ChatMsg {
        sender: String,
        message: String,
    },
}
enum ServerToClientMessage {
    ChatMsg { sender: String, message: String },
}

fn handle_client(
    mut reader: BufReader<TcpStream>,
    mut writer: BufWriter<TcpStream>,
    cts_tx: Sender<ClientToServerMessage>,
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

    // Create ServerToClient chan
    let (stc_tx, stc_rx) = channel();

    // Register out client
    cts_tx
        .send(ClientToServerMessage::RegisterClient {
            name: name.clone(),
            tx: stc_tx,
        })
        .unwrap();

    // We need to tell main/dispatch how to send us server events (tx)
    // And our name/address
    // But that's all that main/dispatch really needs
    //
    // Then, we want to start our chat/client loop
    // And that needs to have the read/write ends of our socket
    // in addition to an rx for server events and a tx to send things up to main/dispatch

    chat_loop(cts_tx, name, stc_rx, reader, writer);

    // Where this got convoluted the first time is that the tx we have here isn't currently going to dispatch,
    // but just to main, and another channel is created there...
}

fn listen(sock: TcpListener, cts_tx: Sender<ClientToServerMessage>) {
    for stream in sock.incoming() {
        let next_tx = cts_tx.clone();
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

// chat_loop(cts_tx, name, stc_rx, reader, writer);
fn chat_loop(
    cts_tx: Sender<ClientToServerMessage>,
    name: String,
    stc_rx: Receiver<ServerToClientMessage>,
    mut reader: BufReader<TcpStream>,
    mut writer: BufWriter<TcpStream>,
) {
    println!("Starting chat loop for {}...", name);
    let prompt = format!("{}> ", name);
    let mut input = String::new();
    loop {
        // First, write out any message received from the server
        match stc_rx.try_recv() {
            Ok(msg) => match msg {
                ServerToClientMessage::ChatMsg { message, .. } => {
                    println!("Writing out message from server");
                    let output = format!("\n{}\n", message);
                    writer.write(output.as_bytes()).unwrap();
                    writer.flush().unwrap();
                }
            },
            _ => {}
        }

        // Then prompt and receive input
        writer.write(prompt.as_bytes()).unwrap();
        writer.flush().unwrap();
        reader.read_line(&mut input).unwrap();
        let len = input.trim_matches(&['\r', '\n'][..]).len();
        input.truncate(len);

        // send input to server as ChatMsg
        if input.len() != 0 {
            let msg = format!("{} sez '{}'", name.clone(), input);
            cts_tx
                .send(ClientToServerMessage::ChatMsg {
                    sender: name.clone(),
                    message: msg,
                })
                .unwrap();
            input.truncate(0);
        }
    }
}

fn dispatch(cts_rx: Receiver<ClientToServerMessage>) {
    let mut clients: HashMap<String, Sender<ServerToClientMessage>> = HashMap::new();
    loop {
        // Add/Remove clients
        match cts_rx.try_recv() {
            Ok(msg) => {
                println!("Received message from a client");

                match msg {
                    ClientToServerMessage::RegisterClient { name, tx } => {
                        println!("Adding client: {}", name);
                        clients.insert(name, tx);
                    }
                    ClientToServerMessage::ChatMsg { sender, message } => {
                        println!("Msg from {}: '{}'", sender, message);
                        for (_name, stc_tx) in &clients {
                            stc_tx
                                .send(ServerToClientMessage::ChatMsg {
                                    sender: sender.clone(),
                                    message: message.clone(),
                                })
                                .unwrap();
                        }
                    }
                }
            }
            Err(_) => {
                // channel empty, nothing to do
            }
        }
    }
}

fn main() {
    let (cts_tx, cts_rx) = channel();
    let listener = TcpListener::bind("127.0.0.1:2323").unwrap();
    println!("Starting up...");
    thread::spawn(move || listen(listener, cts_tx));
    let dispatch_handle = thread::spawn(move || dispatch(cts_rx));

    dispatch_handle.join().unwrap();
}
