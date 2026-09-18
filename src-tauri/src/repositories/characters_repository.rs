use crate::error::AppResult;
use crate::models::Character;
use rusqlite::{named_params, Connection, OptionalExtension};

const COLUMNS: &str = "id, project_id, name, reading, aliases, age, gender, birthday, height,
    occupation, affiliation, role, first_appearance, hair, eyes, build, clothing, features,
    scars, equipment, personality, strengths, weaknesses, beliefs, desires, fears, secret,
    trauma, first_person, second_person, speech_suffix, catchphrase, honorific_level,
    calls_protagonist, calls_others, goal, motivation, past, initial_state, middle_state,
    final_state, character_arc, memo, reference_image_path";

fn row_to_character(row: &rusqlite::Row) -> rusqlite::Result<Character> {
    Ok(Character {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        name: row.get("name")?,
        reading: row.get("reading")?,
        aliases: row.get("aliases")?,
        age: row.get("age")?,
        gender: row.get("gender")?,
        birthday: row.get("birthday")?,
        height: row.get("height")?,
        occupation: row.get("occupation")?,
        affiliation: row.get("affiliation")?,
        role: row.get("role")?,
        first_appearance: row.get("first_appearance")?,
        hair: row.get("hair")?,
        eyes: row.get("eyes")?,
        build: row.get("build")?,
        clothing: row.get("clothing")?,
        features: row.get("features")?,
        scars: row.get("scars")?,
        equipment: row.get("equipment")?,
        personality: row.get("personality")?,
        strengths: row.get("strengths")?,
        weaknesses: row.get("weaknesses")?,
        beliefs: row.get("beliefs")?,
        desires: row.get("desires")?,
        fears: row.get("fears")?,
        secret: row.get("secret")?,
        trauma: row.get("trauma")?,
        first_person: row.get("first_person")?,
        second_person: row.get("second_person")?,
        speech_suffix: row.get("speech_suffix")?,
        catchphrase: row.get("catchphrase")?,
        honorific_level: row.get("honorific_level")?,
        calls_protagonist: row.get("calls_protagonist")?,
        calls_others: row.get("calls_others")?,
        goal: row.get("goal")?,
        motivation: row.get("motivation")?,
        past: row.get("past")?,
        initial_state: row.get("initial_state")?,
        middle_state: row.get("middle_state")?,
        final_state: row.get("final_state")?,
        character_arc: row.get("character_arc")?,
        memo: row.get("memo")?,
        reference_image_path: row.get("reference_image_path")?,
    })
}

pub fn create(conn: &Connection, project_id: &str, c: &Character) -> AppResult<Character> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO characters (id, project_id, name, reading, aliases, age, gender, birthday,
            height, occupation, affiliation, role, first_appearance, hair, eyes, build, clothing,
            features, scars, equipment, personality, strengths, weaknesses, beliefs, desires,
            fears, secret, trauma, first_person, second_person, speech_suffix, catchphrase,
            honorific_level, calls_protagonist, calls_others, goal, motivation, past,
            initial_state, middle_state, final_state, character_arc, memo, reference_image_path,
            created_at, updated_at)
         VALUES (:id, :project_id, :name, :reading, :aliases, :age, :gender, :birthday, :height,
            :occupation, :affiliation, :role, :first_appearance, :hair, :eyes, :build, :clothing,
            :features, :scars, :equipment, :personality, :strengths, :weaknesses, :beliefs,
            :desires, :fears, :secret, :trauma, :first_person, :second_person, :speech_suffix,
            :catchphrase, :honorific_level, :calls_protagonist, :calls_others, :goal, :motivation,
            :past, :initial_state, :middle_state, :final_state, :character_arc, :memo,
            :reference_image_path, :created_at, :updated_at)",
        named_params! {
            ":id": id, ":project_id": project_id, ":name": c.name, ":reading": c.reading,
            ":aliases": c.aliases, ":age": c.age, ":gender": c.gender, ":birthday": c.birthday,
            ":height": c.height, ":occupation": c.occupation, ":affiliation": c.affiliation,
            ":role": c.role, ":first_appearance": c.first_appearance, ":hair": c.hair,
            ":eyes": c.eyes, ":build": c.build, ":clothing": c.clothing, ":features": c.features,
            ":scars": c.scars, ":equipment": c.equipment, ":personality": c.personality,
            ":strengths": c.strengths, ":weaknesses": c.weaknesses, ":beliefs": c.beliefs,
            ":desires": c.desires, ":fears": c.fears, ":secret": c.secret, ":trauma": c.trauma,
            ":first_person": c.first_person, ":second_person": c.second_person,
            ":speech_suffix": c.speech_suffix, ":catchphrase": c.catchphrase,
            ":honorific_level": c.honorific_level, ":calls_protagonist": c.calls_protagonist,
            ":calls_others": c.calls_others, ":goal": c.goal, ":motivation": c.motivation,
            ":past": c.past, ":initial_state": c.initial_state, ":middle_state": c.middle_state,
            ":final_state": c.final_state, ":character_arc": c.character_arc, ":memo": c.memo,
            ":reference_image_path": c.reference_image_path, ":created_at": now, ":updated_at": now,
        },
    )?;
    get(conn, &id)?.ok_or_else(|| crate::error::AppError::Other("failed to reload created character".into()))
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<Character>> {
    let sql = format!("SELECT {COLUMNS} FROM characters WHERE id = ?1 AND deleted_at IS NULL");
    Ok(conn.query_row(&sql, [id], row_to_character).optional()?)
}

pub fn list_by_project(conn: &Connection, project_id: &str) -> AppResult<Vec<Character>> {
    let sql = format!("SELECT {COLUMNS} FROM characters WHERE project_id = ?1 AND deleted_at IS NULL ORDER BY name");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([project_id], row_to_character)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn update(conn: &Connection, id: &str, c: &Character) -> AppResult<Character> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE characters SET name=:name, reading=:reading, aliases=:aliases, age=:age,
            gender=:gender, birthday=:birthday, height=:height, occupation=:occupation,
            affiliation=:affiliation, role=:role, first_appearance=:first_appearance, hair=:hair,
            eyes=:eyes, build=:build, clothing=:clothing, features=:features, scars=:scars,
            equipment=:equipment, personality=:personality, strengths=:strengths,
            weaknesses=:weaknesses, beliefs=:beliefs, desires=:desires, fears=:fears,
            secret=:secret, trauma=:trauma, first_person=:first_person,
            second_person=:second_person, speech_suffix=:speech_suffix, catchphrase=:catchphrase,
            honorific_level=:honorific_level, calls_protagonist=:calls_protagonist,
            calls_others=:calls_others, goal=:goal, motivation=:motivation, past=:past,
            initial_state=:initial_state, middle_state=:middle_state, final_state=:final_state,
            character_arc=:character_arc, memo=:memo, reference_image_path=:reference_image_path,
            updated_at=:updated_at
         WHERE id=:id AND deleted_at IS NULL",
        named_params! {
            ":id": id, ":name": c.name, ":reading": c.reading, ":aliases": c.aliases,
            ":age": c.age, ":gender": c.gender, ":birthday": c.birthday, ":height": c.height,
            ":occupation": c.occupation, ":affiliation": c.affiliation, ":role": c.role,
            ":first_appearance": c.first_appearance, ":hair": c.hair, ":eyes": c.eyes,
            ":build": c.build, ":clothing": c.clothing, ":features": c.features,
            ":scars": c.scars, ":equipment": c.equipment, ":personality": c.personality,
            ":strengths": c.strengths, ":weaknesses": c.weaknesses, ":beliefs": c.beliefs,
            ":desires": c.desires, ":fears": c.fears, ":secret": c.secret, ":trauma": c.trauma,
            ":first_person": c.first_person, ":second_person": c.second_person,
            ":speech_suffix": c.speech_suffix, ":catchphrase": c.catchphrase,
            ":honorific_level": c.honorific_level, ":calls_protagonist": c.calls_protagonist,
            ":calls_others": c.calls_others, ":goal": c.goal, ":motivation": c.motivation,
            ":past": c.past, ":initial_state": c.initial_state, ":middle_state": c.middle_state,
            ":final_state": c.final_state, ":character_arc": c.character_arc, ":memo": c.memo,
            ":reference_image_path": c.reference_image_path, ":updated_at": now,
        },
    )?;
    get(conn, id)?.ok_or_else(|| crate::error::AppError::NotFound(format!("character {id}")))
}

pub fn soft_delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE characters SET deleted_at = ?2 WHERE id = ?1",
        rusqlite::params![id, chrono::Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectInput;
    use crate::repositories::projects_repository;

    #[test]
    fn create_get_update_list_delete_roundtrip() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();
        let project = projects_repository::create(&conn, &ProjectInput { title: "P".into(), ..Default::default() }, false).unwrap();

        let input = Character {
            name: "アリス".into(),
            role: Some("主人公".into()),
            first_person: Some("わたし".into()),
            ..Default::default()
        };
        let created = create(&conn, &project.id, &input).unwrap();
        assert_eq!(created.name, "アリス");
        assert_eq!(created.role.as_deref(), Some("主人公"));

        let mut updated_input = created.clone();
        updated_input.age = Some("17".into());
        let updated = update(&conn, &created.id, &updated_input).unwrap();
        assert_eq!(updated.age.as_deref(), Some("17"));

        assert_eq!(list_by_project(&conn, &project.id).unwrap().len(), 1);

        soft_delete(&conn, &created.id).unwrap();
        assert!(get(&conn, &created.id).unwrap().is_none());
    }
}
