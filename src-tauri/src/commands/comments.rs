use crate::db::Db;
use crate::error::AppResult;
use crate::models::Comment;
use crate::repositories::comments_repository;
use tauri::State;

#[tauri::command]
pub fn list_comments(db: State<Db>, owner_type: String, owner_id: String) -> AppResult<Vec<Comment>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    comments_repository::list_for_owner(&conn, &owner_type, &owner_id)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_comment(
    db: State<Db>,
    project_id: String,
    owner_type: String,
    owner_id: String,
    anchor_start: Option<i64>,
    anchor_end: Option<i64>,
    quote: Option<String>,
    body: String,
) -> AppResult<Comment> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    comments_repository::create(
        &conn,
        &project_id,
        &owner_type,
        &owner_id,
        anchor_start,
        anchor_end,
        quote.as_deref(),
        &body,
    )
}

#[tauri::command]
pub fn set_comment_resolved(db: State<Db>, id: String, resolved: bool) -> AppResult<Comment> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    comments_repository::set_resolved(&conn, &id, resolved)
}

#[tauri::command]
pub fn delete_comment(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    comments_repository::delete(&conn, &id)
}
