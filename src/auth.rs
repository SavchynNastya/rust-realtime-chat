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

#[post("/register", data = "<form>")]
pub async fn register(form: Form<RegisterForm>, conn: DbConn) -> Redirect {
    let new_user = NewUser {
        username: form.user_name.clone(),
        hashed_password: hash(&form.password, DEFAULT_COST).unwrap(),
    };

    let result = conn.run(move |c| {
        use diesel::prelude::*;

        diesel::insert_into(users)
            .values(&new_user)
            .execute(c)
    }).await;

    Redirect::to(uri!(login_form))
}

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
