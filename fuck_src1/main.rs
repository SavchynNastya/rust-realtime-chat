#[macro_use] extern crate rocket;

mod auth;
mod db;
// mod routes;
// mod schema;
pub mod schema;

use std::sync::Arc;
pub use schema::{chat_rooms, files, messages, user_rooms, users};
mod models;
mod chat;
mod ws;

use rocket::{Build, Rocket};
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::database;
use crate::db::ChatDb;
use crate::ws::ChatMessage;
use rocket::tokio::sync::broadcast::{Sender, channel, error::RecvError};

// #[database("chat_db")]
// pub struct ChatDb(diesel::SqliteConnection);

#[launch]
fn rocket() -> Rocket<Build> {
    let (sender, _receiver) = channel(100);

    rocket::build()
        .attach(ChatDb::fairing())
        .attach(Template::fairing())
        .manage(sender)
        // .mount("/", routes::all_routes())
        .mount("/api", routes![chat::create_room, chat::list_rooms, chat::send_message, ws::chat_ws])
        .mount("/auth", routes![auth::register, auth::login])
        .mount("/", routes![chat::user_chats])
}
