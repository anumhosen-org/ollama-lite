use rusqlite::params;
use serde::{Deserialize, Serialize};
use super::DbState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

pub fn now_iso() -> String {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[tauri::command]
pub fn create_chat_session(
    state: tauri::State<'_, DbState>,
    title: String,
) -> Result<ChatSession, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_iso();

    conn.execute(
        "INSERT INTO sessions (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, title, now, now],
    )
    .map_err(|e| format!("Failed inserting session: {}", e))?;

    Ok(ChatSession {
        id,
        title,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn get_chat_sessions(state: tauri::State<'_, DbState>) -> Result<Vec<ChatSession>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, title, created_at, updated_at FROM sessions ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ChatSession {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut sessions = Vec::new();
    for r in rows {
        if let Ok(s) = r {
            sessions.push(s);
        }
    }
    Ok(sessions)
}

#[tauri::command]
pub fn delete_chat_session(state: tauri::State<'_, DbState>, session_id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let _ = conn.execute("DELETE FROM messages WHERE session_id = ?1", params![session_id]);
    conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])
        .map_err(|e| format!("Failed deleting session: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn update_chat_session_title(
    state: tauri::State<'_, DbState>,
    session_id: String,
    title: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE sessions SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now_iso(), session_id],
    )
    .map_err(|e| format!("Failed updating session title: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn save_chat_message(
    state: tauri::State<'_, DbState>,
    session_id: String,
    role: String,
    content: String,
) -> Result<DbMessage, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_iso();

    conn.execute(
        "INSERT INTO messages (id, session_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, session_id, role, content, now],
    )
    .map_err(|e| format!("Failed inserting message: {}", e))?;

    let _ = conn.execute(
        "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
        params![now, session_id],
    );

    Ok(DbMessage {
        id,
        session_id,
        role,
        content,
        timestamp: now,
    })
}

#[tauri::command]
pub fn get_session_messages(
    state: tauri::State<'_, DbState>,
    session_id: String,
) -> Result<Vec<DbMessage>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, session_id, role, content, timestamp FROM messages WHERE session_id = ?1 ORDER BY timestamp ASC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![session_id], |row| {
            Ok(DbMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for r in rows {
        if let Ok(m) = r {
            list.push(m);
        }
    }
    Ok(list)
}
