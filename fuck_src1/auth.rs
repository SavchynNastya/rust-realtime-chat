use rocket::serde::json::Json;
use rocket::State;
use bcrypt::{hash, verify};
use diesel::prelude::*;
use diesel::RunQueryDsl;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;

use crate::models::{User, NewUser, LoginInfo, RegisterInfo};
use crate::schema::users::dsl::*;  // Use schema DSL for convenience
use crate::db::ChatDb;

#[post("/register", data = "<info>")]
pub async fn register(info: Json<RegisterInfo>, conn: ChatDb) -> Result<Json<User>, &'static str> {
    // Hash the provided password
    let password_hash = hash(&info.password, 4).unwrap();

    // Create the new user data
    let new_user = NewUser {
        username: &info.username,
        email: &info.email,
        hashed_password: &password_hash,
    };

    // Insert the new user into the database
    let result = conn.run(|c| {
        diesel::insert_into(users)
            .values(&new_user)
            .execute(c)
    }).await;

    match result {
        Ok(_) => {
            // Retrieve the newly created user to return
            let created_user = conn.run(|c| {
                users.order(id.desc()).first::<User>(c)
            }).await;

            match created_user {
                Ok(user) => Ok(Json(user)),
                Err(_) => Err("Failed to retrieve the created user"),
            }
        }
        Err(_) => Err("User registration failed"),
    }
}

// #[post("/login", data = "<info>")]
// pub async fn login(info: Json<LoginInfo>, conn: ChatDb) -> Result<Json<User>, &'templates str> {
//     // Query the user by username
//     let user = conn.run(|c| users.filter(username.eq(&info.username)).first::<User>(c)).await;
//
//     match user {
//         Ok(user) => {
//             // Verify the provided password against the stored hash
//             if verify(&info.password, &user.hashed_password).unwrap() {
//                 Ok(Json(user))
//             } else {
//                 Err("Invalid password")
//             }
//         }
//         Err(_) => Err("User not found"),
//     }
// }
#[post("/login", data = "<info>")]
pub async fn login(
    info: Json<LoginInfo>,
    conn: ChatDb,
    cookies: &CookieJar<'_>
) -> Result<Redirect, &'static str> {
    use crate::schema::users::dsl::*;

    // Search for the user by username
    let user_result = conn.run(move |c| {
        users.filter(username.eq(&info.username)).first::<User>(c)
    }).await;

    match user_result {
        Ok(user) => {
            // Verify the password using bcrypt
            if verify(&info.password, &user.hashed_password).unwrap() {
                // Add user ID to cookies
                cookies.add_private(Cookie::new("user_id", user.id.to_string()));

                // Redirect to the chat page after successful login
                Ok(Redirect::to(uri!(user_chats)))
            } else {
                Err("Invalid password")
            }
        }
        Err(_) => Err("User not found"),
    }
}


fn is_authenticated(cookies: &CookieJar<'_>) -> bool {
    cookies.get_private("user_id").is_some()
}