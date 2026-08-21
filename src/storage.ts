import { invoke } from "@tauri-apps/api/core";
import type { Note, NotesStore } from "./types";

const browserStorageKey = "mojian-notes";

function isRunningInTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

export async function loadStore(): Promise<NotesStore> {
  let store: NotesStore;
  if (isRunningInTauri()) {
    store = await invoke<NotesStore>("load_store");
  } else {
    const stored = localStorage.getItem(browserStorageKey);
    const parsed = stored ? (JSON.parse(stored) as NotesStore | Note[]) : [];
    store = Array.isArray(parsed)
      ? { knowledgeBases: [], notes: parsed }
      : parsed;
  }

  // Normalize optional fields here as the compatibility boundary. Stores made
  // by older releases can therefore use the current UI without a destructive
  // one-off migration.
  return {
    knowledgeBases: store.knowledgeBases ?? [],
    notes: (store.notes ?? []).map((note) => ({
      ...note,
      knowledgeBaseId: note.knowledgeBaseId ?? "",
      parentId: note.parentId ?? null,
      deletedAt: note.deletedAt ?? null,
      pinned: note.pinned ?? false,
      tags: note.tags ?? [],
    })),
  };
}

export async function saveStore(store: NotesStore): Promise<void> {
  if (isRunningInTauri()) {
    await invoke("save_store", { store });
    return;
  }

  localStorage.setItem(browserStorageKey, JSON.stringify(store));
}
