use futures::{stream::SplitStream, SinkExt, StreamExt};
use termdrawserver::{ClientPayload, Clients, ServerPayload};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use uuid::Uuid;

pub async fn handle_payloads(
    clients: &Clients,
    rx: &mut SplitStream<WebSocketStream<TcpStream>>,
    room_id: &Uuid,
) {
    while let Some(Ok(Message::Text(msg))) = rx.next().await {
        if let Ok(payload) = serde_json::from_str::<ClientPayload>(&msg) {
            match payload {
                ClientPayload::Draw(pixel) => {
                    let mut clients = clients.lock().await;
                    let room = clients
                        .get_mut(room_id)
                        .expect("Could not obtain the client's room");
                    let payload = ServerPayload::Draw(pixel);
                    for (id, tx) in room.users.iter_mut() {
                        if let Err(err) = tx
                            .send(Message::Text(serde_json::to_string(&payload).unwrap()))
                            .await
                        {
                            log::error!("Could not send event to {}: {:?}", id, err);
                        }
                    }
                }
                ClientPayload::Reset => {
                    let mut clients = clients.lock().await;
                    let room = clients
                        .get_mut(room_id)
                        .expect("Could not obtain the client's room");
                    for (id, tx) in room.users.iter_mut() {
                        if let Err(err) = tx
                            .send(Message::Text(
                                serde_json::to_string(&ServerPayload::Reset).unwrap(),
                            ))
                            .await
                        {
                            log::error!("Could not send event to {}: {:?}", id, err);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
