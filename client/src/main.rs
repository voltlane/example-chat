use shared::Message;
use voltlane_client::{net, Connection};
use std::io::Write;
use tokio::io::AsyncReadExt as _;

async fn run(client: &mut Connection) -> anyhow::Result<()> {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = [0; 1024];
        tokio::select! {
            packet = net::recv_size_prefixed(&mut client.read) => {
                let msg: Message = serde_json::from_slice(&packet?)?;
                match msg {
                    Message::ChatMessage(msg) => println!("{msg}"),
                    _ => unreachable!(),
                }
            }
            n = stdin.read(&mut buf) => {
                let n = n?;
                if n == 0 {
                    println!("EOF");
                    std::process::exit(0);
                }
                let msg = String::from_utf8_lossy(&buf[..n]);
                let msg = msg.trim();
                let msg = Message::ChatMessage(msg.to_string());
                let serialized_msg = serde_json::to_vec(&msg)?;
                net::send_size_prefixed(&mut client.write, &serialized_msg).await?;
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = Connection::new("127.0.0.1:42000").await?;

    loop {
        print!("Enter your name: ");
        std::io::stdout().flush()?;
        let mut name = String::new();
        std::io::stdin().read_line(&mut name)?;
        let name = name.trim();
        if name.is_empty() {
            println!("Name cannot be empty");
            continue;
        }
        let login_packet = Message::LoginResponse(name.to_string());
        let serialized_msg = serde_json::to_vec(&login_packet)?;
        if let Err(err) = net::send_size_prefixed(&mut client.write, &serialized_msg).await {
            println!("[ERROR] {err}");
            continue;
        }
        if let Err(err) = run(&mut client).await {
            println!("[ERROR] {err}")
        }
        if let Ok(()) = client.reconnect().await {
            println!("Reconnected successfully");
        } else {
            println!("Reconnect attempt failed!");
            break;
        }
    }
    eprintln!("Exiting...");
    Ok(())
}
