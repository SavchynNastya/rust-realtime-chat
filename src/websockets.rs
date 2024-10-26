use std::sync::Arc;
use futures_util::{Sink, SinkExt, StreamExt};
use futures_util::stream::SplitSink;
use rocket::http::hyper::body::HttpBody;
use rocket::serde::Deserialize;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;


#[derive(Debug, Deserialize)]
struct ChatMessage {
    username: String,
    message: String,
    user_id: i32,
    chat_id: i32,
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
            match serde_json::from_str::<ChatMessage>(&text) {
                Ok(parsed_message) => {
                    {
                        let mut db = messages_db.lock().await;
                        db.push(text.clone());
                    }
                    let clients = connected_clients.lock().await;
                    for client in clients.iter() {
                        let mut client = client.lock().await;
                        if let Err(e) = client.send(Message::Text(text.clone())).await {
                            eprintln!("Error sending message to client: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse message: {:?}", e);
                    continue;
                }
            }
        }
    }

    {
        let mut clients = connected_clients.lock().await;
        clients.retain(|client| Arc::strong_count(client) > 1);
    }
}
