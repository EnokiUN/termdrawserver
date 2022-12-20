mod handle_join;

use anyhow::Context;
use futures::StreamExt;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;

use termdrawserver::Clients;

// New Room
//
// Client                            Server
// RoomCreate      ----------->
//                 <-----------       NewRoom(UUid, Uuid)
//
// Join Room
// Client                            Server
// RoomJoin(Uuid)  ----------->
//                 <-----------      Join(Uuid, Room)
//
// Join non-existent room
// Client                            Server
// RoomJoin(Uuid)  ----------->
//                 <-----------      RoomNotFound
async fn handle_connection(stream: TcpStream, clients: Clients) {
    let addr = stream.peer_addr().expect("Could not obtain peer address");
    let stream = accept_async(stream)
        .await
        .expect("Could not accpet socket stream");
    log::info!("Started a connection with {}", addr);
    let (tx, mut rx) = stream.split();
    let (_room_id, _user_id) = handle_join::handle_join(&clients, &mut rx, tx, &addr)
        .await
        .unwrap();
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

    let clients = Arc::new(Mutex::new(HashMap::new()));

    while let Ok((stream, _)) = socket.accept().await {
        let clients = Arc::clone(&clients);
        tokio::spawn(handle_connection(stream, clients));
    }

    Ok(())
}
