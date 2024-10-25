use bcrypt::{hash, verify};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use crate::models;
use crate::schema::users;


// Users table
#[derive(Queryable, Insertable, Serialize, Deserialize)]
#[table_name = "users"]
pub struct User {
    pub id: i32, // Change to i32
    pub username: String,
    pub email: String,
    pub hashed_password: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub hashed_password: &'a str,
}

impl User {
    pub fn verify_password(&self, password: &str) -> bool {
        verify(password, &self.hashed_password).unwrap_or(false)
    }
}

impl<'a> NewUser<'a> {
    pub fn create(username: &'a str, email: &'a str, password: &'a str, conn: &SqliteConnection) -> QueryResult<User> {
        let password_hash = hash(password, 4).unwrap();
        let new_user = NewUser {
            username,
            email,
            hashed_password: &password_hash,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(conn)?;

        users::table
            .order(users::id.desc())
            .first(conn)
    }
}

// Registration information sent by the user
#[derive(Debug, Deserialize)]
pub struct RegisterInfo {
    pub username: String,
    pub email: String,
    pub password: String,
}

// Login information sent by the user
#[derive(Debug, Deserialize)]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
}


// Chat rooms table
#[derive(Queryable, Insertable, Serialize, Deserialize)]
#[table_name = "chat_rooms"]
pub struct ChatRoom {
    pub id: i32, // Change to i32
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
}

// Messages table
#[derive(Queryable, Insertable, Serialize, Deserialize)]
#[table_name = "messages"]
pub struct Message {
    pub id: i32, // Change to i32
    pub content: String,
    pub user_id: i32, // Change to i32
    pub room_id: i32, // Change to i32
    pub sent_at: NaiveDateTime,
}

// Files table
#[derive(Queryable, Insertable, Serialize, Deserialize)]
#[table_name = "files"]
pub struct File {
    pub id: i32, // Change to i32
    pub filename: String,
    pub filepath: String,
    pub user_id: i32, // Change to i32
    pub room_id: Option<i32>, // Change to i32
    pub uploaded_at: NaiveDateTime,
}

// User-room relationship table
#[derive(Queryable, Insertable, Serialize, Deserialize)]
#[table_name = "user_rooms"]
pub struct UserRoom {
    pub id: i32, // Change to i32
    pub user_id: i32, // Change to i32
    pub room_id: i32, // Change to i32
    pub joined_at: NaiveDateTime,
}
