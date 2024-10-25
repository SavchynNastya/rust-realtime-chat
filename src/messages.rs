// use rocket::State;
// use rocket::response::Redirect;
// use diesel::prelude::*;
// use rocket::serde::{json::Json, Deserialize, Serialize};
// use crate::schema::messages;
// use crate::models::{NewMessage, Message};
// use crate::db::DbConn;
// use rocket_dyn_templates::Template;
// use rocket_dyn_templates::tera::Context;
//
// #[post("/save_message", data = "<message>")]
// pub async fn save_message(message: Json<NewMessage>, conn: DbConn) -> Redirect {
//     conn.run(move |c| {
//         diesel::insert_into(messages::table)
//             .values(&*message)
//             .execute(c)
//     }).await.unwrap();
//
//     Redirect::to(uri!(index))
// }
//
//
// #[get("/")]
// pub async fn index() -> Template {
//     let context = Context::new();
//     Template::render("index", &context)
// }
// extern crate diesel;
use rocket::serde::{Deserialize, Serialize};
// use diesel::prelude::*;
use rocket_sync_db_pools::diesel::insert_into;
use rocket_sync_db_pools::diesel::query_builder;
use crate::schema::messages;
use crate::db::DbConn;
use rocket::response::Redirect;
use diesel::prelude::*;
use rocket_sync_db_pools::diesel::prelude::*;
use crate::schema::{chats, users};
use rocket_csrf_token::CsrfToken;
use rocket::serde::json::{Json, json};
use std::collections::HashMap;
use chrono::NaiveDateTime;
use diesel::associations::HasTable;
use rocket::form::Form;
use rocket::http::{CookieJar, Status};
use rocket_dyn_templates::Template;
use crate::models;
use crate::schema::users::dsl::*;
use crate::schema::chats::dsl::*;

#[post("/save_message", data = "<message>")]
pub async fn save_message(message: Json<models::NewMessage>, conn: DbConn) -> Redirect {
    conn.run(move |c: &mut rocket_sync_db_pools::diesel::SqliteConnection| {
        diesel::insert_into(messages::table)
            .values(message.into_inner())
            .execute(c)
    }).await.expect("Error saving new message");

    Redirect::to(uri!(index))
}


pub async fn get_chat_with_users(mut conn: DbConn, chat_id: i32) -> Result<models::ChatWithUsers, diesel::result::Error> {
    let (user1, user2) = diesel::alias!(users as user1, users as user2);

    let chat = conn.run(move |c| {
        chats::table
            .filter(chats::id.eq(chat_id))
            .inner_join(user1.on(user1.field(users::id).nullable().eq(chats::user1_id.nullable())))
            .inner_join(user2.on(user2.field(users::id).nullable().eq(chats::user2_id.nullable())))
            .select((
                chats::id,
                (user1.field(users::id).nullable(), user1.field(users::username), user1.field(users::hashed_password), user1.field(users::created_at).nullable()),
                (user2.field(users::id).nullable(), user2.field(users::username), user2.field(users::hashed_password), user2.field(users::created_at).nullable()),
                chats::created_at.nullable(),
            ))
            .first::<models::ChatWithUsers>(c)
    }).await?;

    Ok(chat)
}

#[get("/create_chat")]
pub async fn create_chat_form(conn: DbConn, csrf: CsrfToken) -> Result<Template, Status> {
    use crate::schema::users::dsl::users;

    // Load the list of users from the database
    let users_list = conn
        .run(|c| users.load::<models::User>(c))
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut context = HashMap::new();
    context.insert("users", json!(users_list));

    // Insert the CSRF token directly into the context
    match csrf.authenticity_token() {
        Ok(token) => {
            context.insert("csrf_token", json!(token)); // Convert to serde_json::Value
        },
        Err(_) => {
            return Err(Status::InternalServerError);
        }
    }

    Ok(Template::render("create_chat", &context))
}

#[derive(FromForm)]
pub struct ChatFormData {
    pub authenticity_token: String,
    pub form_user1_id: i32,
    pub form_user2_id: i32,
}

#[post("/create_chat", data = "<chat_data>")]
pub async fn create_chat(
    chat_data: Form<ChatFormData>, // Use the new struct for form data
    conn: DbConn,
    cookies: &CookieJar<'_>,
    csrf_token: CsrfToken
) -> Result<Redirect, Status> {
    use crate::schema::chats::dsl::*;
    use diesel::prelude::*;

    // Check if the user is authenticated
    if let Some(_user_id) = cookies.get_private("user_id") {
        // Verify the CSRF token
        if csrf_token.verify(&chat_data.authenticity_token).is_err() {
            return Err(Status::Forbidden); // Return forbidden if the token is invalid
        }

        // Proceed with inserting the chat
        let new_chat = models::NewChat {
            user1_id: chat_data.form_user1_id,
            user2_id: chat_data.form_user2_id,
        };

        conn.run(move |c| {
            diesel::insert_into(chats)
                .values(&new_chat)
                .execute(c)
        })
            .await
            .map_err(|_| Status::InternalServerError)?;

        // Fetch the newly created chat
        let inserted_chat = conn
            .run(move |c| {
                chats.order(id.desc())
                    .first::<models::Chat>(c)
            })
            .await
            .map_err(|_| Status::InternalServerError)?;

        // Ok(Json(inserted_chat))
        Ok(Redirect::to(uri!(get_chats)))
    } else {
        Err(Status::Unauthorized)
    }
}

// #[get("/chats")]
// pub async fn get_chats(conn: DbConn, cookies: &CookieJar<'_>) -> Result<Template, Status> {
//     use diesel::prelude::*;
//
//     if let Some(_user_id) = cookies.get_private("user_id") {
//         let (user1_alias, user2_alias) = diesel::alias!(users as user1_alias, users as user2_alias);
//
//         let chats_list = conn
//             .run(move |c| {
//                 chats::table
//                     .inner_join(user1_alias.on(user1_alias.field(users::id).nullable().eq(chats::user1_id.nullable())))
//                     .inner_join(user2_alias.on(user2_alias.field(users::id).nullable().eq(chats::user2_id.nullable())))
//                     .select((
//                         chats::id.nullable(),
//                         (
//                             user1_alias.field(users::id).nullable(),
//                             user1_alias.field(users::username),
//                             user1_alias.field(users::created_at).nullable(),
//                             user1_alias.field(users::hashed_password),
//                         ),
//                         (
//                             user2_alias.field(users::id).nullable(),
//                             user2_alias.field(users::username),
//                             user2_alias.field(users::created_at).nullable(),
//                             user2_alias.field(users::hashed_password),
//                         ),
//                         chats::created_at.nullable(),
//                     ))
//                     .load::<(Option<i32>, (Option<i32>, String, Option<NaiveDateTime>, String), (Option<i32>, String, Option<NaiveDateTime>, String), Option<NaiveDateTime>)>(c)
//                     .map(|results| {
//                         results.into_iter()
//                             .map(|(chat_id, user1_data, user2_data, chat_created_at)| {
//                                 let user1 = models::User {
//                                     id: user1_data.0,
//                                     username: user1_data.1,
//                                     created_at: user1_data.2,
//                                     hashed_password: user1_data.3,
//                                 };
//                                 let user2 = models::User {
//                                     id: user2_data.0,
//                                     username: user2_data.1,
//                                     created_at: user2_data.2,
//                                     hashed_password: user2_data.3,
//                                 };
//
//                                 models::ChatWithUsers {
//                                     id: chat_id,
//                                     user1: user1,
//                                     user2: user2,
//                                     created_at: chat_created_at,
//                                 }
//                             })
//                             .collect::<Vec<models::ChatWithUsers>>()
//                     })
//             })
//             .await
//             .map_err(|_| Status::InternalServerError)?;
//
//         let mut context = HashMap::new();
//         context.insert("chats", chats_list);
//
//         Ok(Template::render("chats", &context))
//
//     } else {
//         Err(Status::Unauthorized)
//     }
// }

#[derive(Serialize)]
struct ChatTemplate {
    id: Option<i32>,
    user1_username: String,
    user2_username: String,
    created_at: Option<NaiveDateTime>,
    chat_url: String,
}

#[get("/chats")]
pub async fn get_chats(conn: DbConn, cookies: &CookieJar<'_>) -> Result<Template, Status> {
    use diesel::prelude::*;

    if let Some(_user_id) = cookies.get_private("user_id") {
        let (user1_alias, user2_alias) = diesel::alias!(users as user1_alias, users as user2_alias);

        let chats_list = conn
            .run(move |c| {
                chats::table
                    .inner_join(user1_alias.on(user1_alias.field(users::id).nullable().eq(chats::user1_id.nullable())))
                    .inner_join(user2_alias.on(user2_alias.field(users::id).nullable().eq(chats::user2_id.nullable())))
                    .select((
                        chats::id.nullable(),
                        user1_alias.field(users::username),
                        user2_alias.field(users::username),
                        chats::created_at.nullable(),
                    ))
                    .load::<(Option<i32>, String, String, Option<NaiveDateTime>)>(c)
                    .map(|results| {
                        results.into_iter()
                            .map(|(chat_id, user1_username, user2_username, chat_created_at)| {
                                let chat_url = uri!(crate::websockets::chat(chat_id.unwrap_or(0))).to_string();
                                ChatTemplate {
                                    id: chat_id,
                                    user1_username,
                                    user2_username,
                                    created_at: chat_created_at,
                                    chat_url,
                                }
                            })
                            .collect::<Vec<ChatTemplate>>()
                    })
            })
            .await
            .map_err(|_| Status::InternalServerError)?;

        let mut context = HashMap::new();
        context.insert("chats", chats_list);

        Ok(Template::render("chats", &context))
    } else {
        Err(Status::Unauthorized)
    }
}

#[get("/users")]
pub async fn users_list(conn: DbConn) -> Result<Template, Status> {
    use crate::schema::users::dsl::*;

    let users_list = conn
        .run(|c| users.load::<models::User>(c))
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut context = HashMap::new();
    // if let Some(user) = request.local_cache::<Option<User>>() {
    //     context.insert("user_authenticated", user.is_some());
    //     if let Some(user) = user {
    //         context.insert("user", user);
    //     }
    // } else {
    //     context.insert("user_authenticated", false);
    // }
    context.insert("users", users_list);

    Ok(Template::render("users", &context))
}

#[get("/")]
pub async fn index(cookies: &CookieJar<'_>) -> Template {
    let mut context = HashMap::new();
    let user_authenticated = cookies.get_private("user_id").is_some();
    context.insert("user_authenticated", user_authenticated);
    Template::render("base", &context)
}
