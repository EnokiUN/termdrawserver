use std::{collections::HashMap, sync::Arc};

use futures::stream::SplitSink;
use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use uuid::Uuid;

// TODO: make a Room struct
pub type Rooms = Arc<Mutex<HashMap<Uuid, Room>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub x: u32,
    pub y: u32,
    pub colour: PixelColour,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: Uuid,
    #[cfg(feature = "logic")]
    #[serde(serialize_with = "to_pixel_vec")]
    pub pixels: HashMap<(u32, u32), PixelColour>,
    #[cfg(not(feature = "logic"))]
    pub pixels: Vec<Pixel>,
    #[cfg(feature = "logic")]
    #[serde(skip)]
    pub users: HashMap<Uuid, SplitSink<WebSocketStream<TcpStream>, Message>>,
}

pub fn to_pixel_vec<S>(
    value: &HashMap<(u32, u32), PixelColour>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = serializer.serialize_seq(Some(value.len()))?;
    for pixel in value.iter().map(|((x, y), colour)| Pixel {
        x: *x,
        y: *y,
        colour: colour.clone(),
    }) {
        seq.serialize_element(&pixel)?;
    }
    seq.end()
}

#[cfg(feature = "logic")]
#[derive(Debug, Serialize)]
#[serde(tag = "op", content = "d")]
pub enum ServerPayload<'a> {
    Draw { user_id: Uuid, pixel: Pixel },
    Reset(Uuid),
    Join { user_id: Uuid, room: &'a Room },
    NewRoom { room_id: Uuid, user_id: Uuid },
    RoomNotFound,
}

#[cfg(not(feature = "logic"))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", content = "d")]
pub enum ServerPayload {
    Draw { user_id: Uuid, pixel: Pixel },
    Reset(Uuid),
    Join { user_id: Uuid, room: Room },
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
