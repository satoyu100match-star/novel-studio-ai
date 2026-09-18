import { useState } from "react";
import type { Project, ProjectInput } from "@/types/tauriCommands";
import * as projectService from "@/services/projectService";

/** 作品設定(仕様#11)。 */
export function ProjectSettingsForm({
  project,
  onSaved,
  onClose,
}: {
  project: Project;
  onSaved: (p: Project) => void;
  onClose: () => void;
}) {
  const [form, setForm] = useState<ProjectInput>({
    title: project.title,
    subtitle: project.subtitle ?? "",
    authorName: project.authorName ?? "",
    penName: project.penName ?? "",
    genre: project.genre ?? "",
    targetAudience: project.targetAudience ?? "",
    targetLength: project.targetLength ?? undefined,
    deadline: project.deadline ?? "",
    synopsis: project.synopsis ?? "",
    theme: project.theme ?? "",
    concept: project.concept ?? "",
    styleMemo: project.styleMemo ?? "",
    povPolicy: project.povPolicy ?? "",
    tense: project.tense ?? "",
    aiPolicy: project.aiPolicy ?? "",
  });
  const [saving, setSaving] = useState(false);

  function field<K extends keyof ProjectInput>(key: K) {
    return {
      value: (form[key] as string | number | undefined) ?? "",
      onChange: (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) =>
        setForm((f) => ({ ...f, [key]: e.target.value })),
    };
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSaving(true);
    const updated = await projectService.updateProject(project.id, {
      ...form,
      targetLength: form.targetLength ? Number(form.targetLength) : null,
    });
    setSaving(false);
    onSaved(updated);
  }

  return (
    <div className="modal-overlay" role="dialog" aria-label="作品設定">
      <form className="modal card" onSubmit={handleSubmit}>
        <h1>作品設定</h1>
        <label className="form-label">
          タイトル
          <input required {...field("title")} />
        </label>
        <label className="form-label">
          サブタイトル
          <input {...field("subtitle")} />
        </label>
        <div className="form-grid">
          <label className="form-label">
            作者名
            <input {...field("authorName")} />
          </label>
          <label className="form-label">
            ペンネーム
            <input {...field("penName")} />
          </label>
          <label className="form-label">
            ジャンル
            <input {...field("genre")} />
          </label>
          <label className="form-label">
            対象読者
            <input {...field("targetAudience")} />
          </label>
          <label className="form-label">
            想定文字数
            <input type="number" {...field("targetLength")} />
          </label>
          <label className="form-label">
            締切
            <input type="date" {...field("deadline")} />
          </label>
          <label className="form-label">
            POV方針
            <input {...field("povPolicy")} />
          </label>
          <label className="form-label">
            時制
            <input {...field("tense")} />
          </label>
        </div>
        <label className="form-label">
          あらすじ
          <textarea rows={3} {...field("synopsis")} />
        </label>
        <label className="form-label">
          テーマ
          <textarea rows={2} {...field("theme")} />
        </label>
        <label className="form-label">
          作品コンセプト
          <textarea rows={2} {...field("concept")} />
        </label>
        <label className="form-label">
          文体メモ
          <textarea rows={2} {...field("styleMemo")} />
        </label>
        <label className="form-label">
          AI向け作品方針
          <textarea rows={2} {...field("aiPolicy")} />
        </label>
        <div className="field-row" style={{ justifyContent: "flex-end" }}>
          <button type="button" className="secondary" onClick={onClose}>
            キャンセル
          </button>
          <button type="submit" className="primary" disabled={saving}>
            保存
          </button>
        </div>
      </form>
    </div>
  );
}
