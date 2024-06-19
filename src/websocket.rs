use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use base64::prelude::*;
use sha1::{Digest, Sha1};

use crate::{
    client_id::{ClientID, ClientIDFactory},
    message::Message,
};

pub trait Events {
    fn on_connect(&mut self, client_id: ClientID);
    fn on_message(&mut self, client_id: ClientID, message: Message);
    fn on_disconnect(&mut self, client_id: ClientID);
}

pub struct Websocket {
    listener: TcpListener,
    client_id_factory: ClientIDFactory,
    pub clients: Arc<Mutex<HashMap<ClientID, Sender<Message>>>>,
}

impl Websocket {
    pub fn new<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        Ok(Self {
            listener,
            client_id_factory: Default::default(),
            clients: Default::default(),
        })
    }

    pub fn run<T>(&mut self, events: Arc<Mutex<T>>) -> io::Result<()>
    where T: Events + Send + 'static {
        for stream in self.listener.incoming() {
            let stream = stream?;
            let client_id = self.client_id_factory.create_id();
            let (tx, rx) = channel::<Message>();
            self.clients.lock().unwrap().insert(client_id, tx);
            let events_clone = events.clone();
            let events_clone2 = events.clone();
            let transmitters_clone = self.clients.clone();
            thread::spawn(move || {
                if let Err(err) = Self::receiver(client_id, stream, rx, events_clone) {
                    eprintln!("receiver error: {}", err);
                }
                transmitters_clone.lock().unwrap().remove(&client_id);
                events_clone2.lock().unwrap().on_disconnect(client_id);
            });
        }
        Ok(())
    }

    fn receiver<T>(
        client_id: ClientID,
        mut stream: TcpStream,
        rx: Receiver<Message>,
        events: Arc<Mutex<T>>,
    ) -> io::Result<()>
    where
        T: Events + Send + 'static,
    {
        let mut accept = None;
        loop {
            let line = read_line(&mut stream)?;
            if line.is_empty() {
                break;
            }
            if line.starts_with(b"Sec-WebSocket-Key: ") {
                let key = line.strip_prefix(b"Sec-WebSocket-Key: ").unwrap();
                let mut hasher = Sha1::new();
                hasher.update(key);
                hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
                accept = Some(hasher.finalize());
            }
        }
        let accept = accept.ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "missing Sec-WebSocket-Key")
        });
        stream.write_all(b"HTTP/1.1 101 Switching Protocols\r\n")?;
        stream.write_all(b"Upgrade: websocket\r\n")?;
        stream.write_all(b"Connection: Upgrade\r\n")?;
        write!(
            stream,
            "Sec-WebSocket-Accept: {}\r\n",
            BASE64_STANDARD.encode(accept?)
        )?;
        stream.write_all(b"\r\n")?;
        stream.flush()?;
        let stream_clone = stream.try_clone()?;
        thread::spawn(move || {
            if let Err(err) = Self::transmitter(stream_clone, rx) {
                eprintln!("transmitter error: {}", err);
            }
        });
        events.lock().unwrap().on_connect(client_id);
        while let Some(message) = Message::read(&mut stream)? {
            if message.data.is_empty() {
                break;
            }
            events.lock().unwrap().on_message(client_id, message);
        }
        Ok(())
    }

    fn transmitter(mut stream: TcpStream, rx: Receiver<Message>) -> io::Result<()> {
        for message in rx {
            message.write(&mut stream)?;
        }
        Ok(())
    }
}

fn read_line(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    loop {
        let mut byte = [0];
        stream.read_exact(&mut byte)?;
        buf.push(byte[0]);
        if buf.ends_with(b"\r\n") {
            break;
        }
    }
    buf.truncate(buf.len() - 2);
    Ok(buf)
}
