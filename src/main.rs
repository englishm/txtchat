use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

enum ClientMessage {
    Session {
        name: String,
        reader: BufReader<TcpStream>,
        writer: BufWriter<TcpStream>,
    },
}
enum ServerMessage {
    Output { message: String },
}

enum RegistrationMessage {
    AddClient {
        name: String,
        tx: Sender<ServerMessage>,
    },
}

fn handle_client(
    mut reader: BufReader<TcpStream>,
    mut writer: BufWriter<TcpStream>,
    tx: Sender<ClientMessage>,
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
    let session = ClientMessage::Session {
        name: name,
        reader: reader,
        writer: writer,
    };

    tx.send(session).unwrap();
}

fn listen(sock: TcpListener, tx: Sender<ClientMessage>) {
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

fn chat_loop(session: ClientMessage, rx: Receiver<ServerMessage>, tx: Sender<ServerMessage>) {
    match session {
        ClientMessage::Session {
            name,
            mut reader,
            mut writer,
        } => {
            println!("Starting chat loop for {}...", name);
            let prompt = format!("{}> ", name);
            let mut input = String::new();
            loop {
                // First, write out any message received from the server
                match rx.try_recv() {
                    Ok(msg) => match msg {
                        ServerMessage::Output { message } => {
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

                if input.len() != 0 {
                    let msg = format!("{} sez '{}'", name, input);
                    tx.send(ServerMessage::Output { message: msg }).unwrap();
                    input.truncate(0);
                }

                // do something with input
            }
        }
    }
}

fn dispatch(rx: Receiver<RegistrationMessage>, rx2: Receiver<ServerMessage>) {
    let mut clients: HashMap<String, Sender<ServerMessage>> = HashMap::new();
    loop {
        // Add/Remove clients
        match rx.try_recv() {
            Ok(msg) => {
                println!("Received registration message");

                match msg {
                    RegistrationMessage::AddClient { name, tx } => {
                        println!("Adding client: {}", name);
                        clients.insert(name, tx);
                    }
                }
            }

            _ => {
                // no msg available, no registration work to do
            }
        }

        match rx2.try_recv() {
            Ok(msg) => {
                println!("Received server message");

                match msg {
                    ServerMessage::Output { message } => {
                        // iterate through current clients and send msg on their chans
                        for (_name, chan) in &clients {
                            chan.send(ServerMessage::Output {
                                message: message.clone(),
                            })
                            .unwrap();
                        }
                    }
                }
            }
            _ => {}
        }
        // TODO: broadcast Output messages
    }
}

fn main() {
    let (tx, rx) = channel();
    let listener = TcpListener::bind("127.0.0.1:2323").unwrap();
    println!("Starting up...");
    thread::spawn(move || listen(listener, tx));

    let (registration_tx, registration_rx) = channel();
    let (servermsg_tx, servermsg_rx) = channel();
    thread::spawn(move || dispatch(registration_rx, servermsg_rx));

    for session in rx {
        match session {
            ClientMessage::Session {
                name,
                reader,
                writer,
            } => {
                let name_copy = name.clone(); // TODO fix lazy mess
                let tx_copy = servermsg_tx.clone();
                println!("main says: {} has joined.", name);
                let (tx2, rx2) = channel();
                registration_tx
                    .send(RegistrationMessage::AddClient {
                        name: name.clone(),
                        tx: tx2,
                    })
                    .unwrap();
                thread::spawn(move || {
                    chat_loop(
                        ClientMessage::Session {
                            name: name.clone(),
                            reader,
                            writer,
                        },
                        rx2,
                        tx_copy,
                    )
                });
                servermsg_tx
                    .send(ServerMessage::Output {
                        message: format!("{} joined!", name_copy),
                    })
                    .unwrap();
            }
        }
    }
}
