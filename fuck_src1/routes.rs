// use rocket::State;
// use rocket::serde::json::Json;
// use crate::db::DbConn;
// use crate::models::{Message, NewMessage};
// use rocket::tokio::sync::broadcast::{self, Sender};
//
// #[get("/")]
// pub fn index() -> &'templates str {
//     "Welcome to the Chat!"
// }
//
// #[post("/send", data = "<message>")]
// pub async fn send_message(message: Json<NewMessage>, conn: DbConn, state: &State<Sender<Message>>) -> Json<Message> {
//     let msg = Message {
//         id: 0,
//         user_id: message.user_id,
//         content: message.content.clone(),
//         created_at: chrono::Utc::now().naive_utc(),
//     };
//
//     let result = conn.run(|c| diesel::insert_into(messages).values(&msg).execute(c)).await;
//     if result.is_err() {
//         return Json(msg);
//     }
//
//     state.send(msg.clone()).unwrap();
//     Json(msg)
// }
