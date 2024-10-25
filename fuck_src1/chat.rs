use rocket::http::{CookieJar, Status};
use rocket_dyn_templates::{context, Template};
use crate::db::ChatDb;
use crate::models::{ChatRoom, Message};
use crate::schema::chat_rooms::dsl::*;
use crate::schema::user_rooms::dsl::*;
use crate::schema::users::dsl::*;
use crate::schema::messages::dsl::*;

// #[get("/chats")]
// pub async fn user_chats(conn: ChatDb, cookies: &CookieJar<'_>) -> Template {
//     use crate::schema::chat_rooms::dsl::*;
//
//     // Отримуємо ID користувача з кукі
//     if let Some(user_id_cookie) = cookies.get_private("user_id") {
//         let user_id = user_id_cookie.value().parse::<i32>().unwrap();
//
//         // Отримуємо чати, до яких приєднаний користувач
//         let user_chats_result = conn.run(move |c| {
//             chat_rooms.filter(user_rooms::dsl::user_id.eq(user_id))
//                 .load::<ChatRoom>(c)
//         }).await;
//
//         match user_chats_result {
//             Ok(chats) => {
//                 let context = context! { chats: chats };
//                 Template::render("user_chats", &context)
//             }
//             Err(_) => Template::render("error", &context! { error: "Could not load chat rooms" }),
//         }
//     } else {
//         Template::render("error", &context! { error: "Unauthorized" })
//     }
// }


use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::response::status;
use rocket::post;
use rocket::get;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use chrono::NaiveDateTime;

type DbConn = r2d2::Pool<ConnectionManager<SqliteConnection>>;

// // Define your ChatRoom and Message structs
// #[derive(Queryable, Insertable, Serialize, Deserialize)]
// #[table_name = "chat_rooms"]
// pub struct ChatRoom {
//     pub id: i32,
//     pub name: String,
//     pub description: Option<String>,
//     pub created_at: NaiveDateTime,
// }
//
// #[derive(Queryable, Insertable, Serialize, Deserialize)]
// #[table_name = "messages"]
// pub struct Message {
//     pub id: i32,
//     pub content: String,
//     pub user_id: i32,
//     pub room_id: i32,
//     pub sent_at: NaiveDateTime,
// }

// Endpoint to create a new chat room
// #[post("/rooms", format = "json", data = "<chat_room>")]
// pub async fn create_room(chat_room: Json<ChatRoom>, conn: DbConn) -> Result<Json<ChatRoom>, status::BadRequest<String>> {
//     let new_chat_room = ChatRoom {
//         id: 0, // Auto-incremented
//         name: chat_room.name.clone(),
//         description: chat_room.description.clone(),
//         created_at: chrono::Local::now().naive_utc(),
//     };
//
//     // Save to database
//     let mut connection = conn.get().map_err(|e| status::BadRequest(Some(format!("Database error: {}", e))))?;
//     diesel::insert_into(chat_rooms::table)
//         .values(&new_chat_room)
//         .execute(&mut connection)
//         .map_err(|e| status::BadRequest(Some(format!("Failed to create room: {}", e))))?;
//
//     Ok(Json(new_chat_room))
// }
//
// // Endpoint to list all chat rooms
// #[get("/rooms")]
// pub async fn list_rooms(conn: DbConn) -> Result<Json<Vec<ChatRoom>>, status::NotFound<String>> {
//     let mut connection = conn.get().map_err(|e| status::NotFound(format!("Database error: {}", e)))?;
//     let rooms = chat_rooms::table.load::<ChatRoom>(&mut connection)
//         .map_err(|e| status::NotFound(format!("Failed to retrieve rooms: {}", e)))?;
//
//     Ok(Json(rooms))
// }
//
// // Endpoint to send a message
// #[post("/messages", format = "json", data = "<message>")]
// pub async fn send_message(message: Json<Message>, conn: DbConn) -> Result<Json<Message>, status::BadRequest<String>> {
//     // Save the message to the database
//     let mut connection = conn.get().map_err(|e| status::BadRequest(Some(format!("Database error: {}", e))))?;
//     let new_message = Message {
//         id: 0, // Auto-incremented
//         content: message.content.clone(),
//         user_id: message.user_id,
//         room_id: message.room_id,
//         sent_at: chrono::Local::now().naive_utc(),
//     };
//
//     diesel::insert_into(messages::table)
//         .values(&new_message)
//         .execute(&mut connection)
//         .map_err(|e| status::BadRequest(Some(format!("Failed to send message: {}", e))))?;
//
//     Ok(Json(new_message))
// }

use diesel::prelude::*;
use rocket::tokio::sync::broadcast::{Sender, channel, error::RecvError};
use rocket::State;
use crate::schema::chat_rooms::dsl::*;
use std::sync::Arc;
use futures_util::SinkExt;

#[post("/rooms", format = "json", data = "<chat_room>")]
pub async fn create_room(chat_room: Json<ChatRoom>, conn: DbConn) -> Result<Json<ChatRoom>, Status> {
    let new_chat_room = ChatRoom {
        id: 0, // Auto-incremented
        name: chat_room.name.clone(),
        description: chat_room.description.clone(),
        created_at: chrono::Local::now().naive_utc(),
    };

    // Save to database
    let mut connection = conn.get().map_err(|e| Status::BadRequest)?;
    diesel::insert_into(chat_rooms::table)
        .values(&new_chat_room)
        .execute(&mut connection)
        .map_err(|_| Status::BadRequest)?;

    Ok(Json(new_chat_room))
}

// Endpoint to list all chat rooms
#[get("/rooms")]
pub async fn list_rooms(conn: DbConn) -> Result<Json<Vec<ChatRoom>>, Status> {
    let mut connection = conn.get().map_err(|e| Status::NotFound)?;
    let rooms = chat_rooms::table.load::<ChatRoom>(&mut connection)
        .map_err(|_| Status::NotFound)?;

    Ok(Json(rooms))
}

// Endpoint to send a message
#[post("/messages", format = "json", data = "<message>")]
pub async fn send_message(message: Json<Message>, conn: DbConn, queue: &State<Sender<Arc<Message>>>) -> Result<Json<Message>, Status> {
    // Save the message to the database
    let mut connection = conn.get().map_err(|e| Status::BadRequest)?;
    let new_message = Message {
        id: 0, // Auto-incremented
        content: message.content.clone(),
        user_id: message.user_id,
        room_id: message.room_id,
        sent_at: chrono::Local::now().naive_utc(),
    };

    diesel::insert_into(messages::table)
        .values(&new_message)
        .execute(&mut connection)
        .map_err(|_| Status::BadRequest)?;

    // Broadcast the new message to all subscribers
    let _ = queue.send(Arc::new(new_message.clone())); // Broadcast the new message

    Ok(Json(new_message))
}


#[get("/chats")]
pub async fn user_chats(conn: ChatDb, cookies: &CookieJar<'_>) -> Template {
    use crate::schema::chat_rooms::dsl::*;

    // Check if user is authenticated
    if let Some(user_id_cookie) = cookies.get_private("user_id") {
        let user_id = user_id_cookie.value().parse::<i32>().unwrap();

        // Get chats that the user is joined in
        let user_chats_result = conn.run(move |c| {
            chat_rooms
                .inner_join(user_rooms::dsl::user_rooms)
                .filter(user_rooms::dsl::user_id.eq(user_id))
                .load::<ChatRoom>(c)
        }).await; // Make sure to await here

        match user_chats_result {
            Ok(chats) => {
                let context = context! {
                    chats: chats,
                    is_authenticated: true // Pass the variable to the template
                };
                Template::render("user_chats", &context)
            }
            Err(_) => Template::render("error", &context! { error: "Could not load chat rooms" }),
        }
    } else {
        // If not authenticated, return an error template or redirect to login
        Template::render("error", &context! { error: "Unauthorized", is_authenticated: false })
    }
}
