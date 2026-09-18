import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useProjectListStore } from "@/stores/projectStore";
import { APP_NAME_FALLBACK } from "@/app/config";
import { readAppInfo } from "@/services/appInfoService";
import * as backupService from "@/services/backupService";
import type { BackupInfo } from "@/types/tauriCommands";

/**
 * 起動画面 / 作品一覧(仕様#10)。「サンプル作品」はPhase10で実装済み
 * (`is_sample`フラグ自体はPhase1から存在していたが、実際に生成する経路は
 * 無かった)。「バックアップから復元」はPhase8で実装済み(DBはアプリ全体で
 * 1ファイルのため、作品個別ではなくこの起動画面に置く)。
 */
export function ProjectListPage() {
  const projects = useProjectListStore((s) => s.projects);
  const loading = useProjectListStore((s) => s.loading);
  const load = useProjectListStore((s) => s.load);
  const create = useProjectListStore((s) => s.create);
  const createSample = useProjectListStore((s) => s.createSample);
  const remove = useProjectListStore((s) => s.remove);
  const navigate = useNavigate();
  const [deletingId, setDeletingId] = useState<string | null>(null);

  const [appName, setAppName] = useState(APP_NAME_FALLBACK);
  const [newTitle, setNewTitle] = useState("");
  const [creating, setCreating] = useState(false);
  const [creatingSample, setCreatingSample] = useState(false);

  const [showBackups, setShowBackups] = useState(false);
  const [backups, setBackups] = useState<BackupInfo[]>([]);
  const [backupBusy, setBackupBusy] = useState(false);
  const [backupMessage, setBackupMessage] = useState("");

  useEffect(() => {
    load();
    readAppInfo().then((info) => setAppName(info.name));
  }, [load]);

  async function loadBackups() {
    setBackups(await backupService.listBackups());
  }

  async function handleToggleBackups() {
    const next = !showBackups;
    setShowBackups(next);
    if (next) await loadBackups();
  }

  async function handleBackupNow() {
    setBackupBusy(true);
    setBackupMessage("");
    try {
      await backupService.backupNow();
      setBackupMessage("バックアップを作成しました。");
      await loadBackups();
    } finally {
      setBackupBusy(false);
    }
  }

  async function handleRestoreBackup(filename: string) {
    if (
      !window.confirm(
        "選択したバックアップの内容で、すべての作品データを置き換えます。この操作の後は現在のデータには戻れません。よろしいですか?",
      )
    ) {
      return;
    }
    setBackupBusy(true);
    setBackupMessage("");
    try {
      await backupService.restoreBackup(filename);
      setBackupMessage("復元しました。最新の状態を表示するため、作品一覧を再読み込みします。");
      await load();
    } finally {
      setBackupBusy(false);
    }
  }

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    const title = newTitle.trim();
    if (!title) return;
    setCreating(true);
    const project = await create({ title });
    setCreating(false);
    setNewTitle("");
    navigate(`/projects/${project.id}`);
  }

  async function handleCreateSample() {
    setCreatingSample(true);
    try {
      const project = await createSample();
      navigate(`/projects/${project.id}`);
    } finally {
      setCreatingSample(false);
    }
  }

  // 作品一覧からの削除(仕様上はSoft Delete)。バックエンド・ストア
  // (useProjectListStore.remove)はPhase1から存在していたが、この画面に
  // 呼び出しボタンが一つも無く、実際には削除する手段がなかった
  // (ユーザー報告により発覚)。削除済み作品を一覧・復元するUIはまだ無い
  // ため、その旨を確認ダイアログで明示する(既知の制約、CLAUDE.md参照)。
  async function handleDelete(id: string, title: string) {
    if (
      !window.confirm(
        `「${title}」を削除しますか?\n\n現在、削除した作品を作品一覧の画面から元に戻す機能はありません。`,
      )
    ) {
      return;
    }
    setDeletingId(id);
    try {
      await remove(id);
    } finally {
      setDeletingId(null);
    }
  }

  return (
    <div className="app-shell__body app-shell__body--scroll">
      <h2 style={{ margin: 0 }}>{appName}</h2>

      <div className="card">
        <h1>新しい小説</h1>
        <p>タイトルを入力して作品を作成します。詳細設定は後から変更できます。</p>
        <form onSubmit={handleCreate} className="field-row">
          <input
            value={newTitle}
            onChange={(e) => setNewTitle(e.target.value)}
            placeholder="作品タイトル"
            aria-label="新しい作品のタイトル"
          />
          <button className="primary" type="submit" disabled={creating || !newTitle.trim()}>
            作成
          </button>
        </form>
        <p className="status-line">
          何もない状態から試すより先に、機能を一通り触ってみたい場合は、
          人物・世界観・プロットなどが最初から入ったサンプル作品を開けます。
        </p>
        <div className="field-row">
          <button className="secondary" type="button" onClick={handleCreateSample} disabled={creatingSample}>
            {creatingSample ? "作成中..." : "サンプル作品を開く"}
          </button>
        </div>
      </div>

      <div className="card" style={{ maxWidth: 640 }}>
        <h1>最近開いた作品</h1>
        {loading && <p>読み込み中...</p>}
        {!loading && projects.length === 0 && <p>まだ作品がありません。上のフォームから新規作成してください。</p>}
        <ul className="project-list">
          {projects.map((p) => (
            <li key={p.id} className="project-list__row">
              <button className="project-list__item" onClick={() => navigate(`/projects/${p.id}`)}>
                <span className="project-list__title">{p.title}</span>
                <span className="project-list__meta">
                  {p.genre ?? "ジャンル未設定"} ・ 更新: {new Date(p.updatedAt).toLocaleDateString("ja-JP")}
                </span>
              </button>
              <button
                className="secondary"
                type="button"
                onClick={() => handleDelete(p.id, p.title)}
                disabled={deletingId === p.id}
              >
                {deletingId === p.id ? "削除中..." : "削除"}
              </button>
            </li>
          ))}
        </ul>
      </div>

      <div className="card" style={{ maxWidth: 640 }}>
        <div className="field-row" style={{ justifyContent: "space-between" }}>
          <h1 style={{ margin: 0 }}>バックアップ</h1>
          <button className="secondary" type="button" onClick={handleToggleBackups}>
            {showBackups ? "閉じる" : "表示"}
          </button>
        </div>
        {showBackups && (
          <>
            <p className="status-line">
              起動のたびに自動でバックアップが作成されます(最新10件を保持)。作品データ全体
              (すべての作品を含むデータベースファイル)が対象です。
            </p>
            <div className="field-row">
              <button className="secondary" type="button" onClick={handleBackupNow} disabled={backupBusy}>
                {backupBusy ? "処理中..." : "今すぐバックアップ"}
              </button>
            </div>
            {backupMessage && <p className="status-line">{backupMessage}</p>}
            {backups.length === 0 && <p className="status-line">バックアップはまだありません。</p>}
            <ul className="entity-list">
              {backups.map((b) => (
                <li key={b.filename} className="entity-list__item" style={{ cursor: "default" }}>
                  <span>{new Date(b.created_at).toLocaleString()}</span>
                  <span className="entity-list__meta">
                    {(b.size_bytes / 1024).toFixed(0)}KB
                    <button
                      className="secondary"
                      type="button"
                      style={{ marginLeft: 8 }}
                      onClick={() => handleRestoreBackup(b.filename)}
                      disabled={backupBusy}
                    >
                      このバックアップに復元
                    </button>
                  </span>
                </li>
              ))}
            </ul>
          </>
        )}
      </div>
    </div>
  );
}
