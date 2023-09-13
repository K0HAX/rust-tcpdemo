use std::{collections::HashMap, net::SocketAddr, println, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{broadcast::Sender, RwLock},
};

use crate::structs::{Request, UserData, Response};

fn strip_trailing_newline(input: &str) -> &str {
    input
        .strip_suffix("\r\n")
        .or(input.strip_suffix("\n"))
        .unwrap_or(input)
}

pub async fn handler(
    mut socket: TcpStream,
    addr: SocketAddr,
    chat_rwlock: Arc<RwLock<HashMap<SocketAddr, UserData>>>,
    tx: Sender<String>,
    ) {
    let mut rx = tx.subscribe();

    println!("[ ACK ] {} joined", addr);
    tx.send(format!("[ ACK ] {} joined\r\n", addr)).unwrap();

    // FIXME This sucks, can't read more than 2048 bytes
    let mut buf = vec![0; 2048];

    {
        let mut add_user = chat_rwlock.write().await;
        add_user.insert(
            addr,
            UserData {
                addr,
            });
    }

    loop {
        tokio::select! {
            // Buffer to broadcast
            result = socket.read(&mut buf) => {
                // Connection handler
                let buf_len = match result {
                    Err(e) => {
                        eprintln!("[ ERR ] Error reading data: {}", e);
                        return;
                    }
                    Ok(n) if n == 0 => {
                        break;
                    }
                    Ok(n) => n,
                };

                buf.resize(buf_len, 0);

                // Slurp input data
                let data: Request = Request {
                    data: String::from_utf8_lossy(&buf).to_string(),
                };
                println!("[ RCV ] {} | {}", addr, strip_trailing_newline(&data.data));
                {
                    let res = Response {
                        msg: data.data.to_string(),
                    };

                    tx.send(format!("[{}] {}", addr, res.msg)).unwrap();
                }
            }

            // Broadcast to buffer
            msg = rx.recv() => {
                if let Ok(data) = msg {
                    socket.write(data.as_bytes()).await.unwrap();
                }
            }
        };

        // Clear buffer
        buf = vec![0; 2048];
    }
    {
        let mut add_user = chat_rwlock.write().await;
        add_user.remove(&addr);
        println!("[ FIN ] {} left", addr);
    }
}
