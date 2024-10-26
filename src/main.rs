#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_sync_db_pools;

use std::sync::{Arc};
use tokio::sync::Mutex;
use rocket_csrf_token::{CsrfConfig, Fairing};
use dotenv::dotenv;
use rocket::{Build, Rocket};
use rocket_dyn_templates::Template;
use tokio::{net::TcpListener, task};
use tokio_tungstenite::accept_async;

mod db;
mod schema;
mod models;
mod auth;
mod websockets;
mod messages;
mod files;
mod fairing;

fn rocket() -> Rocket<Build> {
    dotenv().ok();

    rocket::build()
        .attach(db::DbConn::fairing())
        .attach(Template::fairing())
        .attach(Fairing::new(CsrfConfig::default()))
        .manage(Arc::new(Mutex::new(Vec::<String>::new())))
        .mount("/", routes![
            auth::register_form,
            auth::register,
            auth::login_form,
            auth::login,
            messages::index,
            messages::chat,
            messages::create_chat_form,
            messages::create_chat,
            messages::users_list,
            messages::get_chats,
        ])
        .mount("/media", rocket::fs::FileServer::from("media"))
}

#[tokio::main]
async fn main() {
    use tokio::sync::Mutex;
    let messages_db = Arc::new(Mutex::new(Vec::new()));
    let connected_clients = Arc::new(Mutex::new(Vec::new()));

    let rocket_instance = rocket();

    let ws_listener = TcpListener::bind("127.0.0.1:9001")
        .await
        .expect("Failed to bind WebSocket server");

    tokio::spawn(async move {
        loop {
            match ws_listener.accept().await {
                Ok((stream, _)) => {
                    let messages_db_clone = Arc::clone(&messages_db);
                    let connected_clients_clone = Arc::clone(&connected_clients); // Share the same list

                    tokio::spawn(async move {
                        let websocket = accept_async(stream)
                            .await
                            .expect("Error during WebSocket handshake");

                        websockets::handle_websocket(
                            websocket,
                            messages_db_clone,
                            connected_clients_clone,
                        ).await;
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    });

    rocket_instance.launch().await.expect("Failed to launch Rocket");
}