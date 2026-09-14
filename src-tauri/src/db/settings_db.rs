use rusqlite::params;
use super::DbState;

#[tauri::command]
pub fn get_db_setting(state: tauri::State<'_, DbState>, key: String) -> Result<Option<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query_map(params![key], |row| row.get(0)).map_err(|e| e.to_string())?;
    if let Some(Ok(val)) = rows.next() {
        Ok(Some(val))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn set_db_setting(
    state: tauri::State<'_, DbState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(|e| format!("Failed saving setting: {}", e))?;
    Ok(())
}
