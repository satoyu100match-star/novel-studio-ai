import { useEffect, useState } from "react";
import type { Character, CharacterInput, CharacterRelation } from "@/types/tauriCommands";
import * as characterService from "@/services/characterService";
import { TagInput } from "@/components/TagInput";

function toInput(c: Character): CharacterInput {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { id, project_id, ...rest } = c;
  return rest;
}

/** グループごとのフィールド定義。仕様#22の分類(基本/外見/内面/話し方/ストーリー/その他)に対応。 */
const FIELD_GROUPS: { heading: string; fields: [keyof CharacterInput, string][] }[] = [
  {
    heading: "基本",
    fields: [
      ["name", "名前"], ["reading", "読み"], ["aliases", "別名"], ["age", "年齢"],
      ["gender", "性別"], ["birthday", "誕生日"], ["height", "身長"], ["occupation", "職業"],
      ["affiliation", "所属"], ["role", "役割"], ["first_appearance", "初登場"],
    ],
  },
  {
    heading: "外見",
    fields: [
      ["hair", "髪"], ["eyes", "目"], ["build", "体格"], ["clothing", "服装"],
      ["features", "特徴"], ["scars", "傷"], ["equipment", "装備"],
    ],
  },
  {
    heading: "内面",
    fields: [
      ["personality", "性格"], ["strengths", "長所"], ["weaknesses", "短所"], ["beliefs", "信念"],
      ["desires", "欲望"], ["fears", "恐怖"], ["secret", "秘密"], ["trauma", "トラウマ"],
    ],
  },
  {
    heading: "話し方",
    fields: [
      ["first_person", "一人称"], ["second_person", "二人称"], ["speech_suffix", "語尾"],
      ["catchphrase", "口癖"], ["honorific_level", "敬語レベル"],
      ["calls_protagonist", "主人公の呼び方"], ["calls_others", "他キャラクターの呼び方"],
    ],
  },
  {
    heading: "ストーリー",
    fields: [
      ["goal", "目的"], ["motivation", "動機"], ["past", "過去"], ["initial_state", "初期状態"],
      ["middle_state", "中間状態"], ["final_state", "最終状態"], ["character_arc", "キャラクターアーク"],
    ],
  },
  { heading: "その他", fields: [["memo", "メモ"]] },
];

function CharacterForm({
  projectId,
  character,
  onSaved,
  onDeleted,
}: {
  projectId: string;
  character: Character;
  onSaved: (c: Character) => void;
  onDeleted: () => void;
}) {
  const [form, setForm] = useState<CharacterInput>(toInput(character));
  const [tags, setTags] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setForm(toInput(character));
    characterService.getCharacterTags(character.id).then(setTags);
  }, [character]);

  function field(key: keyof CharacterInput) {
    return {
      value: (form[key] as string | null | undefined) ?? "",
      onChange: (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) =>
        setForm((f) => ({ ...f, [key]: e.target.value })),
    };
  }

  async function handleSave() {
    setSaving(true);
    const updated = await characterService.updateCharacter(character.id, form);
    await characterService.setCharacterTags(projectId, character.id, tags);
    setSaving(false);
    onSaved(updated);
  }

  return (
    <div className="detail-form">
      {FIELD_GROUPS.map((group) => (
        <fieldset key={group.heading} className="detail-form__group">
          <legend>{group.heading}</legend>
          {group.fields.map(([key, label]) =>
            key === "memo" || key === "personality" || key === "past" || key === "character_arc" ? (
              <label className="form-label" key={key}>
                {label}
                <textarea rows={2} {...field(key)} />
              </label>
            ) : (
              <label className="form-label" key={key}>
                {label}
                <input {...field(key)} />
              </label>
            ),
          )}
        </fieldset>
      ))}
      <fieldset className="detail-form__group">
        <legend>タグ</legend>
        <TagInput tags={tags} onChange={setTags} />
      </fieldset>
      <div className="field-row">
        <button className="primary" onClick={handleSave} disabled={saving}>
          保存
        </button>
        <button
          className="secondary"
          onClick={async () => {
            await characterService.deleteCharacter(character.id);
            onDeleted();
          }}
        >
          削除
        </button>
      </div>
    </div>
  );
}

function RelationsTab({ projectId, characters }: { projectId: string; characters: Character[] }) {
  const [relations, setRelations] = useState<CharacterRelation[]>([]);
  const [fromId, setFromId] = useState("");
  const [toId, setToId] = useState("");
  const [label, setLabel] = useState("");

  useEffect(() => {
    characterService.listCharacterRelations(projectId).then(setRelations);
  }, [projectId]);

  const nameOf = (id: string) => characters.find((c) => c.id === id)?.name ?? "?";

  async function addRelation(e: React.FormEvent) {
    e.preventDefault();
    if (!fromId || !toId || !label.trim()) return;
    const relation = await characterService.createCharacterRelation(projectId, fromId, toId, label.trim(), "bidirectional");
    setRelations((r) => [...r, relation]);
    setLabel("");
  }

  return (
    <div className="card" style={{ maxWidth: "none" }}>
      <h1>人物相関(仕様#24)</h1>
      <p>
        グラフ表示ではなく一覧形式の簡易実装です(Phase 3時点。Node Graph表示はPhase 11候補)。
      </p>
      <form className="field-row" onSubmit={addRelation}>
        <select value={fromId} onChange={(e) => setFromId(e.target.value)} aria-label="人物A">
          <option value="">人物A</option>
          {characters.map((c) => (
            <option key={c.id} value={c.id}>{c.name}</option>
          ))}
        </select>
        <select value={toId} onChange={(e) => setToId(e.target.value)} aria-label="人物B">
          <option value="">人物B</option>
          {characters.map((c) => (
            <option key={c.id} value={c.id}>{c.name}</option>
          ))}
        </select>
        <input value={label} onChange={(e) => setLabel(e.target.value)} placeholder="関係 (例: 幼馴染)" />
        <button className="primary" type="submit">追加</button>
      </form>
      <ul className="relation-list">
        {relations.map((r) => (
          <li key={r.id}>
            {nameOf(r.from_character_id)} ⇔ {nameOf(r.to_character_id)}: {r.label}
            <button
              className="secondary"
              onClick={async () => {
                await characterService.deleteCharacterRelation(r.id);
                setRelations((rs) => rs.filter((x) => x.id !== r.id));
              }}
            >
              削除
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}

export function CharactersPanel({ projectId }: { projectId: string }) {
  const [characters, setCharacters] = useState<Character[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [tab, setTab] = useState<"list" | "relations">("list");
  const [newName, setNewName] = useState("");

  useEffect(() => {
    characterService.listCharacters(projectId).then(setCharacters);
  }, [projectId]);

  const selected = characters.find((c) => c.id === selectedId) ?? null;

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    const name = newName.trim();
    if (!name) return;
    const created = await characterService.createCharacter(projectId, { ...characterService.emptyCharacterInput(), name });
    setCharacters((cs) => [...cs, created]);
    setNewName("");
    setSelectedId(created.id);
  }

  return (
    <div className="section-layout">
      <div className="section-layout__tabs">
        <button className={tab === "list" ? "tab tab--active" : "tab"} onClick={() => setTab("list")}>
          一覧
        </button>
        <button className={tab === "relations" ? "tab tab--active" : "tab"} onClick={() => setTab("relations")}>
          人物相関
        </button>
      </div>

      {tab === "relations" ? (
        <RelationsTab projectId={projectId} characters={characters} />
      ) : (
        <div className="section-layout__body">
          <div className="section-layout__list">
            <form onSubmit={handleCreate} className="field-row">
              <input value={newName} onChange={(e) => setNewName(e.target.value)} placeholder="新しいキャラクター名" />
              <button className="primary" type="submit">追加</button>
            </form>
            <ul className="entity-list">
              {characters.map((c) => (
                <li key={c.id}>
                  <button
                    className={`entity-list__item ${selectedId === c.id ? "entity-list__item--selected" : ""}`}
                    onClick={() => setSelectedId(c.id)}
                  >
                    <span>{c.name}</span>
                    <span className="entity-list__meta">{c.role ?? ""}</span>
                  </button>
                </li>
              ))}
            </ul>
          </div>
          <div className="section-layout__detail">
            {selected ? (
              <CharacterForm
                projectId={projectId}
                character={selected}
                onSaved={(updated) => setCharacters((cs) => cs.map((c) => (c.id === updated.id ? updated : c)))}
                onDeleted={() => {
                  setCharacters((cs) => cs.filter((c) => c.id !== selected.id));
                  setSelectedId(null);
                }}
              />
            ) : (
              <p className="status-line">左の一覧からキャラクターを選択してください。</p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
