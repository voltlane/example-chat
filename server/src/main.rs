use std::fmt::Display;
use shared::Message;

use tokio::net::{
    TcpListener, TcpStream,
    tcp::{OwnedReadHalf, OwnedWriteHalf},
};

pub async fn send_message(
    write: &mut OwnedWriteHalf,
    to_client: u64,
    message: &Message,
) -> anyhow::Result<()> {
    let serialized_msg = serde_json::to_vec(&message)?;
    let packet = net::TaggedPacket::Data {
        client_id: to_client,
        data: serialized_msg,
    };
    net::send_tagged_packet(write, packet).await
}

pub struct Client {
    client_id: u64,
    name: String,
}

pub struct ChatServer {
    read: net::FramedReader<OwnedReadHalf>,
    write: OwnedWriteHalf,
    clients: Vec<Client>,
}

impl ChatServer {
    pub fn new(stream: TcpStream) -> Self {
        let (read, write) = stream.into_split();
        let read = net::new_framed_reader(read);
        Self {
            read,
            write,
            clients: Vec::new(),
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            let packet = net::recv_tagged_packet(&mut self.read).await?;
            match packet {
                net::TaggedPacket::Data { client_id, data } => {
                    self.handle_data(client_id, data).await?
                }
                net::TaggedPacket::Failure { client_id, error } => {
                    println!("Client {client_id} failed: {error}");
                    self.clients.retain(|c| c.client_id != client_id);
                },
                net::TaggedPacket::Kick { client_id: _ } => unreachable!(),
                net::TaggedPacket::Reconnection { client_id } => {
                    println!("Client {} has reconnected", client_id)
                }
            }
        }
    }

    pub async fn handle_data(&mut self, client_id: u64, data: Vec<u8>) -> anyhow::Result<()> {
        let msg: Message = serde_json::from_slice(&data)?;

        match msg {
            Message::ChatMessage(content) => {
                let client = self.get_client(client_id);
                if client.is_none() {
                    println!("Kicking client {client_id}, because they have not logged in");
                    send_message(
                        &mut self.write,
                        client_id,
                        &make_system_message("You must first log in"),
                    )
                    .await?;
                    net::send_tagged_packet(&mut self.write, net::TaggedPacket::Kick { client_id })
                        .await?;
                    return Ok(());
                }
                let client = client.unwrap();

                let msg = Message::ChatMessage(format!("<{}> {}", client.name, content));
                let data = serde_json::to_vec(&msg)?;
                for client in &self.clients {
                    net::send_tagged_packet(
                        &mut self.write,
                        net::TaggedPacket::Data {
                            client_id: client.client_id,
                            data: data.clone(),
                        },
                    )
                    .await?;
                }
            }
            Message::LoginResponse(name) => {
                if self.clients.iter().any(|c| c.name == name) {
                    send_message(
                        &mut self.write,
                        client_id,
                        &make_system_message(
                            "Name already taken. Try again with a different name.",
                        ),
                    )
                    .await?;
                    net::send_tagged_packet(&mut self.write, net::TaggedPacket::Kick { client_id })
                        .await?;
                } else {
                    self.clients.push(Client { client_id, name });
                }
            }
        }

        Ok(())
    }

    fn get_client(&self, client_id: u64) -> Option<&Client> {
        self.clients
            .iter()
            .filter(|c| c.client_id == client_id)
            .next()
    }
}

fn make_system_message(message: impl Display) -> Message {
    Message::ChatMessage(format!("[SYSTEM] {message}"))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let connserver_listener = TcpListener::bind("127.0.0.1:42001").await?;

    loop {
        let (mut connserver_socket, _) = connserver_listener.accept().await?;
        net::configure_performance_tcp_socket(&mut connserver_socket)?;

        let chat_server = ChatServer::new(connserver_socket);

        if let Err(err) = chat_server.run().await {
            println!("Error: {}", err);
        }
    }
}
