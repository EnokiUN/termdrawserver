use std::{collections::HashMap, net::SocketAddr};

use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use termdrawserver::{ClientPayload, Clients, Room, ServerPayload};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use uuid::Uuid;

pub async fn handle_join(
    clients: &Clients,
    rx: &mut SplitStream<WebSocketStream<TcpStream>>,
    mut tx: SplitSink<WebSocketStream<TcpStream>, Message>,
    addr: &SocketAddr,
) -> Result<(Uuid, Uuid), ()> {
    while let Some(Ok(Message::Text(msg))) = rx.next().await {
        if let Ok(payload) = serde_json::from_str::<ClientPayload>(&msg) {
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
                    let mut users = HashMap::new();
                    users.insert(user_id, tx);
                    clients.lock().await.insert(
                        room_id,
                        Room {
                            id: room_id,
                            pixels: HashMap::new(),
                            users,
                        },
                    );
                    log::info!(
                        "New room {} created by user {} with id {}",
                        room_id,
                        addr,
                        room_id
                    );
                    return Ok((room_id, user_id));
                }
                ClientPayload::JoinRoom(room_id) => {
                    let mut clients = clients.lock().await;
                    if let Some(room) = clients.get_mut(&room_id) {
                        let user_id = Uuid::new_v4();
                        tx.send(Message::Text(
                            serde_json::to_string(&ServerPayload::Join { user_id, room }).unwrap(),
                        ))
                        .await
                        .expect("Could not send room create payload");
                        room.users.insert(user_id, tx);
                        log::info!("User {} joined room {} with id {}", addr, room_id, user_id);
                        return Ok((room_id, user_id));
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
    Err(())
}
