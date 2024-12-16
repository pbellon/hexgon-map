use bytes::BytesMut;

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use actix::{Actor, Addr, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{
    web::{Data, Payload},
    Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws::{self, Message as WsMessage, ProtocolError, WebsocketContext};

use crate::{coords::AxialCoords, grid::TileData};

/// Type alias for the list of WebSocket clients
pub type ClientList = Arc<Mutex<HashSet<Addr<MyWebSocket>>>>;

/// Function to initialize a new empty client list
pub fn init_clients() -> ClientList {
    Arc::new(Mutex::new(HashSet::new()))
}

/// WebSocket actor to handle messages and manage connections
pub struct MyWebSocket {
    clients: ClientList, // Shared client list for broadcasting messages
}

impl MyWebSocket {
    // Constructor to create a new instance of MyWebSocket
    pub fn new(clients: ClientList) -> Self {
        MyWebSocket { clients }
    }
}

// Implement the Actor trait for MyWebSocket
impl Actor for MyWebSocket {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        // Insert the client address into the shared client list
        self.clients.lock().unwrap().insert(addr);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        // Remove the client address from the shared client list when the connection stops
        self.clients.lock().unwrap().remove(&addr);
    }
}

// Define a message type for sending binary data to WebSocket clients
#[derive(Message)]
#[rtype(result = "()")]
pub struct MyBinaryMessage(pub Vec<u8>); // A custom message type containing the binary data

// Implement the Handler trait for MyBinaryMessage
impl Handler<MyBinaryMessage> for MyWebSocket {
    type Result = ();

    fn handle(&mut self, msg: MyBinaryMessage, ctx: &mut Self::Context) {
        // Handle sending the binary message to the WebSocket client
        ctx.binary(msg.0); // Send the binary message to the client
    }
}

// Handle incoming WebSocket messages (e.g., text messages)
impl StreamHandler<Result<WsMessage, ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<WsMessage, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(WsMessage::Text(text)) => {
                // Echo the received text message back to the client
                ctx.text(format!("Echo: {}", text));
            }
            Ok(WsMessage::Binary(bin)) => {
                // Handle binary messages if needed
                ctx.binary(bin); // Echo binary message (or process as needed)
            }
            Err(e) => {
                // Handle errors (if necessary)
                eprintln!("Error processing WebSocket message: {:?}", e);
            }
            _ => {}
        }
    }
}

// Function to construct a binary message from AxialCoords and TileData (Message type: 0x01 for tile change)
pub fn tile_change_message(coords: &AxialCoords, tile: &TileData) -> Vec<u8> {
    let user_id_bytes = match &tile.user_id {
        Some(id) => id.as_bytes(),
        None => &[],
    };

    // Create a buffer with enough space to hold all the data
    let mut buffer = BytesMut::with_capacity(9 + user_id_bytes.len()); // 4 (q) + 4 (r) + 1 (strength) + user_id_length
    buffer.extend_from_slice(&[0x01]); // Message type for tile change (0x01)
    buffer.extend_from_slice(&coords.q.to_le_bytes()); // Serialize q (i32)
    buffer.extend_from_slice(&coords.r.to_le_bytes()); // Serialize r (i32)
    buffer.extend_from_slice(&[tile.strength]); // Serialize strength (u8)
    buffer.extend_from_slice(&(user_id_bytes.len() as u8).to_le_bytes()); // Serialize length of user_id
    buffer.extend_from_slice(user_id_bytes); // Add user_id bytes

    buffer.to_vec()
}

// Function to construct a binary message for a new user (Message type: 0x02 for new user)
pub fn new_user_message(user_id: &str, user_name: &str, user_color: &str) -> Vec<u8> {
    let user_id_bytes = user_id.as_bytes();
    let user_name_bytes = user_name.as_bytes();
    let user_color_bytes = user_color.as_bytes();

    let user_id_length = user_id_bytes.len();
    let user_name_length = user_name_bytes.len();
    let user_color_length = user_color_bytes.len();

    let mut buffer = BytesMut::with_capacity(
        1 + 4 + 1 + user_id_length + 1 + user_name_length + 1 + user_color_length + 4,
    );

    // Message type for new user
    buffer.extend_from_slice(&[0x02]); // Message type for new user (0x02)

    // Serialize user ID
    buffer.extend_from_slice(&(user_id_length as u8).to_le_bytes()); // Length of user ID
    buffer.extend_from_slice(user_id_bytes); // User ID bytes

    // Serialize user Name
    buffer.extend_from_slice(&(user_name_length as u8).to_le_bytes()); // Length of user Name
    buffer.extend_from_slice(user_name_bytes); // User Name bytes

    // Serialize user Color
    buffer.extend_from_slice(&(user_color_length as u8).to_le_bytes()); // Length of user Color
    buffer.extend_from_slice(user_color_bytes); // User Color bytes

    buffer.to_vec()
}

pub fn notify_new_user(clients: &ClientList, user_id: &str, user_name: &str, user_color: &str) {
    let login_msg = new_user_message(user_id, user_name, user_color);

    // Send the login message to all connected clients
    for client in clients.lock().unwrap().iter() {
        client.do_send(MyBinaryMessage(login_msg.clone()));
    }
}

pub fn score_change_message(user_id: &str, score: u32) -> Vec<u8> {
    let user_id_bytes = user_id.as_bytes();
    let user_id_length = user_id_bytes.len();

    let mut buffer = BytesMut::with_capacity(1 + 1 + user_id_length + 4); // Type + user ID length + ID + score
    buffer.extend_from_slice(&[0x03]); // Message type for score change
    buffer.extend_from_slice(&(user_id_length as u8).to_le_bytes()); // Length of user ID
    buffer.extend_from_slice(user_id_bytes); // User ID
    buffer.extend_from_slice(&score.to_le_bytes()); // New score (u32)

    buffer.to_vec()
}

pub fn notify_score_change(clients: &ClientList, user_id: &str, score: u32) {
    let score_change_msg = score_change_message(user_id, score);

    for client in clients.lock().unwrap().iter() {
        client.do_send(MyBinaryMessage(score_change_msg.clone()));
    }
}

// WebSocket handler to initialize and manage WebSocket connections
pub async fn ws_handler(
    req: HttpRequest,
    stream: Payload,
    clients: Data<ClientList>, // Shared client list
) -> Result<HttpResponse, Error> {
    // Start the WebSocket actor with the provided client list
    ws::start(MyWebSocket::new(clients.get_ref().clone()), &req, stream)
}
