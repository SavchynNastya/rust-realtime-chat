use rocket::request::{self, FromRequest, Request};
use rocket::outcome::Outcome;
use crate::db::DbConn;
use crate::models::User;
use crate::schema::users::dsl::*;
use diesel::prelude::*;

pub struct AuthenticatedUser {
    pub user: Option<User>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let cookies = request.cookies();
        let db = request.guard::<DbConn>().await.unwrap();

        if let Some(user_cookie) = cookies.get_private("user_id") {
            let user_id: i32 = user_cookie.value().parse().unwrap_or_default();

            let user = db
                .run(move |c| users.filter(id.eq(user_id)).first::<User>(c).optional())
                .await
                .ok()
                .flatten();

            return Outcome::Success(AuthenticatedUser { user });
        }

        Outcome::Success(AuthenticatedUser { user: None })
    }
}
