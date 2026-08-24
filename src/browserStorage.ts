export const browserStorageKeys = {
  settings: "orange-run-notes-settings",
  mcpEnabled: "orange-run-notes-mcp-enabled",
  notes: "orange-run-notes-notes",
  documentAiModels: "orange-run-notes-document-ai-models-v1",
  sidebarCollapsed: "orange-run-notes-sidebar-collapsed",
  libraryRailCollapsed: "orange-run-notes-library-collapsed",
  noteListMode: "orange-run-notes-note-list-mode",
  sidebarWidth: "orange-run-notes-sidebar-width-v3",
  aiPanelWidth: "orange-run-notes-ai-panel-width-v2",
} as const;

const legacyKeys: Partial<Record<string, readonly string[]>> = {
  [browserStorageKeys.settings]: ["mojian-settings"],
  [browserStorageKeys.mcpEnabled]: ["mojian-mcp-enabled"],
  [browserStorageKeys.notes]: ["mojian-notes"],
  [browserStorageKeys.documentAiModels]: ["orange-run-document-ai-model-v1"],
  [browserStorageKeys.sidebarCollapsed]: ["orange-run-sidebar-collapsed"],
  [browserStorageKeys.libraryRailCollapsed]: ["orange-run-library-collapsed"],
  [browserStorageKeys.noteListMode]: ["orange-run-note-list-mode"],
  [browserStorageKeys.sidebarWidth]: ["orange-run-sidebar-width-v3"],
  [browserStorageKeys.aiPanelWidth]: ["orange-run-ai-panel-width-v2"],
};

/** Read once from a legacy key, then move the value to the unified namespace. */
export function readBrowserStorage(key: string): string | null {
  const current = localStorage.getItem(key);
  if (current !== null) return current;

  for (const legacyKey of legacyKeys[key] ?? []) {
    const value = localStorage.getItem(legacyKey);
    if (value === null) continue;
    localStorage.setItem(key, value);
    localStorage.removeItem(legacyKey);
    return value;
  }
  return null;
}

export function writeBrowserStorage(key: string, value: string): void {
  localStorage.setItem(key, value);
  for (const legacyKey of legacyKeys[key] ?? []) localStorage.removeItem(legacyKey);
}
