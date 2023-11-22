// irc.libera.chat/6667

use std::{
    error::Error,
    io::{self, BufRead, BufReader, Write},
    net::TcpStream,
    sync::mpsc::{self, Sender},
    thread,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel::<Message>();
    let tx2 = tx.clone();

    let mut stream = TcpStream::connect("irc.libera.chat:6667")?;
    let stream_copy = stream.try_clone()?;

    thread::spawn(|| input_loop(tx).unwrap());
    thread::spawn(move || socket_loop(tx2, &stream_copy).unwrap());

    write!(stream, "NICK cfel|testclient\r\n")?;
    write!(stream, "USER cfeltestclient cfel cfel cfel|test client\r\n")?;

    loop {
        let msg = rx.recv()?;
        match msg {
            Message::Quit => {
                stream.shutdown(std::net::Shutdown::Both)?;
                break;
            }
            Message::Raw(msg) => {
                println!("{}", msg);
            }
        }
    }

    Ok(())
}

enum Message {
    Quit,
    Raw(String),
}

fn input_loop(tx: Sender<Message>) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(io::stdin());
    let mut buf = String::new();

    loop {
        let read = reader.read_line(&mut buf).unwrap();
        if read == 0 || buf == "/quit\n" {
            tx.send(Message::Quit)?;
            break;
        }

        buf.clear();
    }

    Ok(())
}

fn socket_loop(tx: Sender<Message>, stream: &TcpStream) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(stream);

    let mut buf = String::new();

    loop {
        reader.read_line(&mut buf)?;
        tx.send(Message::Raw(buf.clone()))?;
    }
}
