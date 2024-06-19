mod client;
mod client_id;
mod frame;
mod message;
mod websocket;

use std::{
    collections::HashMap,
    io,
    sync::{mpsc::Sender, Arc, Mutex},
};

use client_id::ClientID;
use message::Message;
use serde::{Deserialize, Serialize};
use websocket::{Events, Websocket};

#[derive(Serialize, Deserialize)]
enum ClientMessage {
    Join { username: String },
    Typing,
    Message { message: String },
}

#[derive(Serialize, Deserialize)]
enum ServerMessage {
    Message { username: String, message: String },
    Typing { username: String },
}

struct App {
    clients: Arc<Mutex<HashMap<ClientID, Sender<Message>>>>,
}

impl Events for App {
    fn on_connect(&mut self, client_id: ClientID) {
        println!("Client {:?} connected", client_id);
    }

    fn on_message(&mut self, client_id: ClientID, message: Message) {
        let message = std::str::from_utf8(&message.data).unwrap();
        println!("Received message from client {:?}: {}", client_id, message);
        let chat_message: ChatMessage = serde_json::from_str(message).unwrap();
        let clients = self.clients.lock().unwrap();
        for (id, sender) in clients.iter() {
            if *id == client_id {
                continue;
            }
            let message = Message::new(
                serde_json::to_string(&chat_message).unwrap().into_bytes(),
            );
            sender.send(message).unwrap();
        }
    }

    fn on_disconnect(&mut self, client_id: ClientID) {
        println!("Client {:?} disconnected", client_id);
    }
}

fn main() -> io::Result<()> {
    let mut ws = Websocket::new("127.0.0.1:8000")?;
    let app = Arc::new(Mutex::new(App { clients: ws.clients.clone() }));
    ws.run(app)
}
