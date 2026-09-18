use crate::db::Db;
use crate::error::AppResult;
use crate::models::{Character, CharacterRelation};
use crate::repositories::{character_relations_repository, characters_repository, tags_repository};
use tauri::State;

#[tauri::command]
pub fn create_character(db: State<Db>, project_id: String, input: Character) -> AppResult<Character> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    characters_repository::create(&conn, &project_id, &input)
}

#[tauri::command]
pub fn list_characters(db: State<Db>, project_id: String) -> AppResult<Vec<Character>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    characters_repository::list_by_project(&conn, &project_id)
}

#[tauri::command]
pub fn get_character(db: State<Db>, id: String) -> AppResult<Option<Character>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    characters_repository::get(&conn, &id)
}

#[tauri::command]
pub fn update_character(db: State<Db>, id: String, input: Character) -> AppResult<Character> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    characters_repository::update(&conn, &id, &input)
}

#[tauri::command]
pub fn delete_character(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    characters_repository::soft_delete(&conn, &id)
}

#[tauri::command]
pub fn set_character_tags(db: State<Db>, project_id: String, character_id: String, tags: Vec<String>) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    tags_repository::set_entity_tags(&conn, &project_id, "character", &character_id, &tags)
}

#[tauri::command]
pub fn get_character_tags(db: State<Db>, character_id: String) -> AppResult<Vec<String>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    tags_repository::get_entity_tags(&conn, "character", &character_id)
}

#[tauri::command]
pub fn create_character_relation(
    db: State<Db>,
    project_id: String,
    from_character_id: String,
    to_character_id: String,
    label: String,
    direction: String,
) -> AppResult<CharacterRelation> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    character_relations_repository::create(
        &conn, &project_id, &from_character_id, &to_character_id, &label, &direction, None, None,
    )
}

#[tauri::command]
pub fn list_character_relations(db: State<Db>, project_id: String) -> AppResult<Vec<CharacterRelation>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    character_relations_repository::list_by_project(&conn, &project_id)
}

#[tauri::command]
pub fn delete_character_relation(db: State<Db>, id: String) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    character_relations_repository::soft_delete(&conn, &id)
}
