use rocket::serde::{Deserialize, Serialize};
use rocket_sync_db_pools::diesel::insert_into;
use rocket_sync_db_pools::diesel::query_builder;
use crate::schema::messages;
use crate::db::DbConn;
use rocket::response::Redirect;
use rocket::serde::json::Value;
use diesel::prelude::*;
use crate::fairing::AuthenticatedUser;
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

#[derive(Serialize)]
struct ChatTemplate {
    id: Option<i32>,
    user1_username: String,
    user2_username: String,
    created_at: Option<NaiveDateTime>,
    chat_url: String,
}

#[post("/save_message", data = "<message>")]
pub async fn save_message(message: Json<models::NewMessage>, conn: DbConn) -> Redirect {
    conn.run(move |c: &mut rocket_sync_db_pools::diesel::SqliteConnection| {
        diesel::insert_into(messages::table)
            .values(message.into_inner())
            .execute(c)
    }).await.expect("Error saving new message");

    Redirect::to(uri!(index))
}


pub async fn get_chat_with_users(mut conn: DbConn, chat_id: i32, auth_user: AuthenticatedUser) -> Result<models::ChatWithUsers, diesel::result::Error> {
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
pub async fn create_chat_form(conn: DbConn, csrf: CsrfToken, auth_user: AuthenticatedUser) -> Result<Template, Status> {
    use crate::schema::users::dsl::users;

    let users_list = conn
        .run(|c| users.load::<models::User>(c))
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut context: HashMap<&str, Value> = HashMap::new();
    context.insert("users", json!(users_list));

    if let Some(user) = auth_user.user {
        context.insert("user_authenticated", json!(true));
        context.insert("user", serde_json::to_value(user).unwrap());
    } else {
        context.insert("user_authenticated", json!(false));
    }

    match csrf.authenticity_token() {
        Ok(token) => {
            context.insert("csrf_token", json!(token));
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
    csrf_token: CsrfToken,
    auth_user: AuthenticatedUser
) -> Result<Redirect, Status> {
    use crate::schema::chats::dsl::*;
    use diesel::prelude::*;

    if let Some(_user_id) = cookies.get_private("user_id") {
        if csrf_token.verify(&chat_data.authenticity_token).is_err() {
            return Err(Status::Forbidden);
        }

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

        let inserted_chat = conn
            .run(move |c| {
                chats.order(id.desc())
                    .first::<models::Chat>(c)
            })
            .await
            .map_err(|_| Status::InternalServerError)?;

        Ok(Redirect::to(uri!(get_chats)))
    } else {
        Err(Status::Unauthorized)
    }
}

#[get("/chats")]
pub async fn get_chats(conn: DbConn, cookies: &CookieJar<'_>, auth_user: AuthenticatedUser) -> Result<Template, Status> {
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
                                let chat_url = uri!(crate::messages::chat(chat_id.unwrap_or(0))).to_string();
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

        let mut context: HashMap<&str, Value> = HashMap::new();
        context.insert("chats", json!(chats_list));
        if let Some(user) = auth_user.user {
            context.insert("user_authenticated", json!(true));
            context.insert("user", serde_json::to_value(user).unwrap());
        } else {
            context.insert("user_authenticated", json!(false));
        }

        Ok(Template::render("chats", &context))
    } else {
        Err(Status::Unauthorized)
    }
}

#[get("/users")]
pub async fn users_list(conn: DbConn, auth_user: AuthenticatedUser) -> Result<Template, Status> {
    use crate::schema::users::dsl::*;

    let users_list = conn
        .run(|c| users.load::<models::User>(c))
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut context = HashMap::new();
    context.insert("users", json!(users_list));
    if let Some(user) = auth_user.user {
        context.insert("user_authenticated", json!(true));
        context.insert("user", serde_json::to_value(user).unwrap());
    } else {
        context.insert("user_authenticated", json!(false));
    }

    Ok(Template::render("users", &context))
}

#[get("/chat/<chat_id>")]
pub fn chat(chat_id: i32, auth_user: AuthenticatedUser) -> Template {
    let mut context = HashMap::new();
    if let Some(user) = auth_user.user {
        context.insert("user_authenticated", json!(true));
        context.insert("user", serde_json::to_value(user).unwrap());
    } else {
        context.insert("user_authenticated", json!(false));
    }
    context.insert("chat_id", json!(chat_id));
    Template::render("chat", &context)
}

#[get("/")]
pub async fn index(cookies: &CookieJar<'_>, auth_user: AuthenticatedUser) -> Template {
    let mut context = HashMap::new();
    if let Some(user) = auth_user.user {
        context.insert("user_authenticated", json!(true));
        context.insert("user", serde_json::to_value(user).unwrap());
    } else {
        context.insert("user_authenticated", json!(false));
    }
    Template::render("base", &context)
}
