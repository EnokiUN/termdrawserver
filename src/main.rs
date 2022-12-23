mod handle_join;
mod handle_payloads;

use anyhow::Context;
use futures::StreamExt;
use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;

use termdrawserver::Rooms;

async fn handle_connection(stream: TcpStream, rooms: Rooms) {
    let addr = stream.peer_addr().expect("Could not obtain peer address");
    let stream = accept_async(stream)
        .await
        .expect("Could not accpet socket stream");
    log::info!("Started a connection with {}", addr);
    let (tx, mut rx) = stream.split();
    let (room_id, user_id) = match handle_join::handle_join(&rooms, &mut rx, tx, &addr).await {
        Ok(ids) => ids,
        Err(_) => return,
    };
    handle_payloads::handle_payloads(&rooms, &mut rx, &room_id).await;
    let tx = {
        let mut rooms = rooms.lock().await;
        let room = rooms.get_mut(&room_id);
        if let Some(room) = room {
            let user = room.users.remove(&user_id);
            if room.users.is_empty() {
                rooms.remove(&room_id);
            }
            user
        } else {
            None
        }
    };
    log::info!("Closed connection with {} at {}", user_id, addr);
    if let Some(tx) = tx {
        if let Ok(mut stream) = rx.reunite(tx) {
            stream
                .close(Some(CloseFrame {
                    code: CloseCode::Normal,
                    reason: Cow::Borrowed("Connection closed"),
                }))
                .await
                .ok();
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let server_address = format!(
        "{}:{}",
        env::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string()),
        env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string())
    );
    let socket = TcpListener::bind(&server_address)
        .await
        .with_context(|| format!("Could not bind a TCP socket to {}", server_address))?;
    log::info!("Started listenting on: {}", server_address);

    let rooms = Arc::new(Mutex::new(HashMap::new()));

    while let Ok((stream, _)) = socket.accept().await {
        let rooms = Arc::clone(&rooms);
        tokio::spawn(handle_connection(stream, rooms));
    }

    Ok(())
}
