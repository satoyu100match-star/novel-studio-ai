import { useEffect, useState } from "react";
import type { Location, LocationInput } from "@/types/tauriCommands";
import * as locationService from "@/services/locationService";

function toInput(l: Location): LocationInput {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { id, project_id, ...rest } = l;
  return rest;
}

/** 場所管理(仕様#28)。国/地域/都市/建物/部屋の階層は parent_location_id で表現。
 * 地図(仕様#29)はPhase後半として見送り。 */
export function LocationsTab({ projectId }: { projectId: string }) {
  const [locations, setLocations] = useState<Location[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [newName, setNewName] = useState("");

  useEffect(() => {
    locationService.listLocations(projectId).then(setLocations);
  }, [projectId]);

  const selected = locations.find((l) => l.id === selectedId) ?? null;

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    const name = newName.trim();
    if (!name) return;
    const created = await locationService.createLocation(projectId, { ...locationService.emptyLocationInput(), name });
    setLocations((ls) => [...ls, created]);
    setNewName("");
    setSelectedId(created.id);
  }

  return (
    <div className="section-layout">
      <div className="section-layout__body">
        <div className="section-layout__list">
          <form onSubmit={handleCreate} className="field-row">
            <input value={newName} onChange={(e) => setNewName(e.target.value)} placeholder="新しい場所" />
            <button className="primary" type="submit">追加</button>
          </form>
          <ul className="entity-list">
            {locations.map((loc) => (
              <li key={loc.id}>
                <button
                  className={`entity-list__item ${selectedId === loc.id ? "entity-list__item--selected" : ""}`}
                  onClick={() => setSelectedId(loc.id)}
                >
                  <span>{loc.name}</span>
                  <span className="entity-list__meta">
                    {loc.parent_location_id ? locations.find((p) => p.id === loc.parent_location_id)?.name : ""}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        </div>
        <div className="section-layout__detail">
          {selected ? (
            <LocationForm
              location={selected}
              locations={locations}
              onSaved={(updated) => setLocations((ls) => ls.map((l) => (l.id === updated.id ? updated : l)))}
              onDeleted={() => {
                setLocations((ls) => ls.filter((l) => l.id !== selected.id));
                setSelectedId(null);
              }}
            />
          ) : (
            <p className="status-line">左の一覧から場所を選択してください。</p>
          )}
        </div>
      </div>
    </div>
  );
}

function LocationForm({ location, locations, onSaved, onDeleted }: {
  location: Location;
  locations: Location[];
  onSaved: (l: Location) => void;
  onDeleted: () => void;
}) {
  const [form, setForm] = useState<LocationInput>(toInput(location));
  const [saving, setSaving] = useState(false);

  useEffect(() => setForm(toInput(location)), [location]);

  async function handleSave() {
    setSaving(true);
    const updated = await locationService.updateLocation(location.id, form);
    setSaving(false);
    onSaved(updated);
  }

  return (
    <div className="detail-form">
      <label className="form-label">
        名前
        <input value={form.name} onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))} />
      </label>
      <label className="form-label">
        親の場所
        <select
          value={form.parent_location_id ?? ""}
          onChange={(e) => setForm((f) => ({ ...f, parent_location_id: e.target.value || null }))}
        >
          <option value="">(なし)</option>
          {locations.filter((l) => l.id !== location.id).map((l) => (
            <option key={l.id} value={l.id}>{l.name}</option>
          ))}
        </select>
      </label>
      <label className="form-label">
        説明
        <textarea rows={4} value={form.description ?? ""} onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))} />
      </label>
      <label className="form-label">
        メモ
        <textarea rows={2} value={form.memo ?? ""} onChange={(e) => setForm((f) => ({ ...f, memo: e.target.value }))} />
      </label>
      <div className="field-row">
        <button className="primary" onClick={handleSave} disabled={saving}>保存</button>
        <button className="secondary" onClick={async () => { await locationService.deleteLocation(location.id); onDeleted(); }}>削除</button>
      </div>
    </div>
  );
}
