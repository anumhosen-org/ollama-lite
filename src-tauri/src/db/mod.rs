pub mod chat_db;
pub mod schema;
pub mod settings_db;

use rusqlite::Connection;
use std::sync::Mutex;

pub struct DbState {
    pub db: Mutex<Connection>,
}

pub use chat_db::*;
pub use schema::*;
pub use settings_db::*;
