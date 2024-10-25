// use diesel::RunQueryDsl;
// use rocket::{Data, fairing::{Fairing, Info, Kind}, Request};
// use rocket::serde::json::Json;
//
// use crate::db::DbConn;
// use crate::diesel::ExpressionMethods;
// use crate::diesel::QueryDsl;
// use crate::models::User;
//
// pub struct AuthFairing;
//
// pub async fn fetch_user_by_id(db_conn: &DbConn, user_id: i32) -> Result<Json<User>, diesel::result::Error> {
//     use crate::schema::users::dsl::*;
//
//     let result = db_conn.run(move |conn| {
//         users.filter(id.eq(user_id)).first::<User>(conn)
//     }).await;
//
//     result.map(Json)
// }
//
// #[rocket::async_trait]
// impl Fairing for AuthFairing {
//     fn info(&self) -> Info {
//         Info {
//             name: "User Authentication and Fetch",
//             kind: Kind::Request,
//         }
//     }
//
//     async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
//         let user_id_cookie = request.cookies().get_private("user_id");
//
//         if let Some(user_id) = user_id_cookie.and_then(|c| c.value().parse::<i32>().ok()) {
//             // Fetch the user from the database using the shared DbConn state
//             let db_conn = request.rocket().state::<DbConn>().expect("DbConn not found");
//
//             // Assume fetch_user_by_id is modified to accept a &State<DbConn>
//             if let Ok(user) = fetch_user_by_id(db_conn, user_id).await {
//                 request.local_cache(|| Some(user));  // Cache the user for this request
//             } else {
//                 request.local_cache(|| None::<User>);  // Cache `None` if user is not found
//             }
//         } else {
//             request.local_cache(|| None::<User>);  // Cache `None` if not authenticated
//         }
//     }
// }
