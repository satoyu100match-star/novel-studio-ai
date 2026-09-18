use crate::db::Db;
use crate::error::AppResult;
use crate::models::Todo;
use crate::repositories::todos_repository;
use tauri::State;

#[tauri::command]
pub fn list_todos(db: State<Db>, project_id: String) -> AppResult<Vec<Todo>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    todos_repository::list_by_project(&conn, &project_id)
}

#[tauri::command]
pub fn create_todo(db: State<Db>, project_id: String, title: String) -> AppResult<Todo> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    todos_repository::create(&conn, &project_id, &title)
}

#[tauri::command]
pub fn update_todo(db: State<Db>, id: String, input: Todo) -> AppResult<Todo> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    todos_repository::update(&conn, &id, &input)
}

#[tauri::command]
pub fn set_todo_done(db: State<Db>, id: String, done: bool) -> AppResult<Todo> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    todos_repository::set_done(&conn, &id, done)
}

#[tauri::command]
pub fn delete_todo(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    todos_repository::delete(&conn, &id)
}
