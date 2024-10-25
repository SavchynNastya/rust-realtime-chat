use rocket_sync_db_pools::database;

#[database("chat_db")]
pub struct ChatDb(diesel::SqliteConnection);