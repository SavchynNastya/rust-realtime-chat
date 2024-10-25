// use bcrypt::{hash, verify, DEFAULT_COST};
// use diesel::prelude::*;
// use rocket::form::Form;
// use rocket::http::{Cookie, CookieJar};
// use rocket::response::Redirect;
// use rocket::serde::{Deserialize, Serialize};
// use rocket_dyn_templates::Template;
// use rocket::State;
//
// use crate::db::DbConn;
// use crate::models::{User, NewUser};
// use crate::schema::users;
//
// #[derive(Debug, FromForm, Deserialize, Serialize)]
// pub struct RegisterForm {
//     pub username: String,
//     pub password: String,
// }
//
// #[derive(Debug, FromForm, Deserialize, Serialize)]
// pub struct LoginForm {
//     pub username: String,
//     pub password: String,
// }
//
// #[post("/register", data = "<form>")]
// pub async fn register(form: Form<RegisterForm>, conn: DbConn) -> Redirect {
//     let new_user = NewUser {
//         username: form.username.clone(),
//         password: hash(&form.password, DEFAULT_COST).unwrap(),
//     };
//
//     conn.run(move |c| {
//         diesel::insert_into(users::table)
//             .values(&new_user)
//             .execute(c)
//     }).await.expect("Error saving new user");
//
//     Redirect::to(uri!(login_form))
// }
//
// #[post("/login", data = "<form>")]
// pub async fn login(form: Form<LoginForm>, conn: DbConn, cookies: &CookieJar<'_>) -> Redirect {
//     let user_result: Result<User, _> = conn.run(move |c| {
//         users::table
//             .filter(users::username.eq(&form.username))
//             .first::<User>(c)
//     }).await;
//
//     if let Ok(user) = user_result {
//         if verify(&form.password, &user.password).unwrap() {
//             cookies.add_private(Cookie::new("user_id", user.id.to_string()));
//             return Redirect::to(uri!(super::messages::index));
//         }
//     }
//
//     Redirect::to(uri!(login_form))
// }
//
// #[get("/register")]
// pub async fn register_form() -> Template {
//     Template::render("register", ())
// }
//
// #[get("/login")]
// pub async fn login_form() -> Template {
//     Template::render("login", ())
// }

extern crate diesel;

use std::collections::HashMap;
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::prelude::*;
use rocket_sync_db_pools::diesel::prelude::*;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::{database};
use rocket::State;

use crate::db::DbConn;
use crate::models::{User, NewUser};
use crate::schema::users;
use crate::schema::users::dsl::*;

use crate::schema::users::username;

#[derive(Debug, FromForm, Deserialize, Serialize)]
pub struct RegisterForm {
    pub user_name: String,
    pub password: String,
}

#[derive(Debug, FromForm, Deserialize, Serialize)]
pub struct LoginForm {
    pub user_name: String,
    pub password: String,
}

// #[post("/register", data = "<form>")]
// pub async fn register(form: Form<RegisterForm>, conn: DbConn) -> Redirect {
//     let new_user = NewUser {
//         username: form.username.clone(),
//         password_hash: hash(&form.password, DEFAULT_COST).unwrap(),
//     };
//
//     conn.run(move |c: &diesel::SqliteConnection| {
//         diesel::insert_into(users::table)
//             .values(&new_user)
//             .execute(c)
//     }).await.expect("Error saving new user");
//
//     Redirect::to(uri!(login_form))
// }

#[post("/register", data = "<form>")]
pub async fn register(form: Form<RegisterForm>, conn: DbConn) -> Redirect {
    let new_user = NewUser {
        username: form.user_name.clone(),
        hashed_password: hash(&form.password, DEFAULT_COST).unwrap(),
    };

    // conn.run(move |c: &mut diesel::SqliteConnection| {
    //     diesel::insert_into(users::table)
    //         .values(&new_user)
    //         .execute(c)
    // }).await.expect("Error saving new user");

    // conn.run(move |c: &mut rocket_sync_db_pools::diesel::SqliteConnection| {
    //     diesel::insert_into(users::table)
    //         .values(&new_user)
    //         .execute(c)
    // }).await.expect("Error saving new user");

    // conn.run(move |c| {
    //     diesel::insert_into(users::table)
    //         .values(&new_user)
    //         .execute(c)
    // }).await.expect("Error saving new user");

    let result = conn.run(move |c| {
        use diesel::prelude::*;
        // use crate::schema::users::dsl::*;

        diesel::insert_into(users)
            .values(&new_user)
            .execute(c)
    }).await;

    Redirect::to(uri!(login_form))
}

// #[post("/login", data = "<form>")]
// pub async fn login(form: Form<LoginForm>, conn: DbConn, cookies: &CookieJar<'_>) -> Redirect {
//     let user_result: Result<User, _> = conn.run(move |c: &mut rocket_sync_db_pools::diesel::SqliteConnection| {
//         users::table
//             .filter(username.eq(&form.user_name))
//             .first::<User>(c)
//     }).await;
//
//     if let Ok(user) = user_result {
//         if verify(&form.password, &user.hashed_password).unwrap() {
//             // cookies.add_private(Cookie::new("user_id", user.id.unwrap().to_string()));
//             // cookies.add_private(Cookie::new("user_id", user.id.unwrap_or(0).to_string()));
//             if let Some(user_id) = user.id {
//                 cookies.add_private(Cookie::new("user_id", user_id.to_string()));
//             }
//
//             return Redirect::to(uri!(super::messages::index));
//         }
//     }
//
//     Redirect::to(uri!(login_form))
// }

#[post("/login", data = "<form>")]
pub async fn login(form: Form<LoginForm>, conn: DbConn, cookies: &CookieJar<'_>) -> Redirect {
    let form_data = form.into_inner();

    let user_result: Result<User, _> = conn.run(move |c| {
        users
            .filter(username.eq(&form_data.user_name))
            .first::<User>(c)
    }).await;

    match user_result {
        Ok(user) => {
            if verify(&form_data.password, &user.hashed_password).unwrap_or(false) {
                if let Some(user_id) = user.id {
                    cookies.add_private(Cookie::new("user_id", user_id.to_string()));
                }
                return Redirect::to(uri!(super::messages::index));
            }
        }
        Err(_) => {
            eprintln!("User not found or database error");
        }
    }
    Redirect::to(uri!(login_form))
}

#[get("/register")]
pub async fn register_form() -> Template {
    let mut context: HashMap<String, serde_json::Value> = HashMap::new();
    Template::render("register", &context)
}

#[get("/login")]
pub async fn login_form() -> Template {
    let mut context: HashMap<String, serde_json::Value> = HashMap::new();
    Template::render("login", &context)
}
