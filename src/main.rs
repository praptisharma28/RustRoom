use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use warp::Filter;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub room: Option<String>,
    pub is_streaming: bool,
}

#[derive(Debug, Clone)]
pub struct Room {
    pub name: String,
    pub users: HashMap<String, User>,
    pub streamer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    JoinRoom { room_name: String },
    LeaveRoom,
    StartStream,
    StopStream,
    ChatMessage { content: String },
    WebRTCSignal { target_user: String, signal: serde_json::Value },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    RoomJoined { room_name: String, users: Vec<User>, streamer: Option<String> },
    UserJoined { user: User },
    UserLeft { user_id: String },
    StreamStarted { user_id: String },
    StreamStopped { user_id: String },
    ChatMessage { user_id: String, content: String },
    WebRTCSignal { from_user: String, signal: serde_json::Value },
    Error { message: String },
}

type Rooms = Arc<Mutex<HashMap<String, Room>>>;
type Connections = Arc<Mutex<HashMap<String, mpsc::UnboundedSender<ServerMessage>>>>;

#[tokio::main]
async fn main() {
    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));
    let connections: Connections = Arc::new(Mutex::new(HashMap::new()));

    // Serve static files
    let static_files = warp::fs::dir("static");

    // WebSocket route
    let rooms_filter = warp::any().map(move || rooms.clone());
    let connections_filter = warp::any().map(move || connections.clone());
    
    let websocket = warp::path("ws")
        .and(warp::ws())
        .and(rooms_filter)
        .and(connections_filter)
        .map(|ws: warp::ws::Ws, rooms, connections| {
            ws.on_upgrade(move |socket| handle_connection(socket, rooms, connections))
        });

    let routes = static_files.or(websocket);

    println!("🚀 RustRoom server running on http://localhost:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_connection(
    ws_stream: warp::ws::WebSocket,
    rooms: Rooms,
    connections: Connections,
) {
    let user_id = Uuid::new_v4().to_string();
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Add connection
    connections.lock().await.insert(user_id.clone(), tx);

    // Handle outgoing messages
    let user_id_clone = user_id.clone();
    let connections_clone = connections.clone();
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            let json = serde_json::to_string(&message).unwrap();
            if ws_sender.send(Message::text(json)).await.is_err() {
                break;
            }
        }
        connections_clone.lock().await.remove(&user_id_clone);
    });

    // Handle incoming messages
    let mut current_user = User {
        id: user_id.clone(),
        room: None,
        is_streaming: false,
    };

    while let Some(result) = ws_receiver.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(_) => break,
        };

        if let Ok(text) = msg.to_text() {
            if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(text) {
                handle_client_message(
                    client_msg,
                    &mut current_user,
                    &rooms,
                    &connections,
                ).await;
            }
        }
    }

    // Cleanup on disconnect
    cleanup_user(&current_user, &rooms, &connections).await;
}

async fn handle_client_message(
    message: ClientMessage,
    user: &mut User,
    rooms: &Rooms,
    connections: &Connections,
) {
    match message {
        ClientMessage::JoinRoom { room_name } => {
            // Leave current room if any
            if user.room.is_some() {
                leave_room(user, rooms, connections).await;
            }

            // Join new room
            let mut rooms_guard = rooms.lock().await;
            let room = rooms_guard.entry(room_name.clone()).or_insert_with(|| Room {
                name: room_name.clone(),
                users: HashMap::new(),
                streamer: None,
            });

            user.room = Some(room_name.clone());
            room.users.insert(user.id.clone(), user.clone());

            // Notify user
            let users: Vec<User> = room.users.values().cloned().collect();
            let streamer = room.streamer.clone();
            
            if let Some(tx) = connections.lock().await.get(&user.id) {
                let _ = tx.send(ServerMessage::RoomJoined { 
                    room_name: room_name.clone(), 
                    users, 
                    streamer 
                });
            }

            // Notify others in room
            broadcast_to_room(&room_name, &user.id, ServerMessage::UserJoined { user: user.clone() }, rooms, connections).await;
        },
        
        ClientMessage::StartStream => {
            if let Some(room_name) = &user.room {
                let mut rooms_guard = rooms.lock().await;
                if let Some(room) = rooms_guard.get_mut(room_name) {
                    // Stop current streamer if any
                    if let Some(current_streamer) = &room.streamer {
                        if let Some(streamer_user) = room.users.get_mut(current_streamer) {
                            streamer_user.is_streaming = false;
                        }
                    }
                    
                    // Set new streamer
                    room.streamer = Some(user.id.clone());
                    user.is_streaming = true;
                    room.users.insert(user.id.clone(), user.clone());

                    // Notify all users in room
                    broadcast_to_room(room_name, "", ServerMessage::StreamStarted { user_id: user.id.clone() }, rooms, connections).await;
                }
            }
        },

        ClientMessage::ChatMessage { content } => {
            if let Some(room_name) = &user.room {
                broadcast_to_room(room_name, "", ServerMessage::ChatMessage { 
                    user_id: user.id.clone(), 
                    content 
                }, rooms, connections).await;
            }
        },

        ClientMessage::WebRTCSignal { target_user, signal } => {
            if let Some(tx) = connections.lock().await.get(&target_user) {
                let _ = tx.send(ServerMessage::WebRTCSignal { 
                    from_user: user.id.clone(), 
                    signal 
                });
            }
        },

        _ => {} // Handle other messages
    }
}

async fn leave_room(user: &mut User, rooms: &Rooms, connections: &Connections) {
    if let Some(room_name) = &user.room {
        let mut rooms_guard = rooms.lock().await;
        if let Some(room) = rooms_guard.get_mut(room_name) {
            room.users.remove(&user.id);
            
            if room.streamer.as_ref() == Some(&user.id) {
                room.streamer = None;
                broadcast_to_room(room_name, "", ServerMessage::StreamStopped { user_id: user.id.clone() }, rooms, connections).await;
            }

            broadcast_to_room(room_name, &user.id, ServerMessage::UserLeft { user_id: user.id.clone() }, rooms, connections).await;

            if room.users.is_empty() {
                rooms_guard.remove(room_name);
            }
        }
        user.room = None;
        user.is_streaming = false;
    }
}

async fn cleanup_user(user: &User, rooms: &Rooms, connections: &Connections) {
    let mut user_copy = user.clone();
    leave_room(&mut user_copy, rooms, connections).await;
    connections.lock().await.remove(&user.id);
}

async fn broadcast_to_room(
    room_name: &str,
    except_user: &str,
    message: ServerMessage,
    rooms: &Rooms,
    connections: &Connections,
) {
    let rooms_guard = rooms.lock().await;
    if let Some(room) = rooms_guard.get(room_name) {
        let connections_guard = connections.lock().await;
        
        for user_id in room.users.keys() {
            if user_id != except_user {
                if let Some(tx) = connections_guard.get(user_id) {
                    let _ = tx.send(message.clone());
                }
            }
        }
    }
}
