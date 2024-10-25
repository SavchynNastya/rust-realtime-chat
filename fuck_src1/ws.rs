// // use warp::Filter;
// // use tokio_tungstenite::tungstenite::protocol::Message;
// // use tokio_tungstenite::accept_async;
// // use std::sync::{Arc, Mutex};
// // use futures_util::{stream::SplitSink, SinkExt, StreamExt};
// // use std::collections::HashMap;
// //
// // type Clients = Arc<Mutex<HashMap<u32, SplitSink<tokio_tungstenite::WebSocketStream<warp::ws::WebSocket>, Message>>>>;
// //
// // #[tokio::main]
// // async fn main() {
// //     let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
// //
// //     let clients_clone = clients.clone();
// //     let chat_route = warp::path("chat")
// //         .and(warp::ws())
// //         .map(move |ws: warp::ws::Ws| {
// //             let clients_clone = clients_clone.clone();
// //             ws.on_upgrade(move |socket| handle_connection(socket, clients_clone))
// //         });
// //
// //     warp::serve(chat_route).run(([127, 0, 0, 1], 3030)).await;
// // }
// //
// // async fn handle_connection(ws: warp::ws::WebSocket, clients: Clients) {
// //     let (tx, mut rx) = ws.split();
// //     let id = rand::random::<u32>(); // Unique ID for the client
// //     clients.lock().unwrap().insert(id, tx);
// //
// //     while let Some(msg) = rx.next().await {
// //         match msg {
// //             Ok(Message::Text(text)) => {
// //                 let msg = Message::Text(format!("User {}: {}", id, text));
// //                 for client in clients.lock().unwrap().values_mut() {
// //                     if let Err(_) = client.send(msg.clone()).await {
// //                         // Handle error, e.g. remove client
// //                     }
// //                 }
// //             },
// //             Ok(Message::Close(_)) => {
// //                 // Handle disconnection
// //                 break;
// //             },
// //             _ => {}
// //         }
// //     }
// // }
// use rocket::tokio::sync::RwLock;
// use tokio_tungstenite::tungstenite::Message;
// use std::sync::Arc;
// use std::collections::HashMap;
// use rocket::{Request, State};
//
// #[derive(Default)]
// struct ChatRooms {
//     rooms: HashMap<i32, Vec<tokio_tungstenite::WebSocketStream<tokio_tungstenite::tungstenite::protocol::WebSocket<tokio::net::TcpStream>>>>,
// }
//
// type SharedChatRooms = Arc<RwLock<ChatRooms>>;
//
// // #[get("/ws/<room_id>")]
// // async fn websocket_handler(room_id: i32, request: &Request<'_>, chat_rooms: &State<SharedChatRooms>) {
// //     // Establish a WebSocket connection
// //     let ws_stream = tokio_tungstenite::accept_async(request).await.expect("Error during WebSocket handshake");
// //
// //     // Lock the shared chat rooms
// //     let mut chat_rooms = chat_rooms.write().await;
// //
// //     // Insert the new WebSocket stream into the room
// //     chat_rooms.rooms.entry(room_id).or_insert(vec![]).push(ws_stream);
// //
// //     // Message handling loop
// //     while let Some(msg) = ws_stream.next().await {
// //         match msg {
// //             Ok(Message::Text(text)) => {
// //                 // Broadcast message to all users in the room
// //                 for socket in &chat_rooms.rooms[&room_id] {
// //                     let _ = socket.send(Message::Text(text.clone())).await;
// //                 }
// //             },
// //             Err(e) => eprintln!("Error processing message: {:?}", e),
// //             _ => {},
// //         }
// //     }
// // }
// use rocket::tokio_tungstenite::{accept_async, WebSocketUpgrade, WebSocket};
// use tokio::sync::Mutex;
//
// struct WebSocketState {
//     clients: Arc<Mutex<Vec<WebSocket>>>, // Store connected clients
// }
//
// #[get("/ws")]
// pub async fn ws_handler(state: &rocket::State<WebSocketState>, ws: WebSocketUpgrade) -> WebSocketUpgrade {
//     ws.on_upgrade(|socket| async move {
//         let mut clients = state.clients.lock().unwrap();
//         clients.push(socket.clone()); // Add the new client to the list
//
//         // Handle the WebSocket connection
//         let (mut sender, mut receiver) = socket.split();
//
//         while let Some(message) = receiver.next().await {
//             match message {
//                 Ok(Message::Text(text)) => {
//                     // Broadcast the message to all connected clients
//                     let message_to_send = Message::Text(text.clone());
//                     for client in clients.iter_mut() {
//                         if let Err(_) = client.send(message_to_send.clone()).await {
//                             // Handle send error (client disconnected)
//                         }
//                     }
//                 }
//                 _ => {}
//             }
//         }
//
//         // Remove client when connection is closed
//         clients.retain(|client| client != &socket);
//     })
// }

use rocket::State;
use rocket::tokio::sync::broadcast::{Sender, channel, error::RecvError};
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{Deserialize, Serialize};
use std::sync::Arc;
use rocket_dyn_templates::Template;
use rocket::get;
use rocket::tokio::select;
use crate::models::Message;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct ChatMessage {
    username: String,
    message: String,
    timestamp: String,
}


#[get("/stream")]
pub fn events(queue: &State<Sender<Arc<Message>>>) -> EventStream![] {
    let mut rx = queue.subscribe(); // Subscribe to the channel
    EventStream! {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    yield Event::json(msg.as_ref()); // Yield the received message as JSON
                },
                Err(RecvError::Closed) => break,  // Break the loop if the sender is closed
                Err(RecvError::Lagged(_)) => continue,  // If lagged, continue to receive
            }
        }
    }
}

#[get("/ws/<room_id>")]
pub async fn chat_ws(room_id: u32, queue: &State<Sender<Arc<Message>>>) -> Result<EventStream![], Status> {
    let (tx, mut rx) = channel(100); // Create a new broadcast channel with a capacity of 100 messages

    let mut subscriber = queue.subscribe(); // Subscribe to the main broadcast channel
    tokio::spawn(async move {
        while let Ok(message) = subscriber.recv().await {
            let _ = tx.send(Arc::new(message)); // Forward received messages to the new WebSocket connection
        }
    });

    // Return the EventStream to the client
    Ok(EventStream! {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    yield Event::json(&*msg); // Send the message as JSON
                },
                Err(RecvError::Closed) => break,  // Break the loop if the sender is closed
                Err(RecvError::Lagged(_)) => continue,  // If lagged, continue to receive
            }
        }
    })
}
