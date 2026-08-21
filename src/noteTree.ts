import type { Note } from "./types";

export interface NoteTreeRow {
  note: Note;
  depth: number;
  hasChildren: boolean;
}

export function compareNotes(a: Note, b: Note): number {
  if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
  return new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime();
}

function safeParentIds(notes: Note[]): Map<string, string | null> {
  const byId = new Map(notes.map((note) => [note.id, note]));
  const result = new Map<string, string | null>();

  for (const note of notes) {
    const parentId = note.parentId;
    if (!parentId || parentId === note.id || !byId.has(parentId)) {
      result.set(note.id, null);
      continue;
    }

    // Imported or manually edited Markdown may contain a parent cycle. Treat the
    // affected note as a root so rendering never recurses forever.
    const seen = new Set([note.id]);
    let cursor: Note | undefined = byId.get(parentId);
    let cyclic = false;
    while (cursor?.parentId) {
      if (seen.has(cursor.id)) {
        cyclic = true;
        break;
      }
      seen.add(cursor.id);
      cursor = byId.get(cursor.parentId);
    }
    result.set(note.id, cyclic ? null : parentId);
  }
  return result;
}

export function flattenNoteTree(
  notes: Note[],
  collapsedIds: ReadonlySet<string> = new Set(),
): NoteTreeRow[] {
  const parents = safeParentIds(notes);
  const children = new Map<string | null, Note[]>();
  for (const note of notes) {
    const parentId = parents.get(note.id) ?? null;
    const siblings = children.get(parentId) ?? [];
    siblings.push(note);
    children.set(parentId, siblings);
  }
  for (const siblings of children.values()) siblings.sort(compareNotes);

  const rows: NoteTreeRow[] = [];
  const append = (note: Note, depth: number): void => {
    const nested = children.get(note.id) ?? [];
    rows.push({ note, depth, hasChildren: nested.length > 0 });
    if (!collapsedIds.has(note.id)) {
      for (const child of nested) append(child, depth + 1);
    }
  };
  for (const root of children.get(null) ?? []) append(root, 0);
  return rows;
}

export function notesMatchingQuery(notes: Note[], query: string): Note[] {
  const normalized = query.trim().toLocaleLowerCase();
  if (!normalized) return notes;
  const byId = new Map(notes.map((note) => [note.id, note]));
  const included = new Set<string>();

  for (const note of notes) {
    const haystack = `${note.title}\n${note.content}\n${note.tags.join(" ")}`.toLocaleLowerCase();
    if (!haystack.includes(normalized)) continue;
    // Keep every ancestor of a hit so an outline search result remains
    // understandable instead of showing an orphaned child.
    let cursor: Note | undefined = note;
    while (cursor && !included.has(cursor.id)) {
      included.add(cursor.id);
      cursor = cursor.parentId ? byId.get(cursor.parentId) : undefined;
    }
  }
  return notes.filter((note) => included.has(note.id));
}
