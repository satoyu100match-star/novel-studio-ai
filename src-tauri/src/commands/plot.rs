use crate::db::Db;
use crate::error::AppResult;
use crate::models::{PlotCard, PlotLane};
use crate::repositories::plot_repository;
use tauri::State;

#[tauri::command]
pub fn list_plot_lanes(db: State<Db>, project_id: String) -> AppResult<Vec<PlotLane>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    plot_repository::list_lanes(&conn, &project_id)
}

#[tauri::command]
pub fn create_plot_lane(db: State<Db>, project_id: String, name: String) -> AppResult<PlotLane> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    plot_repository::create_lane(&conn, &project_id, &name)
}

#[tauri::command]
pub fn rename_plot_lane(db: State<Db>, id: String, name: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    plot_repository::rename_lane(&conn, &id, &name)
}

#[tauri::command]
pub fn delete_plot_lane(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    plot_repository::delete_lane(&conn, &id)
}

#[tauri::command]
pub fn list_plot_cards(db: State<Db>, project_id: String) -> AppResult<Vec<PlotCard>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    plot_repository::list_cards(&conn, &project_id)
}

#[tauri::command]
pub fn create_plot_card(db: State<Db>, project_id: String, lane_id: String, title: String) -> AppResult<PlotCard> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    plot_repository::create_card(&conn, &project_id, &lane_id, &title)
}

#[tauri::command]
pub fn update_plot_card(db: State<Db>, id: String, input: PlotCard) -> AppResult<PlotCard> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    plot_repository::update_card(&conn, &id, &input)
}

#[tauri::command]
pub fn move_plot_card(db: State<Db>, id: String, lane_id: String, order_index: i64) -> AppResult<PlotCard> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    plot_repository::move_card(&conn, &id, &lane_id, order_index)
}

#[tauri::command]
pub fn delete_plot_card(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    plot_repository::delete_card(&conn, &id)
}
