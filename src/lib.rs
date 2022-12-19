use std::{collections::HashMap, sync::Arc};

use futures::stream::SplitSink;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use uuid::Uuid;

// TODO: make a Room struct
pub type Clients = Arc<Mutex<HashMap<Uuid, Vec<SplitSink<WebSocketStream<TcpStream>, Message>>>>>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PixelColour {
    Clear,
    White,
    DarkRed,
    Red,
    DarkBlue,
    Blue,
    DarkGreen,
    Green,
    DarkYellow,
    Yellow,
    DarkMagenta,
    Magenta,
    DarkGrey,
    Grey,
    Black,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pixel {
    x: u32,
    y: u32,
    colour: PixelColour,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
pub enum ServerPayload {
    Draw(Pixel),
    Reset,
    Join { user_id: Uuid, pixels: Vec<Pixel> },
    NewRoom { room_id: Uuid, user_id: Uuid },
    RoomNotFound,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
pub enum ClientPayload {
    CreateRoom,
    JoinRoom(Uuid),
    Draw(Pixel),
    Reset,
}
