//! Phase 10: サンプル作品。初めてこのアプリを開いた人が、原稿を1文字も
//! 書かなくても「人物/世界観/プロット/時系列/伏線/TODO/原稿」がひと通り
//! 埋まった状態のプロジェクトをすぐ触って全機能を試せるようにする
//! (仕様のOnboarding要件、CLAUDE.md「ダミーUIを避ける」の精神に沿い、
//! 空のフォームではなく実際に読める文章・実データを入れる)。
//!
//! `projects_repository::create`の`is_sample`フラグは実はPhase1から
//! 存在していたが、これまで実際にサンプルを生成する経路が無かった。
//! このモジュールがその経路を提供する。生成されたプロジェクトは通常の
//! プロジェクトと全く同じ機能で編集・削除できる(特別扱いはしない)。

use crate::error::AppResult;
use crate::models::{
    Character, Foreshadowing, GlossaryEntry, Location, Project, ProjectInput, TimelineEvent, WorldEntry,
};
use crate::repositories::{
    characters_repository, chapters_repository, documents_repository, foreshadowing_repository, glossary_repository,
    locations_repository, plot_repository, projects_repository, scenes_repository, timeline_repository,
    todos_repository, world_repository,
};
use rusqlite::Connection;

pub fn create(conn: &Connection) -> AppResult<Project> {
    let project = projects_repository::create(
        conn,
        &ProjectInput {
            title: "灯火の図書館".to_string(),
            genre: Some("ファンタジー".to_string()),
            synopsis: Some(
                "小さな街の図書館で働く陽向は、閉架書庫の奥で誰も読めない本を見つける。\n\
                 その夜、本の持ち主だという青年レインが図書館に現れたことで、陽向の平穏な日常は少しずつ変わっていく。"
                    .to_string(),
            ),
            theme: Some("平凡な日常に紛れ込む小さな非日常、言葉の持つ力".to_string()),
            pov_policy: Some("三人称単視点(陽向視点)".to_string()),
            tense: Some("過去形".to_string()),
            ..Default::default()
        },
        true,
    )?;
    let project_id = project.id.clone();

    seed_manuscript(conn, &project_id)?;
    seed_characters(conn, &project_id)?;
    seed_world(conn, &project_id)?;
    seed_plot_and_schedule(conn, &project_id)?;

    Ok(project)
}

fn seed_manuscript(conn: &Connection, project_id: &str) -> AppResult<String> {
    let chapter = chapters_repository::create(conn, project_id, None, "第一章 灯火の下で")?;

    let scene1 = scenes_repository::create(conn, &chapter.id, "静かな図書館")?;
    documents_repository::save_body(
        conn,
        "scene",
        &scene1.id,
        "灯火市中央図書館の朝は、埃と古い紙の匂いから始まる。\n\n\
         陽向は開館前の一時間を、いつも閉架書庫の整理にあてていた。誰にも読まれないまま棚の奥で眠っている本たちに、せめて背表紙だけでも陽の光を当ててやりたかったからだ。\n\n\
         その日、いつもの棚の隙間に、見覚えのない一冊を見つけた。\n\n\
         「……こんな本、うちにあったかな」\n\n\
         革の表紙には文字がない。開いてみても、そこにあるのは陽向の知らない記号の羅列だった。読めないはずなのに、指先でなぞると、頭の奥でかすかに声が響いた気がした。",
    )?;

    let scene2 = scenes_repository::create(conn, &chapter.id, "見知らぬ来訪者")?;
    documents_repository::save_body(
        conn,
        "scene",
        &scene2.id,
        "閉館時刻を過ぎても、陽向はその本を手放せずにいた。\n\n\
         カウンターの明かりを落とそうとしたとき、扉の鈴が鳴った。\n\n\
         「すみません、閉館は……」\n\n\
         顔を上げると、そこには見知らぬ青年が立っていた。黒いコートの裾が、風もないのに揺れている。\n\n\
         「その本、僕のなんです」\n\n\
         青年は静かにそう言うと、まっすぐに陽向の手元を見つめた。\n\n\
         「レインといいます。少し、話を聞いてもらえますか」\n\n\
         図書館の静けさの中で、陽向はまだ、この出会いが何を意味するのか知らなかった。",
    )?;

    Ok(chapter.id)
}

fn seed_characters(conn: &Connection, project_id: &str) -> AppResult<()> {
    characters_repository::create(
        conn,
        project_id,
        &Character {
            name: "陽向(ひなた)".to_string(),
            reading: Some("ひなた".to_string()),
            age: Some("22歳".to_string()),
            occupation: Some("図書館司書".to_string()),
            role: Some("主人公".to_string()),
            personality: Some("穏やかで観察力があるが、一度気になると後を引く性格。".to_string()),
            first_person: Some("わたし".to_string()),
            goal: Some("誰にも読まれない本たちに、もう一度光を当てること。".to_string()),
            ..Default::default()
        },
    )?;

    characters_repository::create(
        conn,
        project_id,
        &Character {
            name: "レイン".to_string(),
            age: Some("不明(見た目は20代前半)".to_string()),
            role: Some("ヒロイン/謎の来訪者".to_string()),
            personality: Some("物腰は穏やかだが、核心には決して触れさせない。".to_string()),
            first_person: Some("僕".to_string()),
            secret: Some("「言霊術」の最後の使い手であること。本人はまだ語っていない。".to_string()),
            ..Default::default()
        },
    )?;

    Ok(())
}

fn seed_world(conn: &Connection, project_id: &str) -> AppResult<()> {
    world_repository::ensure_builtin_categories(conn, project_id)?;
    let categories = world_repository::list_categories(conn, project_id)?;
    let city_category = categories.iter().find(|c| c.name == "都市");
    let magic_category = categories.iter().find(|c| c.name == "魔法");

    world_repository::create_entry(
        conn,
        project_id,
        &WorldEntry {
            category_id: city_category.map(|c| c.id.clone()),
            name: "灯火市(とうかし)".to_string(),
            summary: Some("物語の舞台となる、古い港街。".to_string()),
            detail: Some("石畳の坂道と、時代の異なる建物が入り混じる街並みが特徴。中央図書館は街の中心にある。".to_string()),
            ..Default::default()
        },
    )?;

    world_repository::create_entry(
        conn,
        project_id,
        &WorldEntry {
            category_id: magic_category.map(|c| c.id.clone()),
            name: "言霊術(ことだまじゅつ)".to_string(),
            summary: Some("文字に込めた言葉が力を持つ、この世界に伝わる古い術。".to_string()),
            detail: Some("使い手は既にほとんど残っていないとされる。専用の紙(言霊札)に文字を刻むことで発動する。".to_string()),
            ..Default::default()
        },
    )?;

    locations_repository::create(
        conn,
        project_id,
        &Location {
            name: "灯火市中央図書館".to_string(),
            description: Some("陽向が働く図書館。閉架書庫には古い蔵書が多く眠っている。".to_string()),
            ..Default::default()
        },
    )?;

    glossary_repository::create(
        conn,
        project_id,
        &GlossaryEntry {
            term: "言霊札(ことだまふだ)".to_string(),
            reading: Some("ことだまふだ".to_string()),
            definition: Some("言霊術を行使するための専用の札。作中の重要アイテム。".to_string()),
            ..Default::default()
        },
    )?;

    Ok(())
}

fn seed_plot_and_schedule(conn: &Connection, project_id: &str) -> AppResult<()> {
    let chapter_id = chapters_repository::list_by_project(conn, project_id)?
        .into_iter()
        .next()
        .map(|c| c.id);

    plot_repository::ensure_default_lanes(conn, project_id)?;
    let lanes = plot_repository::list_lanes(conn, project_id)?;
    if let Some(first_lane) = lanes.first() {
        let card1 = plot_repository::create_card(conn, project_id, &first_lane.id, "第一章: 出会い")?;
        if let Some(cid) = &chapter_id {
            let mut updated = card1.clone();
            updated.chapter_id = Some(cid.clone());
            updated.summary = Some("陽向が謎の本を見つけ、レインと出会う。".to_string());
            plot_repository::update_card(conn, &card1.id, &updated)?;
        }
        plot_repository::create_card(conn, project_id, &first_lane.id, "第二章: 言霊術の秘密(仮)")?;
    }

    let event = timeline_repository::create(conn, project_id, "陽向とレインが出会う")?;
    let mut event_update = TimelineEvent { chapter_id: chapter_id.clone(), ..event.clone() };
    event_update.description = Some("灯火市暦・晩秋のある日。図書館の閉館後。".to_string());
    timeline_repository::update(conn, &event.id, &event_update)?;

    let foreshadowing = foreshadowing_repository::create(conn, project_id, "レインの正体")?;
    let mut f_update = Foreshadowing { planted_chapter_id: chapter_id, ..foreshadowing.clone() };
    f_update.status = "planted".to_string();
    f_update.detail = Some("レインが「言霊術の最後の使い手」であることは、まだ本人の口から語られていない。".to_string());
    foreshadowing_repository::update(conn, &foreshadowing.id, &f_update)?;

    todos_repository::create(conn, project_id, "第二章のプロットを詰める")?;
    todos_repository::create(conn, project_id, "レインの過去エピソードを考える")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_fully_populated_sample_project() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::run(&mut conn).unwrap();

        let project = create(&conn).unwrap();
        assert!(project.is_sample);
        assert!(!project.title.is_empty());

        let chapters = chapters_repository::list_by_project(&conn, &project.id).unwrap();
        assert_eq!(chapters.len(), 1);
        let scenes = scenes_repository::list_by_chapter(&conn, &chapters[0].id).unwrap();
        assert_eq!(scenes.len(), 2);
        for scene in &scenes {
            let doc = documents_repository::get(&conn, "scene", &scene.id).unwrap().unwrap();
            assert!(!doc.body.is_empty(), "sample scene body must not be empty");
        }

        let characters = characters_repository::list_by_project(&conn, &project.id).unwrap();
        assert_eq!(characters.len(), 2);

        let world_entries = world_repository::list_entries(&conn, &project.id).unwrap();
        assert_eq!(world_entries.len(), 2);
        let locations = locations_repository::list_by_project(&conn, &project.id).unwrap();
        assert_eq!(locations.len(), 1);
        let glossary = glossary_repository::list_by_project(&conn, &project.id).unwrap();
        assert_eq!(glossary.len(), 1);

        let cards = plot_repository::list_cards(&conn, &project.id).unwrap();
        assert_eq!(cards.len(), 2);
        assert!(cards.iter().any(|c| c.chapter_id == Some(chapters[0].id.clone())));

        let events = timeline_repository::list_by_project(&conn, &project.id).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].chapter_id, Some(chapters[0].id.clone()));

        let foreshadowings = foreshadowing_repository::list_by_project(&conn, &project.id).unwrap();
        assert_eq!(foreshadowings.len(), 1);
        assert_eq!(foreshadowings[0].status, "planted");

        let todos = todos_repository::list_by_project(&conn, &project.id).unwrap();
        assert_eq!(todos.len(), 2);
    }
}
