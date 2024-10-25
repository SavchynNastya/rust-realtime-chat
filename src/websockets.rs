// use rocket::State;
// use rocket::tokio::sync::broadcast::Sender;
// use rocket::tokio::sync::broadcast::{channel, error::RecvError};
// use rocket::response::stream::{Event, EventStream};
// use rocket::form::FromForm;
// use rocket::serde::{Deserialize, Serialize};
// use rocket::tokio::select;
// use rocket::Shutdown;
// use rocket::Request;
// use rocket::response::Redirect;
// use rocket::form::Form;
// use std::sync::Arc;
// use rocket_dyn_templates::Template;
// // use rocket_dyn_templates::tera::Template;
//
// #[derive(Debug, Clone, Deserialize, Serialize, FromForm)]
// pub(crate) struct ChatMessage {
//     username: String,
//     message: String,
//     timestamp: String,
// }
//
// #[get("/chat")]
// pub fn chat() -> Template {
//     Template::render("chat", &())
// }
//
// #[get("/stream")]
// pub fn events(queue: &State<Sender<Arc<ChatMessage>>>) -> EventStream![] {
//     let mut rx = queue.subscribe();
//     EventStream! {
//         loop {
//             match rx.recv().await {
//                 Ok(msg) => yield Event::json(&msg),
//                 Err(RecvError::Closed) => break,
//                 Err(RecvError::Lagged(_)) => continue,
//             }
//         }
//     }
// }
//
// #[post("/chat", data = "<msg_form>")]
// pub async fn post_message(msg_form: Form<ChatMessage>, queue: &State<Sender<Arc<ChatMessage>>>) -> Redirect {
//     let message = Arc::new(msg_form.into_inner());
//     let _res = queue.send(message.clone());
//     Redirect::to(uri!(chat))
// }


use rocket::State;
use rocket::tokio::sync::broadcast::Sender;
use rocket::tokio::sync::broadcast::{channel, error::RecvError};
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::select;
use rocket::Shutdown;
use rocket::Request;
use rocket::response::Redirect;
use rocket::form::Form;
use std::sync::{Arc};
use tokio::sync::{Mutex, MutexGuard};
use rocket_dyn_templates::Template;
use rocket::form::FromForm;

// #[derive(Debug, Clone, Deserialize, Serialize, FromForm)]
// pub(crate) struct ChatMessage {
//     username: String,
//     message: String,
//     timestamp: String,
// }
//
// #[get("/chat")]
// pub fn chat() -> Template {
//     Template::render("chat", &())
// }

// #[get("/stream")]
// pub fn events(queue: &State<Sender<Arc<ChatMessage>>>) -> EventStream![] {
//     let mut rx = queue.subscribe();
//     EventStream! {
//         loop {
//             match rx.recv().await {
//                 Ok(msg) => yield Event::json(msg.as_ref()),
//                 Err(RecvError::Closed) => break,
//                 Err(RecvError::Lagged(_)) => continue,
//             }
//         }
//     }
// }
//
// #[post("/chat", data = "<msg_form>")]
// pub async fn post_message(msg_form: Form<ChatMessage>, queue: &State<Sender<Arc<ChatMessage>>>) -> Redirect {
//     let message = Arc::new(msg_form.into_inner());
//     let _res = queue.send(message.clone());
//     Redirect::to(uri!(chat))
// }

use tungstenite::{accept, Error, Message};
use tungstenite::{Message as WsMessage, WebSocket};
// use std::{
//     net::TcpStream,
// };
use std::collections::HashMap;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
// use std::thread;
use futures_util::{SinkExt, StreamExt, Sink, Stream};
use tokio::sync::Mutex as TokioMutex;
use futures_util::sink::Send;
use futures_util::stream::SplitSink;
// use async_std::stream::StreamExt;
// use futures_util::SinkExt;
// use futures_util::stream::{SplitSink, SplitStream};
// use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::{accept_async, WebSocketStream};
use rocket::http::hyper::body::HttpBody;
use futures_util::TryFutureExt;
// use rocket::serde::json::from_str;
// use tokio::io::{AsyncBufReadExt, Split};
use tokio::net::TcpStream;
use crate::db::DbConn;
use crate::{models, schema};
use crate::schema::chats::table;

#[derive(Debug, Clone, Deserialize, Serialize, FromForm)]
pub(crate) struct ChatMessage {
    pub username: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SocketMessageFormat {
    pub command: String,
    pub message: Option<String>,
}

#[derive(Serialize, Debug)]
struct SendMessage {
    command: String,
    message: String,
}

pub enum SocketCommands {
    FetchMessages,
    NewMessage,
    Nothing,
}

// impl SocketCommands {
//     // pub fn execute(
//     //     &self,
//     //     websocket: &mut WebSocket<TcpStream>,
//     //     messages_db: &Arc<Mutex<Vec<String>>>,
//     //     json_message: SocketMessageFormat,
//     // ) {
//     //     match self {
//     //         SocketCommands::FetchMessages => self.fetch_messages(websocket, messages_db),
//     //         SocketCommands::NewMessage => match json_message.message {
//     //             Some(socket_msg) => {
//     //                 messages_db.lock().unwrap().push(socket_msg.clone());
//     //                 self.send_message(websocket, socket_msg);
//     //             }
//     //             None => {}
//     //         },
//     //         SocketCommands::Nothing => {}
//     //     };
//     // }
//
//     pub fn execute(
//         &self,
//         websocket: &mut WebSocketStream<TcpStream>, // Updated to WebSocketStream
//         messages_db: &Arc<Mutex<Vec<String>>>,
//         json_message: SocketMessageFormat,
//     ) {
//         match self {
//             SocketCommands::FetchMessages => self.fetch_messages(websocket, messages_db),
//             SocketCommands::NewMessage => {
//                 if let Some(socket_msg) = json_message.message.clone() {
//                     messages_db.lock().unwrap().push(socket_msg.clone());
//                     self.send_message(websocket, socket_msg);
//                 }
//             }
//             SocketCommands::Nothing => {}
//         };
//     }
//
//     fn send_message(
//         &self,
//         websocket: &mut WebSocketStream<TcpStream>,
//         message: String,
//     ) {
//         let msg_to_send = serde_json::to_string(&SendMessage {
//             command: String::from("new_message"),
//             message,
//         })
//             .unwrap();
//
//         let _ = websocket.send(Message::Text(msg_to_send)).map_err(|err| {
//             eprintln!("Cannot send message: {}", err);
//         });
//     }
//
//     fn fetch_messages(
//         &self,
//         websocket: &mut WebSocketStream<TcpStream>,
//         messages: &Arc<Mutex<Vec<String>>>,
//     ) {
//         for msg in messages.lock().unwrap().iter() {
//             self.send_message(websocket, msg.clone());
//         }
//     }
//
//     // fn send_message(&self, websocket: &mut WebSocket<TcpStream>, message: String) {
//     //
//     //     let msg_to_send = serde_json::to_string(&SendMessage {
//     //         command: String::from("new_message"),
//     //         message: message,
//     //     })
//     //         .unwrap();
//     //
//     //     let _ = websocket.write_message(Message::Text(msg_to_send)).map_err(|err| {
//     //         eprintln!("cannot Send message, {}", err);
//     //     });
//     // }
//     //
//     // fn fetch_messages(
//     //     &self,
//     //     websocket: &mut WebSocket<TcpStream>,
//     //     messages: &Arc<Mutex<Vec<String>>>,
//     // ) {
//     //     for msg in messages.lock().unwrap().iter() {
//     //         self.send_message(websocket, msg.clone())
//     //     }
//     // }
// }

impl SocketCommands {
    // Updating the `execute` method to be async
    pub async fn execute(
        &self,
        websocket: &mut WebSocketStream<TcpStream>,
        messages_db: &Arc<Mutex<Vec<String>>>,
        json_message: SocketMessageFormat,
    ) {
        match self {
            SocketCommands::FetchMessages => self.fetch_messages(websocket, messages_db).await,
            SocketCommands::NewMessage => {
                if let Some(socket_msg) = json_message.message.clone() {
                    let mut db = messages_db.lock().await; // Await the mutex lock
                    db.push(socket_msg.clone());
                    self.send_message(websocket, socket_msg).await; // Send message asynchronously
                }
            }
            SocketCommands::Nothing => {}
        };
    }

    // Updating `send_message` method to be async
    async fn send_message(
        &self,
        websocket: &mut WebSocketStream<TcpStream>,
        message: String,
    ) {
        let msg_to_send = serde_json::to_string(&SendMessage {
            command: String::from("new_message"),
            message,
        })
            .unwrap();

        if let Err(err) = websocket.send(WsMessage::Text(msg_to_send)).await {
            eprintln!("Cannot send message: {}", err);
        }
    }

    // Updating `fetch_messages` method to be async
    async fn fetch_messages(
        &self,
        websocket: &mut WebSocketStream<TcpStream>,
        messages: &Arc<Mutex<Vec<String>>>,
    ) {
        let db = messages.lock().await; // Await the mutex lock
        for msg in db.iter() {
            self.send_message(websocket, msg.clone()).await; // Send each message asynchronously
        }
    }
}

pub fn match_command_or_message(input: &str) -> SocketCommands {
    match input {
        "fetch_messages" => SocketCommands::FetchMessages,
        "new_message" => SocketCommands::NewMessage,
        _ => SocketCommands::Nothing,
    }
}

#[get("/stream")]
pub fn events(queue: &State<Sender<Arc<ChatMessage>>>) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            match rx.recv().await {
                Ok(msg) => yield Event::json(msg.as_ref()),
                Err(RecvError::Closed) => break,
                Err(RecvError::Lagged(_)) => continue,
            }
        }
    }
}

// #[get("/chat")]
// pub fn chat() -> Template {
//     Template::render("chat", &())
// }

#[get("/chat/<chat_id>")]
pub fn chat(chat_id: i32) -> Template {
    let mut context = HashMap::new();
    context.insert("chat_id", chat_id);
    Template::render("chat", &context)
}


// #[post("/chat", data = "<msg_form>")]
// pub async fn post_message(msg_form: Form<ChatMessage>, queue: &State<Sender<Arc<ChatMessage>>>) -> Redirect {
//     let message = Arc::new(msg_form.into_inner());
//     let _res = queue.send(message.clone());
//     Redirect::to(uri!(chat: chat_id = message.chat_id))
// }


pub async fn handle_connection(
    ws_stream: WebSocketStream<TcpStream>, // Use WebSocketStream here
    messages_db: Arc<Mutex<Vec<String>>>,
) {
    let mut websocket = ws_stream;

    while let Some(msg) = websocket.next().await {
        match msg {
            Ok(message) => {
                if message.is_text() {
                    let text = message.to_text().unwrap();
                    let json_message: SocketMessageFormat =
                        serde_json::from_str(text).unwrap();
                    let command = match_command_or_message(&json_message.command);
                    command.execute(&mut websocket, &messages_db, json_message); // Correct type used here
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {:?}", e);
                break;
            }
        }
    }
}

pub async fn handle_websocket(
    websocket: WebSocketStream<TcpStream>,
    messages_db: Arc<TokioMutex<Vec<String>>>,
    connected_clients: Arc<TokioMutex<Vec<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>>>,
) {
    let (ws_sender, mut ws_receiver) = websocket.split();
    let ws_sender = Arc::new(Mutex::new(ws_sender));

    {
        let db = messages_db.lock().await;
        let mut sender = ws_sender.lock().await;
        for message in db.iter() {
            if let Err(e) = sender.send(Message::Text(message.clone())).await {
                eprintln!("Error sending previous message: {:?}", e);
            }
        }
    }

    {
        let mut clients = connected_clients.lock().await;
        clients.push(Arc::clone(&ws_sender));
    }

    while let Some(message) = ws_receiver.next().await {
        if let Ok(Message::Text(text)) = message {
            let mut db = messages_db.lock().await;
            db.push(text.clone());

            let clients = connected_clients.lock().await;
            for client in clients.iter() {
                let mut client = client.lock().await;
                if let Err(e) = client.send(Message::Text(text.clone())).await {
                    eprintln!("Error sending message to client: {:?}", e);
                }
            }
        }
    }

    {
        let mut clients = connected_clients.lock().await;
        clients.retain(|client| {
            Arc::strong_count(client) > 1
        });
    }
}

// use tokio::{fs::File, io::AsyncWriteExt};
// use uuid::Uuid;
// use std::path::Path;
//
// pub async fn handle_websocket(
//     websocket: WebSocketStream<TcpStream>,
//     messages_db: Arc<TokioMutex<Vec<String>>>,
//     connected_clients: Arc<TokioMutex<Vec<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>>>,
// ) {
//     let (ws_sender, mut ws_receiver) = websocket.split();
//     let ws_sender = Arc::new(Mutex::new(ws_sender));
//
//     {
//         let db = messages_db.lock().await;
//         let mut sender = ws_sender.lock().await;
//         for message in db.iter() {
//             if let Err(e) = sender.send(Message::Text(message.clone())).await {
//                 eprintln!("Error sending previous message: {:?}", e);
//             }
//         }
//     }
//
//     {
//         let mut clients = connected_clients.lock().await;
//         clients.push(Arc::clone(&ws_sender));
//     }
//
//     let mut current_filename = None;
//
//     while let Some(message) = ws_receiver.next().await {
//         match message {
//             Ok(Message::Text(text)) => {
//                 // Parse metadata if the message contains file information
//                 if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&text) {
//                     if let Some(file_name) = metadata.get("file_name").and_then(|f| f.as_str()) {
//                         current_filename = Some(file_name.to_string());
//                     } else {
//                         // Otherwise, it's a regular text message
//                         let mut db = messages_db.lock().await;
//                         db.push(text.clone());
//
//                         let clients = connected_clients.lock().await;
//                         for client in clients.iter() {
//                             let mut client = client.lock().await;
//                             if let Err(e) = client.send(Message::Text(text.clone())).await {
//                                 eprintln!("Error sending message to client: {:?}", e);
//                             }
//                         }
//                     }
//                 }
//             }
//             Ok(Message::Binary(data)) => {
//                 // if let Some(original_name) = current_filename.take() {
//                 //     // Extract file extension from the original file name
//                 //     let extension = Path::new(&original_name)
//                 //         .extension()
//                 //         .and_then(|ext| ext.to_str())
//                 //         .unwrap_or("bin");
//                 //
//                 //     let file_name = format!("{}.{}", Uuid::new_v4(), extension);
//                 //     let file_path = format!("media/{}", file_name);
//                 //
//                 //     // Save the file
//                 //     let mut file = File::create(&file_path).await.expect("Failed to create file");
//                 //     file.write_all(&data).await.expect("Failed to write data");
//                 //
//                 //     // Notify clients about the file upload
//                 //     let file_message = format!("File uploaded: {}", file_path);
//                 //     let mut db = messages_db.lock().await;
//                 //     db.push(file_message.clone());
//                 //
//                 //     let clients = connected_clients.lock().await;
//                 //     for client in clients.iter() {
//                 //         let mut client = client.lock().await;
//                 //         if let Err(e) = client.send(Message::Text(file_message.clone())).await {
//                 //             eprintln!("Error sending file path to client: {:?}", e);
//                 //         }
//                 //     }
//                 // } else {
//                 //     eprintln!("No filename provided; cannot save file.");
//                 // }
//                 if let Some(original_name) = current_filename.take() {
//                     // Extract file extension and save with UUID
//                     let extension = Path::new(&original_name)
//                         .extension()
//                         .and_then(|ext| ext.to_str())
//                         .unwrap_or("bin");
//                     let file_name = format!("{}.{}", Uuid::new_v4(), extension);
//                     let file_path = format!("media/{}", file_name);
//
//                     let mut file = File::create(&file_path).await.expect("Failed to create file");
//                     file.write_all(&data).await.expect("Failed to write data");
//
//                     // Message to clients with file path
//                     let file_message = format!("File uploaded: <a href='/media/{}' target='_blank'>{}</a>", file_name, original_name);
//                     let mut db = messages_db.lock().await;
//                     let clients = connected_clients.lock().await;
//                     db.push(file_message.clone());
//
//                     for client in clients.iter() {
//                         let mut client = client.lock().await;
//                         client.send(Message::Text(file_message.clone())).await.unwrap();
//                     }
//                 }
//             }
//             Err(e) => {
//                 eprintln!("Error processing message: {:?}", e);
//             }
//             _ => {}
//         }
//     }
//
//     {
//         let mut clients = connected_clients.lock().await;
//         clients.retain(|client| Arc::strong_count(client) > 1);
//     }
// }

// pub async fn handle_websocket(
//     websocket: WebSocketStream<TcpStream>,
//     messages_db: Arc<TokioMutex<Vec<String>>>,
//     connected_clients: Arc<TokioMutex<Vec<Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>>>>,
// ) {
//     let (ws_sender, mut ws_receiver) = websocket.split();
//     let ws_sender = Arc::new(Mutex::new(ws_sender));
//
//     {
//         let db = messages_db.lock().await;
//         let mut sender = ws_sender.lock().await;
//         for message in db.iter() {
//             if let Err(e) = sender.send(Message::Text(message.clone())).await {
//                 eprintln!("Error sending previous message: {:?}", e);
//             }
//         }
//     }
//
//     {
//         let mut clients = connected_clients.lock().await;
//         clients.push(Arc::clone(&ws_sender));
//     }
//
//     while let Some(message) = ws_receiver.next().await {
//         match message {
//             Ok(Message::Text(text)) => {
//                 // Handle text messages
//                 let mut db = messages_db.lock().await;
//                 db.push(text.clone());
//
//                 let clients = connected_clients.lock().await;
//                 for client in clients.iter() {
//                     let mut client = client.lock().await;
//                     if let Err(e) = client.send(Message::Text(text.clone())).await {
//                         eprintln!("Error sending message to client: {:?}", e);
//                     }
//                 }
//             }
//             Ok(Message::Binary(data)) => {
//                 // Handle binary file uploads
//                 let file_name = format!("{}.bin", Uuid::new_v4());
//                 let file_path = format!("media/{}", file_name);
//
//                 let mut file = File::create(&file_path).await.expect("Failed to create file");
//                 file.write_all(&data).await.expect("Failed to write data");
//
//                 let file_message = format!("File uploaded: {}", file_name);
//                 let mut db = messages_db.lock().await;
//                 db.push(file_message.clone());
//
//                 let clients = connected_clients.lock().await;
//                 for client in clients.iter() {
//                     let mut client = client.lock().await;
//                     if let Err(e) = client.send(Message::Text(file_message.clone())).await {
//                         eprintln!("Error sending file path to client: {:?}", e);
//                     }
//                 }
//             }
//             Err(e) => {
//                 eprintln!("Error processing message: {:?}", e);
//             }
//             _ => {}
//         }
//     }
//
//     {
//         let mut clients = connected_clients.lock().await;
//         clients.retain(|client| Arc::strong_count(client) > 1);
//     }
// }

// pub async fn handle_websocket(
//     websocket: WebSocketStream<tokio::net::TcpStream>,
//     db_conn: &DbConn,
// ) {
//     // Split the WebSocket stream into sender and receiver
//     let (mut ws_sender, mut ws_receiver) = websocket.split();
//
//     // Fetch previous messages from the database
//     let previous_messages = db_conn.run(|c| {
//         use crate::schema::messages::dsl::*;
//         messages.select(models::Message::as_select()).load::<models::Message>(c)
//     }).await.expect("Error loading messages");
//
//     // Send previous messages to the client
//     for message in previous_messages {
//         if let Some(content) = message.content {
//             let _ = ws_sender.send(WsMessage::Text(content)).await;
//         }
//     }
//
//     // Process incoming WebSocket messages
//     while let Some(message) = ws_receiver.next().await {
//         match message {
//             Ok(WsMessage::Text(text)) => {
//                 println!("Received: {}", text);
//
//                 // Create a new message to save to the database
//                 let new_message = models::NewMessage {
//                     chat_id: 0,
//                     user_id: 0,
//                     content: Some(text.clone()),
//                 };
//
//                 // Save the message to the database
//                 db_conn.run(move |c| {
//                     diesel::insert_into(schema::messages::table)
//                         .values(&new_message)
//                         .execute(c)
//                 }).await.expect("Error saving new message");
//
//                 // Echo the message back to the client
//                 if let Err(e) = ws_sender.send(WsMessage::Text(text)).await {
//                     eprintln!("Error sending message: {:?}", e);
//                 }
//             }
//             Err(e) => {
//                 eprintln!("Error receiving message: {:?}", e);
//                 break;
//             }
//             _ => {}
//         }
//     }
// }

// #[get("/ws")]
// pub fn websocket_handler(
//     messages_db: &State<Arc<Mutex<Vec<String>>>>,
//     _shutdown: Shutdown,
// ) -> Result<(), ()> {
//     // Clone the State directly into a new Arc
//     let messages_db_clone = Arc::clone(messages_db.inner()); // Get the inner Arc
//
//     // Spawn the server listener in a tokio task
//     tokio::spawn(async move {
//         let server = tokio::net::TcpListener::bind("127.0.0.1:9001")
//             .await
//             .expect("Failed to bind WebSocket server");
//
//         loop {
//             match server.accept().await {
//                 Ok((stream, _)) => {
//                     let websocket = tokio_tungstenite::accept_async(stream)
//                         .await
//                         .expect("Error during WebSocket handshake");
//
//                     // Clone the Arc inside the loop for each connection
//                     let messages_db_clone = Arc::clone(&messages_db_clone); // Clone for each task
//
//                     // Pass the cloned Arc directly to the handler
//                     tokio::spawn(async move {
//                         handle_websocket(websocket, messages_db_clone).await; // Pass the Arc directly
//                     });
//                 }
//                 Err(e) => {
//                     eprintln!("Error accepting connection: {:?}", e);
//                 }
//             }
//         }
//     });
//
//     Ok(())
// }

// #[get("/ws")]
// pub async fn websocket_handler(
//     messages_db: &State<Arc<Mutex<Vec<String>>>>,
//     shutdown: Shutdown,
// ) {
//     let messages_db = Arc::clone(&messages_db);
//     let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();
//
//     loop {
//         // Await for an incoming connection
//         match listener.accept().await {
//             Ok((stream, _)) => {
//                 let messages_db = Arc::clone(&messages_db);
//                 tokio::spawn(async move {
//                     // Accept the WebSocket connection
//                     let mut websocket = accept_async(stream).await.unwrap();
//
//                     while let Some(msg) = websocket.next().await {
//                         match msg {
//                             Ok(Message::Text(text)) => {
//                                 let json_message: SocketMessageFormat = serde_json::from_str(&text).unwrap();
//                                 let command = match_command_or_message(&json_message.command);
//                                 command.execute(&mut websocket, &messages_db, json_message).await; // Ensure execute is async if needed
//                             }
//                             _ => {}
//                         }
//                     }
//                 });
//             }
//             Err(e) => {
//                 eprintln!("Failed to accept connection: {:?}", e);
//                 // Optionally handle shutdown logic here
//                 break; // Or continue depending on your error handling strategy
//             }
//         }
//     }
// }
