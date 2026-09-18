/**
 * Typed wrappers around the Tauri commands exposed by src-tauri.
 *
 * All backend responses are validated with Zod before the rest of the app
 * trusts their shape -- the Rust side and the TS side can drift, and a
 * silent shape mismatch here would otherwise surface as a confusing bug
 * deep in a feature. This is the *only* place that calls `invoke`
 * directly; features go through `src/services/*`, never straight to
 * `@tauri-apps/api`.
 */
import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";

// ---------------------------------------------------------------------------
// app_info / settings (Phase 0)
// ---------------------------------------------------------------------------

const AppInfoSchema = z.object({
  name: z.string(),
  version: z.string(),
  identifier: z.string(),
});
export type AppInfo = z.infer<typeof AppInfoSchema>;

export async function getAppInfo(): Promise<AppInfo> {
  return AppInfoSchema.parse(await invoke("app_info"));
}

export async function getSetting(key: string): Promise<string | null> {
  return z.string().nullable().parse(await invoke("get_setting", { key }));
}

export async function setSetting(key: string, value: string): Promise<void> {
  await invoke("set_setting", { key, value });
}

export async function listSettings(): Promise<[string, string][]> {
  return z.array(z.tuple([z.string(), z.string()])).parse(await invoke("list_settings"));
}

// ---------------------------------------------------------------------------
// Projects (Phase 1)
// ---------------------------------------------------------------------------

export const ProjectSchema = z.object({
  id: z.string(),
  title: z.string(),
  subtitle: z.string().nullable(),
  authorName: z.string().nullable(),
  penName: z.string().nullable(),
  genre: z.string().nullable(),
  targetAudience: z.string().nullable(),
  targetLength: z.number().nullable(),
  deadline: z.string().nullable(),
  synopsis: z.string().nullable(),
  theme: z.string().nullable(),
  concept: z.string().nullable(),
  styleMemo: z.string().nullable(),
  povPolicy: z.string().nullable(),
  tense: z.string().nullable(),
  aiPolicy: z.string().nullable(),
  isSample: z.boolean(),
  createdAt: z.string(),
  updatedAt: z.string(),
});
export type Project = z.infer<typeof ProjectSchema>;

export interface ProjectInput {
  title: string;
  subtitle?: string | null;
  authorName?: string | null;
  penName?: string | null;
  genre?: string | null;
  targetAudience?: string | null;
  targetLength?: number | null;
  deadline?: string | null;
  synopsis?: string | null;
  theme?: string | null;
  concept?: string | null;
  styleMemo?: string | null;
  povPolicy?: string | null;
  tense?: string | null;
  aiPolicy?: string | null;
}

// Rust's serde (default, snake_case field names) is what actually crosses
// the wire; camelCase above is the idiomatic TS-facing shape. This helper
// pair is the one place that translates between them.
function projectFromWire(raw: unknown): Project {
  const s = z
    .object({
      id: z.string(),
      title: z.string(),
      subtitle: z.string().nullable(),
      author_name: z.string().nullable(),
      pen_name: z.string().nullable(),
      genre: z.string().nullable(),
      target_audience: z.string().nullable(),
      target_length: z.number().nullable(),
      deadline: z.string().nullable(),
      synopsis: z.string().nullable(),
      theme: z.string().nullable(),
      concept: z.string().nullable(),
      style_memo: z.string().nullable(),
      pov_policy: z.string().nullable(),
      tense: z.string().nullable(),
      ai_policy: z.string().nullable(),
      is_sample: z.boolean(),
      created_at: z.string(),
      updated_at: z.string(),
    })
    .parse(raw);
  return {
    id: s.id,
    title: s.title,
    subtitle: s.subtitle,
    authorName: s.author_name,
    penName: s.pen_name,
    genre: s.genre,
    targetAudience: s.target_audience,
    targetLength: s.target_length,
    deadline: s.deadline,
    synopsis: s.synopsis,
    theme: s.theme,
    concept: s.concept,
    styleMemo: s.style_memo,
    povPolicy: s.pov_policy,
    tense: s.tense,
    aiPolicy: s.ai_policy,
    isSample: s.is_sample,
    createdAt: s.created_at,
    updatedAt: s.updated_at,
  };
}

function projectInputToWire(input: ProjectInput) {
  return {
    title: input.title,
    subtitle: input.subtitle ?? null,
    author_name: input.authorName ?? null,
    pen_name: input.penName ?? null,
    genre: input.genre ?? null,
    target_audience: input.targetAudience ?? null,
    target_length: input.targetLength ?? null,
    deadline: input.deadline ?? null,
    synopsis: input.synopsis ?? null,
    theme: input.theme ?? null,
    concept: input.concept ?? null,
    style_memo: input.styleMemo ?? null,
    pov_policy: input.povPolicy ?? null,
    tense: input.tense ?? null,
    ai_policy: input.aiPolicy ?? null,
  };
}

export async function createProject(input: ProjectInput): Promise<Project> {
  return projectFromWire(await invoke("create_project", { input: projectInputToWire(input) }));
}
export async function createSampleProject(): Promise<Project> {
  return projectFromWire(await invoke("create_sample_project"));
}
export async function listProjects(): Promise<Project[]> {
  return z.array(z.unknown()).parse(await invoke("list_projects")).map(projectFromWire);
}
export async function getProject(id: string): Promise<Project | null> {
  const raw = await invoke("get_project", { id });
  return raw ? projectFromWire(raw) : null;
}
export async function updateProject(id: string, input: ProjectInput): Promise<Project> {
  return projectFromWire(await invoke("update_project", { id, input: projectInputToWire(input) }));
}
export async function deleteProject(id: string): Promise<void> {
  await invoke("delete_project", { id });
}

// ---------------------------------------------------------------------------
// Parts / Chapters / Scenes (Phase 1)
// ---------------------------------------------------------------------------

export const PartSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  title: z.string(),
  order_index: z.number(),
});
export type Part = z.infer<typeof PartSchema>;

export const ChapterSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  part_id: z.string().nullable(),
  title: z.string(),
  subtitle: z.string().nullable(),
  synopsis: z.string().nullable(),
  memo: z.string().nullable(),
  pov: z.string().nullable(),
  status: z.string(),
  start_at: z.string().nullable(),
  end_at: z.string().nullable(),
  location: z.string().nullable(),
  order_index: z.number(),
  char_count: z.number(),
});
export type Chapter = z.infer<typeof ChapterSchema>;

export const SceneSchema = z.object({
  id: z.string(),
  chapter_id: z.string(),
  title: z.string(),
  summary: z.string().nullable(),
  pov_character: z.string().nullable(),
  location: z.string().nullable(),
  event_date: z.string().nullable(),
  start_time: z.string().nullable(),
  end_time: z.string().nullable(),
  purpose: z.string().nullable(),
  conflict: z.string().nullable(),
  result: z.string().nullable(),
  emotion: z.string().nullable(),
  memo: z.string().nullable(),
  order_index: z.number(),
  char_count: z.number(),
});
export type Scene = z.infer<typeof SceneSchema>;

export const DocumentSchema = z.object({
  id: z.string(),
  owner_type: z.enum(["chapter", "scene"]),
  owner_id: z.string(),
  body: z.string(),
  char_count: z.number(),
  updated_at: z.string(),
});
export type ManuscriptDocument = z.infer<typeof DocumentSchema>;

export const SearchHitSchema = z.object({
  owner_type: z.enum(["chapter", "scene"]),
  owner_id: z.string(),
  title: z.string(),
  snippet: z.string(),
});
export type SearchHit = z.infer<typeof SearchHitSchema>;

export const CHAPTER_STATUSES = [
  "not_started",
  "planning",
  "writing",
  "first_draft",
  "revising",
  "done",
] as const;
export type ChapterStatus = (typeof CHAPTER_STATUSES)[number];

export async function createPart(projectId: string, title: string): Promise<Part> {
  return PartSchema.parse(await invoke("create_part", { projectId, title }));
}
export async function listParts(projectId: string): Promise<Part[]> {
  return z.array(PartSchema).parse(await invoke("list_parts", { projectId }));
}
export async function renamePart(id: string, title: string): Promise<void> {
  await invoke("rename_part", { id, title });
}
export async function deletePart(id: string): Promise<void> {
  await invoke("delete_part", { id });
}

export async function createChapter(projectId: string, partId: string | null, title: string): Promise<Chapter> {
  return ChapterSchema.parse(await invoke("create_chapter", { projectId, partId, title }));
}
export async function listChapters(projectId: string): Promise<Chapter[]> {
  return z.array(ChapterSchema).parse(await invoke("list_chapters", { projectId }));
}
export async function getChapter(id: string): Promise<Chapter | null> {
  const raw = await invoke("get_chapter", { id });
  return raw ? ChapterSchema.parse(raw) : null;
}
export async function updateChapterMeta(id: string, title: string, status: ChapterStatus): Promise<void> {
  await invoke("update_chapter_meta", { id, title, status });
}
export async function deleteChapter(id: string): Promise<void> {
  await invoke("delete_chapter", { id });
}

export async function createScene(chapterId: string, title: string): Promise<Scene> {
  return SceneSchema.parse(await invoke("create_scene", { chapterId, title }));
}
export async function listScenes(chapterId: string): Promise<Scene[]> {
  return z.array(SceneSchema).parse(await invoke("list_scenes", { chapterId }));
}
export async function getScene(id: string): Promise<Scene | null> {
  const raw = await invoke("get_scene", { id });
  return raw ? SceneSchema.parse(raw) : null;
}
export async function renameScene(id: string, title: string): Promise<void> {
  await invoke("rename_scene", { id, title });
}
export async function deleteScene(id: string): Promise<void> {
  await invoke("delete_scene", { id });
}

export async function getDocument(ownerType: "chapter" | "scene", ownerId: string): Promise<ManuscriptDocument | null> {
  const raw = await invoke("get_document", { ownerType, ownerId });
  return raw ? DocumentSchema.parse(raw) : null;
}
export async function saveDocumentBody(
  ownerType: "chapter" | "scene",
  ownerId: string,
  body: string,
): Promise<ManuscriptDocument> {
  return DocumentSchema.parse(await invoke("save_document_body", { ownerType, ownerId, body }));
}
export async function projectCharCount(projectId: string): Promise<number> {
  return z.number().parse(await invoke("project_char_count", { projectId }));
}

export async function searchProject(projectId: string, query: string): Promise<SearchHit[]> {
  return z.array(SearchHitSchema).parse(await invoke("search_project", { projectId, query }));
}

// ---------------------------------------------------------------------------
// Characters / World Bible / Glossary / Locations / Notes / Tags (Phase 3)
// ---------------------------------------------------------------------------

const nullableStr = z.string().nullable().optional();

export const CharacterSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  name: z.string(),
  reading: nullableStr,
  aliases: nullableStr,
  age: nullableStr,
  gender: nullableStr,
  birthday: nullableStr,
  height: nullableStr,
  occupation: nullableStr,
  affiliation: nullableStr,
  role: nullableStr,
  first_appearance: nullableStr,
  hair: nullableStr,
  eyes: nullableStr,
  build: nullableStr,
  clothing: nullableStr,
  features: nullableStr,
  scars: nullableStr,
  equipment: nullableStr,
  personality: nullableStr,
  strengths: nullableStr,
  weaknesses: nullableStr,
  beliefs: nullableStr,
  desires: nullableStr,
  fears: nullableStr,
  secret: nullableStr,
  trauma: nullableStr,
  first_person: nullableStr,
  second_person: nullableStr,
  speech_suffix: nullableStr,
  catchphrase: nullableStr,
  honorific_level: nullableStr,
  calls_protagonist: nullableStr,
  calls_others: nullableStr,
  goal: nullableStr,
  motivation: nullableStr,
  past: nullableStr,
  initial_state: nullableStr,
  middle_state: nullableStr,
  final_state: nullableStr,
  character_arc: nullableStr,
  memo: nullableStr,
  reference_image_path: nullableStr,
});
export type Character = z.infer<typeof CharacterSchema>;
export type CharacterInput = Omit<Character, "id" | "project_id">;

export const EMPTY_CHARACTER_INPUT: CharacterInput = { name: "" };

export const CharacterRelationSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  from_character_id: z.string(),
  to_character_id: z.string(),
  label: z.string(),
  color: nullableStr,
  direction: z.string(),
  detail: nullableStr,
  start_at: nullableStr,
  end_at: nullableStr,
});
export type CharacterRelation = z.infer<typeof CharacterRelationSchema>;

export async function createCharacter(projectId: string, input: CharacterInput): Promise<Character> {
  return CharacterSchema.parse(await invoke("create_character", { projectId, input }));
}
export async function listCharacters(projectId: string): Promise<Character[]> {
  return z.array(CharacterSchema).parse(await invoke("list_characters", { projectId }));
}
export async function getCharacter(id: string): Promise<Character | null> {
  const raw = await invoke("get_character", { id });
  return raw ? CharacterSchema.parse(raw) : null;
}
export async function updateCharacter(id: string, input: CharacterInput): Promise<Character> {
  return CharacterSchema.parse(await invoke("update_character", { id, input }));
}
export async function deleteCharacter(id: string): Promise<void> {
  await invoke("delete_character", { id });
}
export async function setCharacterTags(projectId: string, characterId: string, tags: string[]): Promise<void> {
  await invoke("set_character_tags", { projectId, characterId, tags });
}
export async function getCharacterTags(characterId: string): Promise<string[]> {
  return z.array(z.string()).parse(await invoke("get_character_tags", { characterId }));
}
export async function createCharacterRelation(
  projectId: string,
  fromCharacterId: string,
  toCharacterId: string,
  label: string,
  direction: string,
): Promise<CharacterRelation> {
  return CharacterRelationSchema.parse(
    await invoke("create_character_relation", { projectId, fromCharacterId, toCharacterId, label, direction }),
  );
}
export async function listCharacterRelations(projectId: string): Promise<CharacterRelation[]> {
  return z.array(CharacterRelationSchema).parse(await invoke("list_character_relations", { projectId }));
}
export async function deleteCharacterRelation(id: string): Promise<void> {
  await invoke("delete_character_relation", { id });
}

export const WorldCategorySchema = z.object({
  id: z.string(),
  project_id: z.string(),
  name: z.string(),
  is_builtin: z.boolean(),
  order_index: z.number(),
});
export type WorldCategory = z.infer<typeof WorldCategorySchema>;

export const WorldEntrySchema = z.object({
  id: z.string(),
  project_id: z.string(),
  category_id: z.string().nullable(),
  name: z.string(),
  reading: nullableStr,
  summary: nullableStr,
  detail: nullableStr,
  related_character_ids: z.array(z.string()),
  related_location_ids: z.array(z.string()),
  related_entry_ids: z.array(z.string()),
  image_path: nullableStr,
  memo: nullableStr,
});
export type WorldEntry = z.infer<typeof WorldEntrySchema>;
export type WorldEntryInput = Omit<WorldEntry, "id" | "project_id">;
export const EMPTY_WORLD_ENTRY_INPUT: WorldEntryInput = {
  category_id: null,
  name: "",
  related_character_ids: [],
  related_location_ids: [],
  related_entry_ids: [],
};

export async function listWorldCategories(projectId: string): Promise<WorldCategory[]> {
  return z.array(WorldCategorySchema).parse(await invoke("list_world_categories", { projectId }));
}
export async function createWorldCategory(projectId: string, name: string): Promise<WorldCategory> {
  return WorldCategorySchema.parse(await invoke("create_world_category", { projectId, name }));
}
export async function createWorldEntry(projectId: string, input: WorldEntryInput): Promise<WorldEntry> {
  return WorldEntrySchema.parse(await invoke("create_world_entry", { projectId, input }));
}
export async function listWorldEntries(projectId: string): Promise<WorldEntry[]> {
  return z.array(WorldEntrySchema).parse(await invoke("list_world_entries", { projectId }));
}
export async function updateWorldEntry(id: string, input: WorldEntryInput): Promise<WorldEntry> {
  return WorldEntrySchema.parse(await invoke("update_world_entry", { id, input }));
}
export async function deleteWorldEntry(id: string): Promise<void> {
  await invoke("delete_world_entry", { id });
}
export async function setWorldEntryTags(projectId: string, entryId: string, tags: string[]): Promise<void> {
  await invoke("set_world_entry_tags", { projectId, entryId, tags });
}
export async function getWorldEntryTags(entryId: string): Promise<string[]> {
  return z.array(z.string()).parse(await invoke("get_world_entry_tags", { entryId }));
}

export const LocationSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  parent_location_id: z.string().nullable(),
  name: z.string(),
  description: nullableStr,
  coordinates: nullableStr,
  related_character_ids: z.array(z.string()),
  image_path: nullableStr,
  memo: nullableStr,
});
export type Location = z.infer<typeof LocationSchema>;
export type LocationInput = Omit<Location, "id" | "project_id">;
export const EMPTY_LOCATION_INPUT: LocationInput = {
  parent_location_id: null,
  name: "",
  related_character_ids: [],
};

export async function createLocation(projectId: string, input: LocationInput): Promise<Location> {
  return LocationSchema.parse(await invoke("create_location", { projectId, input }));
}
export async function listLocations(projectId: string): Promise<Location[]> {
  return z.array(LocationSchema).parse(await invoke("list_locations", { projectId }));
}
export async function updateLocation(id: string, input: LocationInput): Promise<Location> {
  return LocationSchema.parse(await invoke("update_location", { id, input }));
}
export async function deleteLocation(id: string): Promise<void> {
  await invoke("delete_location", { id });
}

export const GlossaryEntrySchema = z.object({
  id: z.string(),
  project_id: z.string(),
  term: z.string(),
  reading: nullableStr,
  definition: nullableStr,
});
export type GlossaryEntry = z.infer<typeof GlossaryEntrySchema>;
export type GlossaryEntryInput = Omit<GlossaryEntry, "id" | "project_id">;

export async function createGlossaryEntry(projectId: string, input: GlossaryEntryInput): Promise<GlossaryEntry> {
  return GlossaryEntrySchema.parse(await invoke("create_glossary_entry", { projectId, input }));
}
export async function listGlossaryEntries(projectId: string): Promise<GlossaryEntry[]> {
  return z.array(GlossaryEntrySchema).parse(await invoke("list_glossary_entries", { projectId }));
}
export async function updateGlossaryEntry(id: string, input: GlossaryEntryInput): Promise<GlossaryEntry> {
  return GlossaryEntrySchema.parse(await invoke("update_glossary_entry", { id, input }));
}
export async function deleteGlossaryEntry(id: string): Promise<void> {
  await invoke("delete_glossary_entry", { id });
}

export const NoteSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  folder: z.string(),
  title: z.string(),
  body: z.string(),
  updated_at: z.string(),
});
export type Note = z.infer<typeof NoteSchema>;

export async function createNote(projectId: string, folder: string, title: string): Promise<Note> {
  return NoteSchema.parse(await invoke("create_note", { projectId, folder, title }));
}
export async function listNotes(projectId: string): Promise<Note[]> {
  return z.array(NoteSchema).parse(await invoke("list_notes", { projectId }));
}
export async function saveNote(id: string, folder: string, title: string, body: string): Promise<Note> {
  return NoteSchema.parse(await invoke("save_note", { id, folder, title, body }));
}
export async function deleteNote(id: string): Promise<void> {
  await invoke("delete_note", { id });
}

export const TagSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  name: z.string(),
  color: nullableStr,
});
export type Tag = z.infer<typeof TagSchema>;

export async function listTags(projectId: string): Promise<Tag[]> {
  return z.array(TagSchema).parse(await invoke("list_tags", { projectId }));
}

// ---------------------------------------------------------------------------
// Phase 4: Plot Board / Timeline / Foreshadowing / TODO / Comments
// ---------------------------------------------------------------------------

export const PlotLaneSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  name: z.string(),
  order_index: z.number(),
});
export type PlotLane = z.infer<typeof PlotLaneSchema>;

export const PlotCardSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  lane_id: z.string(),
  title: z.string(),
  summary: nullableStr,
  chapter_id: nullableStr,
  color: nullableStr,
  order_index: z.number(),
});
export type PlotCard = z.infer<typeof PlotCardSchema>;
export type PlotCardInput = Omit<PlotCard, "id" | "project_id">;

export async function listPlotLanes(projectId: string): Promise<PlotLane[]> {
  return z.array(PlotLaneSchema).parse(await invoke("list_plot_lanes", { projectId }));
}
export async function createPlotLane(projectId: string, name: string): Promise<PlotLane> {
  return PlotLaneSchema.parse(await invoke("create_plot_lane", { projectId, name }));
}
export async function renamePlotLane(id: string, name: string): Promise<void> {
  await invoke("rename_plot_lane", { id, name });
}
export async function deletePlotLane(id: string): Promise<void> {
  await invoke("delete_plot_lane", { id });
}
export async function listPlotCards(projectId: string): Promise<PlotCard[]> {
  return z.array(PlotCardSchema).parse(await invoke("list_plot_cards", { projectId }));
}
export async function createPlotCard(projectId: string, laneId: string, title: string): Promise<PlotCard> {
  return PlotCardSchema.parse(await invoke("create_plot_card", { projectId, laneId, title }));
}
export async function updatePlotCard(id: string, input: PlotCard): Promise<PlotCard> {
  return PlotCardSchema.parse(await invoke("update_plot_card", { id, input }));
}
export async function movePlotCard(id: string, laneId: string, orderIndex: number): Promise<PlotCard> {
  return PlotCardSchema.parse(await invoke("move_plot_card", { id, laneId, orderIndex }));
}
export async function deletePlotCard(id: string): Promise<void> {
  await invoke("delete_plot_card", { id });
}

export const TimelineEventSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  title: z.string(),
  event_date: nullableStr,
  description: nullableStr,
  chapter_id: nullableStr,
  scene_id: nullableStr,
  order_index: z.number(),
});
export type TimelineEvent = z.infer<typeof TimelineEventSchema>;

export async function listTimelineEvents(projectId: string): Promise<TimelineEvent[]> {
  return z.array(TimelineEventSchema).parse(await invoke("list_timeline_events", { projectId }));
}
export async function createTimelineEvent(projectId: string, title: string): Promise<TimelineEvent> {
  return TimelineEventSchema.parse(await invoke("create_timeline_event", { projectId, title }));
}
export async function updateTimelineEvent(id: string, input: TimelineEvent): Promise<TimelineEvent> {
  return TimelineEventSchema.parse(await invoke("update_timeline_event", { id, input }));
}
export async function deleteTimelineEvent(id: string): Promise<void> {
  await invoke("delete_timeline_event", { id });
}

// 仕様#35: 構想/設置予定/設置済/ヒント提示済/回収予定/回収済/破棄
export const FORESHADOWING_STATUSES = [
  "idea",
  "planned",
  "planted",
  "hinted",
  "payoff_planned",
  "resolved",
  "discarded",
] as const;
export type ForeshadowingStatus = (typeof FORESHADOWING_STATUSES)[number];
export const FORESHADOWING_STATUS_LABELS: Record<ForeshadowingStatus, string> = {
  idea: "構想",
  planned: "設置予定",
  planted: "設置済",
  hinted: "ヒント提示済",
  payoff_planned: "回収予定",
  resolved: "回収済",
  discarded: "破棄",
};

export const ForeshadowingSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  title: z.string(),
  detail: nullableStr,
  status: z.string(),
  planted_chapter_id: nullableStr,
  payoff_chapter_id: nullableStr,
  memo: nullableStr,
});
export type Foreshadowing = z.infer<typeof ForeshadowingSchema>;

export async function listForeshadowings(projectId: string): Promise<Foreshadowing[]> {
  return z.array(ForeshadowingSchema).parse(await invoke("list_foreshadowings", { projectId }));
}
export async function createForeshadowing(projectId: string, title: string): Promise<Foreshadowing> {
  return ForeshadowingSchema.parse(await invoke("create_foreshadowing", { projectId, title }));
}
export async function updateForeshadowing(id: string, input: Foreshadowing): Promise<Foreshadowing> {
  return ForeshadowingSchema.parse(await invoke("update_foreshadowing", { id, input }));
}
export async function deleteForeshadowing(id: string): Promise<void> {
  await invoke("delete_foreshadowing", { id });
}

export const TodoSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  title: z.string(),
  done: z.boolean(),
  due_date: nullableStr,
  memo: nullableStr,
  order_index: z.number(),
});
export type Todo = z.infer<typeof TodoSchema>;

export async function listTodos(projectId: string): Promise<Todo[]> {
  return z.array(TodoSchema).parse(await invoke("list_todos", { projectId }));
}
export async function createTodo(projectId: string, title: string): Promise<Todo> {
  return TodoSchema.parse(await invoke("create_todo", { projectId, title }));
}
export async function updateTodo(id: string, input: Todo): Promise<Todo> {
  return TodoSchema.parse(await invoke("update_todo", { id, input }));
}
export async function setTodoDone(id: string, done: boolean): Promise<Todo> {
  return TodoSchema.parse(await invoke("set_todo_done", { id, done }));
}
export async function deleteTodo(id: string): Promise<void> {
  await invoke("delete_todo", { id });
}

export const CommentSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  owner_type: z.enum(["chapter", "scene"]),
  owner_id: z.string(),
  anchor_start: z.number().nullable().optional(),
  anchor_end: z.number().nullable().optional(),
  quote: nullableStr,
  body: z.string(),
  resolved: z.boolean(),
  created_at: z.string(),
  updated_at: z.string(),
});
export type Comment = z.infer<typeof CommentSchema>;

export async function listComments(ownerType: "chapter" | "scene", ownerId: string): Promise<Comment[]> {
  return z.array(CommentSchema).parse(await invoke("list_comments", { ownerType, ownerId }));
}
export async function createComment(
  projectId: string,
  ownerType: "chapter" | "scene",
  ownerId: string,
  anchorStart: number | null,
  anchorEnd: number | null,
  quote: string | null,
  body: string,
): Promise<Comment> {
  return CommentSchema.parse(
    await invoke("create_comment", { projectId, ownerType, ownerId, anchorStart, anchorEnd, quote, body }),
  );
}
export async function setCommentResolved(id: string, resolved: boolean): Promise<Comment> {
  return CommentSchema.parse(await invoke("set_comment_resolved", { id, resolved }));
}
export async function deleteComment(id: string): Promise<void> {
  await invoke("delete_comment", { id });
}

// ---------------------------------------------------------------------------
// Phase 5: AI基盤 (Provider abstraction / Context Builder / Chat / Usage)
// ---------------------------------------------------------------------------

export const AI_PROVIDER_LABELS: Record<string, string> = {
  anthropic: "Anthropic (Claude)",
  openai: "OpenAI",
  gemini: "Google Gemini",
  openai_compatible: "OpenAI互換 (ローカルLLM等)",
};

export const ContextBlockSchema = z.object({
  label: z.string(),
  content: z.string(),
  char_count: z.number(),
  included_by_default: z.boolean(),
});
export type ContextBlock = z.infer<typeof ContextBlockSchema>;

export const AiChatMessageSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  role: z.enum(["user", "assistant"]),
  content: z.string(),
  context_summary: nullableStr,
  provider: nullableStr,
  model: nullableStr,
  created_at: z.string(),
});
export type AiChatMessage = z.infer<typeof AiChatMessageSchema>;

export const AiUsageSummarySchema = z.object({
  today_requests: z.number(),
  today_input_tokens: z.number(),
  today_output_tokens: z.number(),
  month_requests: z.number(),
  month_input_tokens: z.number(),
  month_output_tokens: z.number(),
});
export type AiUsageSummary = z.infer<typeof AiUsageSummarySchema>;

export async function listAiProviderKinds(): Promise<string[]> {
  return z.array(z.string()).parse(await invoke("list_ai_provider_kinds"));
}
export async function setAiApiKey(provider: string, apiKey: string): Promise<void> {
  await invoke("set_ai_api_key", { provider, apiKey });
}
export async function hasAiApiKey(provider: string): Promise<boolean> {
  return z.boolean().parse(await invoke("has_ai_api_key", { provider }));
}
export async function clearAiApiKey(provider: string): Promise<void> {
  await invoke("clear_ai_api_key", { provider });
}
export async function buildAiContext(
  projectId: string,
  chapterId: string | null,
  sceneId: string | null,
  userQuestion: string | null,
): Promise<ContextBlock[]> {
  return z
    .array(ContextBlockSchema)
    .parse(await invoke("build_ai_context", { projectId, chapterId, sceneId, userQuestion }));
}
export async function listAiChatMessages(projectId: string): Promise<AiChatMessage[]> {
  return z.array(AiChatMessageSchema).parse(await invoke("list_ai_chat_messages", { projectId }));
}
export async function clearAiChat(projectId: string): Promise<void> {
  await invoke("clear_ai_chat", { projectId });
}
export async function sendAiChatMessage(
  projectId: string,
  providerKind: string,
  model: string,
  baseUrl: string | null,
  temperature: number | null,
  maxOutputTokens: number | null,
  selectedContext: ContextBlock[],
  userContent: string,
): Promise<AiChatMessage[]> {
  return z.array(AiChatMessageSchema).parse(
    await invoke("send_ai_chat_message", {
      projectId,
      providerKind,
      model,
      baseUrl,
      temperature,
      maxOutputTokens,
      selectedContext,
      userContent,
    }),
  );
}
export async function getAiUsageSummary(projectId: string | null): Promise<AiUsageSummary> {
  return AiUsageSummarySchema.parse(await invoke("get_ai_usage_summary", { projectId }));
}

// ---------------------------------------------------------------------------
// Phase 6: AI小説支援機能 (推敲/続き案/描写追加/会話改善/各種生成)
// ---------------------------------------------------------------------------

export const AI_WRITING_FEATURE_LABELS: Record<string, string> = {
  rewrite: "推敲",
  continue: "続き案",
  add_description: "描写追加",
  improve_dialogue: "会話改善",
  generate_character: "キャラクター生成",
  generate_world: "世界観項目生成",
  generate_plot: "プロット生成",
  generate_synopsis: "あらすじ生成",
  generate_titles: "タイトル案",
  generate_concept: "作品設計を一括生成",
};

/**
 * Phase13: 「どの機能が何をするか分かりにくい」というフィードバックを
 * 受けて追加。`AiWritingAssistPanel`の機能選択プルダウンの下に、選択中の
 * 機能が何をするかを1行で表示するために使う。
 */
export const AI_WRITING_FEATURE_DESCRIPTIONS: Record<string, string> = {
  rewrite: "選択中の本文を、意味を変えずに文章の質を高めて書き直します。",
  continue: "選択中の本文の続きとなる文章を提案します。",
  add_description: "選択中の本文に、情景・五感・心理などの描写を加えて書き直します。",
  improve_dialogue: "選択中の本文の会話部分を、自然さやキャラクターらしさを保ちながら磨きます。",
  generate_character: "新しい登場人物を1人提案します(「人物」セクションに追加できます)。",
  generate_world: "新しい世界観項目を1つ提案します(「世界観」セクションに追加できます)。",
  generate_plot: "プロットのアイデアを2〜4件提案します(「ストーリー」セクションに追加できます)。",
  generate_synopsis: "作品全体のあらすじを提案します(作品設定に反映できます)。",
  generate_titles: "作品のタイトル候補を5件提案します(コピーして使えます)。",
  generate_concept:
    "タイトル・あらすじ・キャラクター・世界観・プロットを一括で提案します。「作品設計チャット」タブで使えます。",
};

export const AiSendResponseSchema = z.object({
  content: z.string(),
  input_tokens: z.number().nullable().optional(),
  output_tokens: z.number().nullable().optional(),
});
export type AiSendResponseDto = z.infer<typeof AiSendResponseSchema>;

export async function listAiWritingFeatures(): Promise<string[]> {
  return z.array(z.string()).parse(await invoke("list_ai_writing_features"));
}

export async function runAiWritingFeature(
  projectId: string,
  feature: string,
  providerKind: string,
  model: string,
  baseUrl: string | null,
  temperature: number | null,
  maxOutputTokens: number | null,
  selectedContext: ContextBlock[],
  inputText: string | null,
): Promise<AiSendResponseDto> {
  return AiSendResponseSchema.parse(
    await invoke("run_ai_writing_feature", {
      projectId,
      feature,
      providerKind,
      model,
      baseUrl,
      temperature,
      maxOutputTokens,
      selectedContext,
      inputText,
    }),
  );
}

// ---------------------------------------------------------------------------
// Phase 7: 高度AI分析 (矛盾チェック/Character口調/Timeline矛盾/設定矛盾/
// 未回収伏線/文体分析) + 可読性分析(AI不要)
// ---------------------------------------------------------------------------

export const AI_ANALYSIS_TYPE_LABELS: Record<string, string> = {
  contradiction: "矛盾チェック",
  character_tone: "キャラクター口調チェック",
  timeline: "時系列矛盾チェック",
  setting: "設定矛盾チェック",
  foreshadowing: "伏線チェック",
  style: "文体講評",
};

export const AiAnalysisReportSchema = z.object({
  id: z.string(),
  project_id: z.string(),
  analysis_type: z.string(),
  target_summary: nullableStr,
  context_summary: nullableStr,
  result: z.string(),
  provider: z.string(),
  model: z.string(),
  created_at: z.string(),
});
export type AiAnalysisReport = z.infer<typeof AiAnalysisReportSchema>;

export const ReadabilityStatsSchema = z.object({
  total_chars: z.number(),
  sentence_count: z.number(),
  avg_sentence_length: z.number(),
  max_sentence_length: z.number(),
  long_sentence_count: z.number(),
  dialogue_ratio: z.number(),
  kanji_ratio: z.number(),
  hiragana_ratio: z.number(),
  katakana_ratio: z.number(),
  avg_touten_per_sentence: z.number(),
});
export type ReadabilityStats = z.infer<typeof ReadabilityStatsSchema>;

export async function listAiAnalysisTypes(): Promise<string[]> {
  return z.array(z.string()).parse(await invoke("list_ai_analysis_types"));
}

export async function buildAiAnalysisContext(
  projectId: string,
  analysisType: string,
  characterId: string | null,
): Promise<ContextBlock[]> {
  return z
    .array(ContextBlockSchema)
    .parse(await invoke("build_ai_analysis_context", { projectId, analysisType, characterId }));
}

export async function computeReadabilityStats(projectId: string): Promise<ReadabilityStats> {
  return ReadabilityStatsSchema.parse(await invoke("compute_readability_stats", { projectId }));
}

export async function runAiAnalysis(
  projectId: string,
  analysisType: string,
  characterId: string | null,
  providerKind: string,
  model: string,
  baseUrl: string | null,
  temperature: number | null,
  maxOutputTokens: number | null,
  selectedContext: ContextBlock[],
): Promise<AiAnalysisReport> {
  return AiAnalysisReportSchema.parse(
    await invoke("run_ai_analysis", {
      projectId,
      analysisType,
      characterId,
      providerKind,
      model,
      baseUrl,
      temperature,
      maxOutputTokens,
      selectedContext,
    }),
  );
}

export async function listAiAnalysisReports(
  projectId: string,
  analysisType: string | null,
): Promise<AiAnalysisReport[]> {
  return z.array(AiAnalysisReportSchema).parse(await invoke("list_ai_analysis_reports", { projectId, analysisType }));
}

export async function deleteAiAnalysisReport(id: string): Promise<void> {
  await invoke("delete_ai_analysis_report", { id });
}

// ---------------------------------------------------------------------------
// Phase 8: Revision履歴 / Crash Recovery / Auto Backup / ゴミ箱
// ---------------------------------------------------------------------------

export const RevisionSchema = z.object({
  id: z.string(),
  owner_type: z.enum(["chapter", "scene"]),
  owner_id: z.string(),
  body: z.string(),
  char_count: z.number(),
  trigger: z.enum(["auto", "manual"]),
  label: nullableStr,
  created_at: z.string(),
});
export type Revision = z.infer<typeof RevisionSchema>;

export async function listRevisions(ownerType: "chapter" | "scene", ownerId: string): Promise<Revision[]> {
  return z.array(RevisionSchema).parse(await invoke("list_revisions", { ownerType, ownerId }));
}
export async function getRevision(id: string): Promise<Revision | null> {
  return RevisionSchema.nullable().parse(await invoke("get_revision", { id }));
}
export async function createManualRevision(
  ownerType: "chapter" | "scene",
  ownerId: string,
  label: string | null,
): Promise<Revision> {
  return RevisionSchema.parse(await invoke("create_manual_revision", { ownerType, ownerId, label }));
}
export async function restoreRevision(id: string): Promise<ManuscriptDocument> {
  return DocumentSchema.parse(await invoke("restore_revision", { id }));
}

export const TrashItemSchema = z.object({
  entity_type: z.string(),
  id: z.string(),
  project_id: z.string(),
  title: z.string(),
  deleted_at: z.string(),
});
export type TrashItem = z.infer<typeof TrashItemSchema>;

export const TRASH_ENTITY_TYPE_LABELS: Record<string, string> = {
  part: "パート",
  chapter: "章",
  scene: "シーン",
  character: "キャラクター",
  world_entry: "世界観項目",
  location: "場所",
  glossary_entry: "用語",
  note: "メモ",
  plot_card: "プロットカード",
  timeline_event: "時系列イベント",
  foreshadowing: "伏線",
  todo: "TODO",
};

export async function listTrash(projectId: string): Promise<TrashItem[]> {
  return z.array(TrashItemSchema).parse(await invoke("list_trash", { projectId }));
}
export async function restoreTrashItem(entityType: string, id: string): Promise<void> {
  await invoke("restore_trash_item", { entityType, id });
}
export async function purgeTrashItem(entityType: string, id: string): Promise<void> {
  await invoke("purge_trash_item", { entityType, id });
}
export async function purgeAllTrash(projectId: string): Promise<number> {
  return z.number().parse(await invoke("purge_all_trash", { projectId }));
}

export const BackupInfoSchema = z.object({
  filename: z.string(),
  created_at: z.string(),
  size_bytes: z.number(),
});
export type BackupInfo = z.infer<typeof BackupInfoSchema>;

export async function wasUncleanShutdown(): Promise<boolean> {
  return z.boolean().parse(await invoke("was_unclean_shutdown"));
}
export async function listBackups(): Promise<BackupInfo[]> {
  return z.array(BackupInfoSchema).parse(await invoke("list_backups"));
}
export async function backupNow(): Promise<BackupInfo> {
  return BackupInfoSchema.parse(await invoke("backup_now"));
}
export async function restoreBackup(filename: string): Promise<void> {
  await invoke("restore_backup", { filename });
}

// ---------------------------------------------------------------------------
// Phase 9: Export (TXT/Markdown/HTML/DOCX/PDF/EPUB)
// ---------------------------------------------------------------------------

export const EXPORT_FORMAT_LABELS: Record<string, string> = {
  txt: "テキスト (.txt)",
  markdown: "Markdown (.md)",
  html: "HTML (.html)",
  docx: "Word (.docx)",
  pdf: "PDF (簡易・横書き)",
  epub: "EPUB (.epub)",
};

export const ExportOutputSchema = z.object({
  filename: z.string(),
  mime_type: z.string(),
  bytes: z.array(z.number()),
});
export type ExportOutput = z.infer<typeof ExportOutputSchema>;

export async function listExportFormats(): Promise<string[]> {
  return z.array(z.string()).parse(await invoke("list_export_formats"));
}

export async function exportProject(projectId: string, format: string): Promise<ExportOutput> {
  return ExportOutputSchema.parse(await invoke("export_project", { projectId, format }));
}

// ---------------------------------------------------------------------------
// Phase 10: このアプリについて(About/License/Privacy/Changelog/診断ログ)
// ---------------------------------------------------------------------------

export async function getChangelog(): Promise<string> {
  return z.string().parse(await invoke("get_changelog"));
}
export async function getThirdPartyNotices(): Promise<string> {
  return z.string().parse(await invoke("get_third_party_notices"));
}
export async function getPrivacyNotice(): Promise<string> {
  return z.string().parse(await invoke("get_privacy_notice"));
}
export async function exportDiagnostics(): Promise<ExportOutput> {
  return ExportOutputSchema.parse(await invoke("export_diagnostics"));
}
