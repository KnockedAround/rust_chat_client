use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect.");
    client.set_nonblocking(true).expect("Failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buffer = vec![0; MSG_SIZE];

        match client.read_exact(&mut buffer) {
            Ok(_) => {
                let msg = buffer.into_iter().take_while(|&chr| chr != 0).collect::<Vec<_>>();
                println!("messaged received {:?}", msg);
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server severed");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buffer = msg.clone().into_bytes();
                buffer.resize(MSG_SIZE, 0);
                client.write_all(&buffer).expect("Writing to socket failed");
                println!("message sent {:?}", msg);
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a message:");
    loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).expect("Reading from stdin failed");
        let msg = buffer.trim().to_string();

        if msg == ":quit" || tx.send(msg).is_err() { break }
    }

    println!("Bye!");

}