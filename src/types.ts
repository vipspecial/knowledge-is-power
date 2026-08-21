export interface Note {
  id: string;
  title: string;
  content: string;
  knowledgeBaseId: string;
  parentId: string | null;
  deletedAt: string | null;
  pinned: boolean;
  tags: string[];
  createdAt: string;
  updatedAt: string;
}

export type NoteListMode = "cards" | "outline";

export interface KnowledgeBase {
  id: string;
  name: string;
  createdAt: string;
}

export interface NotesStore {
  knowledgeBases: KnowledgeBase[];
  notes: Note[];
}

export type SaveState = "idle" | "saving" | "saved" | "error";

export interface GeneralSettings {
  autoSaveDelayMs: number;
}

export type AiProtocol = "chatCompletions" | "responses" | "anthropic";

export interface AiSettings {
  enabled: boolean;
  provider: string;
  baseUrl: string;
  protocol: AiProtocol;
  model: string;
  models: string[];
  temperature: number;
  maxContextChars: number;
}

export interface AppSettings {
  general: GeneralSettings;
  ai: AiSettings;
  documentDirectory: string;
}

export interface SettingsView {
  settings: AppSettings;
  hasApiKey: boolean;
}

export type AiOperation =
  | "chat"
  | "write"
  | "summarize"
  | "polish"
  | "shorten"
  | "expand"
  | "continue"
  | "outline"
  | "proofread"
  | "brainstorm"
  | "explain"
  | "translate"
  | "todos"
  | "title"
  | "tags";

export type AiContentTarget = "selection" | "append" | "document" | "insert";

export interface AiTextRange {
  from: number;
  to: number;
}

/** A scene-specific AI request dispatched to the document conversation panel. */
export interface AiPanelTask {
  id: string;
  document: Note;
  operation: AiOperation;
  label: string;
  prompt: string;
  selection: string;
  target: Exclude<AiContentTarget, "insert">;
  range?: AiTextRange;
}

export interface AiApplyPayload {
  documentId: string;
  content: string;
  target: AiContentTarget;
  range?: AiTextRange;
}

export interface AiRequest {
  documentId: string;
  model: string;
  operation: AiOperation;
  prompt: string;
  selection: string;
  noteTitle: string;
  noteContent: string;
}

export type AiStreamEvent =
  | { event: "started" }
  | { event: "delta"; content: string }
  | { event: "done" }
  | { event: "error"; message: string };
