use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};
use std::str;

fn handle_client(mut stream: TcpStream){
    let mut buf = [0; 140];
    let _ = stream.write(b"Name: ");
    let _ = stream.read(&mut buf);
    let name = str::from_utf8(&buf).unwrap().trim_end();

    println!("{} has joined.", name);
    let greeting = format!("Hello {}\n", name);
    let _ = stream.write(greeting.as_bytes());

}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:2323").unwrap();
    println!("Starting up...");
    for stream in listener.incoming(){
        match stream {
            Ok(stream) => {
                thread::spawn(move|| {
                    handle_client(stream)
                });
            }
            Err(e) => { /* connection failed */ }
        }
    }
    drop(listener);
}
