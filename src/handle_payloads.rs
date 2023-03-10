use futures::{stream::SplitStream, SinkExt, StreamExt};
use termdrawserver::{ClientPayload, PixelColour, Rooms, ServerPayload};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use uuid::Uuid;

pub async fn handle_payloads(
    rooms: &Rooms,
    rx: &mut SplitStream<WebSocketStream<TcpStream>>,
    room_id: &Uuid,
    user_id: &Uuid,
) {
    while let Some(Ok(Message::Text(msg))) = rx.next().await {
        if let Ok(payload) = serde_json::from_str::<ClientPayload>(&msg) {
            match payload {
                ClientPayload::Draw(pixel) => {
                    let mut rooms = rooms.lock().await;
                    let room = rooms
                        .get_mut(room_id)
                        .expect("Could not obtain the client's room");
                    if let PixelColour::Clear = pixel.colour {
                        room.pixels.remove(&(pixel.x, pixel.y));
                    } else {
                        room.pixels.insert((pixel.x, pixel.y), pixel.colour.clone());
                    }
                    let payload = serde_json::to_string(&ServerPayload::Draw {
                        user_id: *user_id,
                        pixel,
                    })
                    .unwrap();
                    for (id, tx) in room.users.iter_mut() {
                        if let Err(err) = tx.send(Message::Text(payload.clone())).await {
                            log::error!("Could not send event to {}: {:?}", id, err);
                        }
                    }
                }
                ClientPayload::Reset => {
                    let mut rooms = rooms.lock().await;
                    let room = rooms
                        .get_mut(room_id)
                        .expect("Could not obtain the client's room");
                    room.pixels.clear();
                    let payload = serde_json::to_string(&ServerPayload::Reset(*user_id)).unwrap();
                    for (id, tx) in room.users.iter_mut() {
                        if let Err(err) = tx.send(Message::Text(payload.clone())).await {
                            log::error!("Could not send event to {}: {:?}", id, err);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
