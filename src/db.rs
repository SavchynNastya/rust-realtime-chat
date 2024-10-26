// use diesel::prelude::*;
// use diesel::sqlite::SqliteConnection;
// use rocket_sync_db_pools::database;
use rocket_sync_db_pools::{database, diesel};

#[database("sqlite_db")]
pub struct DbConn(diesel::SqliteConnection);
// #[database("sqlite_database")]
// pub struct DbConn(diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>);
