use rusqlite::Connection;

pub fn init_db() -> Result<Connection, String> {
    let app_dir = super::super::downloader::paths::get_app_dir();
    let db_path = app_dir.join("ollama_lite.db");

    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open SQLite db: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed creating sessions table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| format!("Failed creating messages table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed creating settings table: {}", e))?;

    Ok(conn)
}
