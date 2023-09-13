use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
};

use tokio::{
    net::TcpListener,
    sync::{broadcast, RwLock},
};

mod structs;
mod handler;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chat: Arc<RwLock<HashMap<SocketAddr, structs::UserData>>> = Arc::new(RwLock::new(HashMap::new()));

    let (tx, _) = broadcast::channel::<String>(10);

    let port = String::from("3000");

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    println!("[ SRV ] Server listening on {}", listener.local_addr().unwrap());

    loop {
        let (socket, addr) = listener.accept().await?;

        let thr_chat = chat.clone();

        let tx = tx.clone();

        tokio::spawn(async move {
            handler::handler(socket, addr, thr_chat, tx).await;
        });
    }
}
