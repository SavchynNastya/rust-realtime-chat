use diesel::prelude::*;
use diesel::{Queryable, QueryableByName};
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use crate::schema::{users, chats, messages};

#[derive(Queryable, Serialize, Identifiable, Selectable)]
#[table_name = "users"]
pub struct User {
    pub id: Option<i32>,
    pub username: String,
    pub hashed_password: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Queryable, Serialize, Selectable)]
#[table_name = "chats"]
pub struct Chat {
    pub id: Option<i32>,
    pub user1_id: i32,
    pub user2_id: i32,
    // pub created_at: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Serialize)]
pub struct ChatWithUsers {
    pub id: Option<i32>,
    pub user1: User,
    pub user2: User,
    // pub created_at: String,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Serialize, Selectable)]
#[diesel(table_name = messages)]
pub struct Message {
    pub id: Option<i32>,
    pub chat_id: i32,
    pub user_id: i32,
    pub content: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub hashed_password: String,
    // pub created_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "chats"]
pub struct NewChat {
    pub user1_id: i32,
    pub user2_id: i32,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "messages"]
pub struct NewMessage {
    pub chat_id: i32,  // Added chat_id field
    pub user_id: i32,
    pub content: Option<String>,
}
