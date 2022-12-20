use anyhow::Context;
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

use termdrawserver::*;

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
    let (mut tx, mut rx) = stream.split();
    let (_room_id, _user_id) = loop {
        if let Some(Ok(Message::Text(msg))) = rx.next().await {
            if let Ok(payload) = serde_json::from_str::<ClientPayload>(&msg) {
                log::info!("Got payload {:?}", payload);
                match payload {
                    ClientPayload::CreateRoom => {
                        let room_id = Uuid::new_v4();
                        let user_id = Uuid::new_v4();
                        tx.send(Message::Text(
                            serde_json::to_string(&ServerPayload::NewRoom { room_id, user_id })
                                .unwrap(),
                        ))
                        .await
                        .expect("Could not send room create payload");
                        let mut pixels = HashMap::new();
                        pixels.insert((0, 0), PixelColour::White);
                        clients.lock().await.insert(
                            room_id,
                            Room {
                                id: room_id,
                                pixels,
                                users: vec![tx],
                            },
                        );
                        log::info!(
                            "New room {} created by user {} with id {}",
                            room_id,
                            addr,
                            room_id
                        );
                        break (room_id, user_id);
                    }
                    ClientPayload::JoinRoom(room_id) => {
                        let mut clients = clients.lock().await;
                        if let Some(room) = clients.get_mut(&room_id) {
                            let user_id = Uuid::new_v4();
                            tx.send(Message::Text(
                                serde_json::to_string(&ServerPayload::Join { user_id, room })
                                    .unwrap(),
                            ))
                            .await
                            .expect("Could not send room create payload");
                            room.users.push(tx);
                            log::info!("User {} joined room {} with id {}", addr, room_id, user_id);
                            break (room_id, user_id);
                        } else {
                            tx.send(Message::Text(
                                serde_json::to_string(&ServerPayload::RoomNotFound).unwrap(),
                            ))
                            .await
                            .expect("Could not send room create payload");
                            log::info!("Client {} tried to access unknown room", addr);
                        }
                    }
                    _ => {}
                }
            }
        }
    };
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
