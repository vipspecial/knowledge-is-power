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

export type EditorMode = "edit" | "split" | "preview";

export interface GeneralSettings {
  autoSaveDelayMs: number;
  defaultEditorMode: EditorMode;
}

export type AiProtocol = "chatCompletions" | "responses";

export interface AiSettings {
  enabled: boolean;
  baseUrl: string;
  protocol: AiProtocol;
  model: string;
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

export interface AiRequest {
  documentId: string;
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
