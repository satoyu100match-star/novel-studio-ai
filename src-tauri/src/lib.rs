//! Novel Studio AI -- Tauri backend entry point.
//!
//! Phase 0 scope only: open/migrate the local SQLite database and expose
//! a minimal settings round-trip so the frontend can prove read/write
//! works end to end. Domain features (projects, chapters, characters,
//! ...) are added from Phase 1 onward -- see docs/ROADMAP.md.

mod ai;
mod commands;
mod db;
mod error;
mod export;
mod models;
mod readability;
mod repositories;
mod sample_project;

use ai::keystore::AiKeyStore;
use commands::backup::UncleanShutdownFlag;
use db::Db;
use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind};

/// クリーンシャットダウンを記録する`app_settings`のキー。起動時に前回値を
/// 見てから即座に"1"へ上書きし、アプリが正常終了できた時だけ`RunEvent::
/// Exit`で"0"に戻す。次回起動時に"1"のままだったら前回は正常終了しな
/// かったと判断できる(Phase8 Crash Recovery、CLAUDE.md参照)。
const SESSION_OPEN_MARKER_KEY: &str = "app.session_open";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    // Phase15: Auto Update。tauri-plugin-updaterはデスクトップ専用
    // (モバイルには存在しない)ため、Tauri公式の例に倣い`cfg(desktop)`で
    // 分岐する。実際の更新確認・ダウンロード・適用はフロント側
    // (`@tauri-apps/plugin-updater`)から明示的なユーザー操作で呼び出す
    // 設計とし、ここではプラグインの登録のみ行う(バックグラウンドで
    // 勝手に更新を当てることはしない -- CLAUDE.md「データ安全性」優先の
    // 方針と、勝手に再起動されると執筆中の内容を見失う不安につながる
    // ことを避けるため)。`tauri-plugin-process`は更新適用後の再起動
    // (`relaunch()`)に使う。
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build()).plugin(tauri_plugin_process::init());

    builder
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                // Log to both a rotating file (for crash / bug reports)
                // and stdout (for `pnpm tauri dev`). Never log manuscript
                // text or API keys -- see CLAUDE.md "禁止事項" and
                // docs/AI.md.
                .targets([
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Stdout),
                ])
                .build(),
        )
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("resolvable app data dir");
            let db_path = app_data_dir.join("novel-studio-ai.db");

            log::info!("opening database at {}", db_path.display());
            let db = Db::open(&db_path).expect("failed to open/migrate database");

            // Phase 8: Crash Recovery -- 前回のセッションマーカーが立った
            // ままなら、前回は正常終了できなかった可能性が高い(強制終了/
            // クラッシュ等)。実データ自体は保存のたびにSQLiteへ同期書き
            // 込みされているため喪失の心配はないが、フロント側で一度だけ
            // 案内バナーを出せるようフラグを渡す。
            let was_unclean = {
                let conn = db.conn.lock().expect("db mutex poisoned");
                let marker = repositories::settings_repository::get(&conn, SESSION_OPEN_MARKER_KEY)
                    .unwrap_or(None);
                let was_unclean = marker.as_deref() == Some("1");
                if let Err(e) = repositories::settings_repository::set(&conn, SESSION_OPEN_MARKER_KEY, "1") {
                    log::warn!("failed to write session marker: {e}");
                }
                was_unclean
            };
            app.manage(UncleanShutdownFlag(was_unclean));

            // Phase 8: Auto Backup -- 起動のたびにDB全体のスナップショットを
            // 1件作成する(直近MAX_BACKUPS件のみ保持)。失敗しても起動は
            // 継続する(バックアップはあくまで保険であり、これ自体が理由で
            // 執筆できなくなってはならない -- CLAUDE.md優先順位#1「データ
            // 安全性」は基本機能の可用性とセットで考える)。
            if let Err(e) = db.backup_now() {
                log::warn!("startup backup failed: {e}");
            }

            app.manage(db);
            // Phase 5: AIキーはディスクへ永続化せず、アプリ実行中のみ
            // メモリ上に保持する(ai::keystore参照。keyringクレートを
            // 導入できる環境になり次第、OS資格情報ストア経由に切り替える)。
            app.manage(AiKeyStore::new());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info::app_info,
            commands::settings::get_setting,
            commands::settings::set_setting,
            commands::settings::list_settings,
            commands::projects::create_project,
            commands::projects::create_sample_project,
            commands::projects::list_projects,
            commands::projects::get_project,
            commands::projects::update_project,
            commands::projects::delete_project,
            commands::chapters::create_part,
            commands::chapters::list_parts,
            commands::chapters::rename_part,
            commands::chapters::delete_part,
            commands::chapters::create_chapter,
            commands::chapters::list_chapters,
            commands::chapters::get_chapter,
            commands::chapters::update_chapter_meta,
            commands::chapters::delete_chapter,
            commands::scenes::create_scene,
            commands::scenes::list_scenes,
            commands::scenes::get_scene,
            commands::scenes::rename_scene,
            commands::scenes::delete_scene,
            commands::documents::get_document,
            commands::documents::save_document_body,
            commands::documents::project_char_count,
            commands::search::search_project,
            commands::characters::create_character,
            commands::characters::list_characters,
            commands::characters::get_character,
            commands::characters::update_character,
            commands::characters::delete_character,
            commands::characters::set_character_tags,
            commands::characters::get_character_tags,
            commands::characters::create_character_relation,
            commands::characters::list_character_relations,
            commands::characters::delete_character_relation,
            commands::world::list_world_categories,
            commands::world::create_world_category,
            commands::world::create_world_entry,
            commands::world::list_world_entries,
            commands::world::get_world_entry,
            commands::world::update_world_entry,
            commands::world::delete_world_entry,
            commands::world::set_world_entry_tags,
            commands::world::get_world_entry_tags,
            commands::locations::create_location,
            commands::locations::list_locations,
            commands::locations::update_location,
            commands::locations::delete_location,
            commands::glossary::create_glossary_entry,
            commands::glossary::list_glossary_entries,
            commands::glossary::update_glossary_entry,
            commands::glossary::delete_glossary_entry,
            commands::notes::create_note,
            commands::notes::list_notes,
            commands::notes::save_note,
            commands::notes::delete_note,
            commands::tags::list_tags,
            commands::ai::list_ai_provider_kinds,
            commands::ai::list_ai_writing_features,
            commands::ai::run_ai_writing_feature,
            commands::ai::set_ai_api_key,
            commands::ai::has_ai_api_key,
            commands::ai::clear_ai_api_key,
            commands::ai::build_ai_context,
            commands::ai::list_ai_chat_messages,
            commands::ai::clear_ai_chat,
            commands::ai::send_ai_chat_message,
            commands::ai::get_ai_usage_summary,
            commands::plot::list_plot_lanes,
            commands::plot::create_plot_lane,
            commands::plot::rename_plot_lane,
            commands::plot::delete_plot_lane,
            commands::plot::list_plot_cards,
            commands::plot::create_plot_card,
            commands::plot::update_plot_card,
            commands::plot::move_plot_card,
            commands::plot::delete_plot_card,
            commands::timeline::list_timeline_events,
            commands::timeline::create_timeline_event,
            commands::timeline::update_timeline_event,
            commands::timeline::delete_timeline_event,
            commands::foreshadowing::list_foreshadowings,
            commands::foreshadowing::create_foreshadowing,
            commands::foreshadowing::update_foreshadowing,
            commands::foreshadowing::delete_foreshadowing,
            commands::todos::list_todos,
            commands::todos::create_todo,
            commands::todos::update_todo,
            commands::todos::set_todo_done,
            commands::todos::delete_todo,
            commands::comments::list_comments,
            commands::comments::create_comment,
            commands::comments::set_comment_resolved,
            commands::comments::delete_comment,
            commands::analysis::list_ai_analysis_types,
            commands::analysis::build_ai_analysis_context,
            commands::analysis::compute_readability_stats,
            commands::analysis::run_ai_analysis,
            commands::analysis::list_ai_analysis_reports,
            commands::analysis::delete_ai_analysis_report,
            commands::revisions::list_revisions,
            commands::revisions::get_revision,
            commands::revisions::create_manual_revision,
            commands::revisions::restore_revision,
            commands::trash::list_trash,
            commands::trash::restore_trash_item,
            commands::trash::purge_trash_item,
            commands::trash::purge_all_trash,
            commands::backup::was_unclean_shutdown,
            commands::backup::list_backups,
            commands::backup::backup_now,
            commands::backup::restore_backup,
            commands::export::list_export_formats,
            commands::export::export_project,
            commands::about::get_changelog,
            commands::about::get_third_party_notices,
            commands::about::get_privacy_notice,
            commands::about::export_diagnostics,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // Phase 8: Crash Recovery -- 正常終了できた時だけセッション
            // マーカーを下ろす。ここに到達せずプロセスが終わった場合
            // (クラッシュ/強制終了)は次回起動時に前回の異常終了として
            // 検出される。
            if let tauri::RunEvent::Exit = event {
                if let Some(db) = app_handle.try_state::<Db>() {
                    if let Ok(conn) = db.conn.lock() {
                        let _ = repositories::settings_repository::set(&conn, SESSION_OPEN_MARKER_KEY, "0");
                    }
                }
            }
        });
}
