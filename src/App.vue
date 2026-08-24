<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { createDocumentAiRequest, streamAi } from "./ai";
import AiPanel from "./components/AiPanel.vue";
import AiWritingDialog from "./components/AiWritingDialog.vue";
import AppContextMenu from "./components/AppContextMenu.vue";
import GlobalSearchDialog from "./components/GlobalSearchDialog.vue";
import KnowledgeRail from "./components/KnowledgeRail.vue";
import NoteListPane from "./components/NoteListPane.vue";
import RichTextEditor from "./components/RichTextEditor.vue";
import SettingsDialog from "./components/SettingsDialog.vue";
import TrashPane from "./components/TrashPane.vue";
import {
  cloneAppSettings,
  defaultSettings,
  loadAppSettings,
  saveAppSettings,
} from "./settings";
import { loadStore, saveStore } from "./storage";
import type {
  AppSettings,
  AiApplyPayload,
  AiOperation,
  AiPanelTask,
  KnowledgeBase,
  Note,
  NoteListMode,
  NotesStore,
  SaveState,
} from "./types";

interface ImportedMarkdown {
  title: string;
  content: string;
}

const notes = ref<Note[]>([]);
const knowledgeBases = ref<KnowledgeBase[]>([]);
const selectedKnowledgeBaseId = ref<string | null>(null);
const selectedId = ref<string | null>(null);
const searchQuery = ref("");
const isLoading = ref(true);
const saveState = ref<SaveState>("idle");
const errorMessage = ref("");
const toastMessage = ref("");
const showDeleteDialog = ref(false);
const trashDeleteTarget = ref<"all" | string | null>(null);
const showSettingsDialog = ref(false);
const showGlobalSearchDialog = ref(false);
const settingsInitialTab = ref<"general" | "ai" | "storage" | "mcp" | "about">("general");
const savingSettings = ref(false);
const settings = ref<AppSettings>(cloneAppSettings(defaultSettings));
const hasApiKey = ref(false);
const aiPanelOpen = ref(false);
const showAiWritingDialog = ref(false);
const sidebarCollapsed = ref(localStorage.getItem("orange-run-sidebar-collapsed") === "true");
const libraryRailCollapsed = ref(localStorage.getItem("orange-run-library-collapsed") === "true");
const activeNavigation = ref<"library" | "trash">("library");
const noteListMode = ref<NoteListMode>(localStorage.getItem("orange-run-note-list-mode") === "outline" ? "outline" : "cards");
const collapsedNoteIds = ref<Set<string>>(new Set());
const documentMenuOpen = ref(false);
const editorAiMenuOpen = ref(false);
const selectedText = ref("");
const selectionRange = ref({ start: 0, end: 0 });
const knowledgeBaseDialog = ref<"create" | "rename" | "delete" | null>(null);
const knowledgeBaseName = ref("");
const titleInput = ref<HTMLInputElement | null>(null);
const noteListPane = ref<InstanceType<typeof NoteListPane> | null>(null);
const aiPanel = ref<InstanceType<typeof AiPanel> | null>(null);
const richTextEditor = ref<InstanceType<typeof RichTextEditor> | null>(null);
const sidebarWidth = ref(clamp(readStoredNumber("orange-run-sidebar-width-v3", 264), 220, 380));
const aiPanelWidth = ref(clamp(readStoredNumber("orange-run-ai-panel-width-v2", 330), 300, 480));
const metadataAiBusy = ref<"title" | "tags" | null>(null);
const contextMenu = ref<ContextMenuState | null>(null);
const isMacOsDesktop =
  "__TAURI_INTERNALS__" in window && /Macintosh|Mac OS X/.test(navigator.userAgent);

type ResizeTarget = "sidebar" | "ai";
type ContextMenuKind = "knowledgeBase" | "note";

interface ContextMenuState {
  kind: ContextMenuKind;
  id: string;
  x: number;
  y: number;
}

interface ContextMenuItem {
  id: string;
  label: string;
  icon?: string;
  danger?: boolean;
  disabled?: boolean;
  separatorBefore?: boolean;
}

interface ResizeState {
  target: ResizeTarget;
  startX: number;
  startValue: number;
}

let saveTimer: ReturnType<typeof setTimeout> | undefined;
let toastTimer: ReturnType<typeof setTimeout> | undefined;
let unlistenTrayToast: (() => void) | null = null;
let hydrated = false;
let resizeState: ResizeState | null = null;
let metadataAiVersion = 0;

const layoutStyle = computed(() => ({
  "--sidebar-width": `${sidebarWidth.value}px`,
  "--ai-width": `${aiPanelWidth.value}px`,
}));

function toggleSidebar(): void {
  sidebarCollapsed.value = !sidebarCollapsed.value;
  localStorage.setItem("orange-run-sidebar-collapsed", String(sidebarCollapsed.value));
}

function toggleLibraryRail(): void {
  libraryRailCollapsed.value = !libraryRailCollapsed.value;
  localStorage.setItem("orange-run-library-collapsed", String(libraryRailCollapsed.value));
}

function openNavigation(): void {
  libraryRailCollapsed.value = false;
  sidebarCollapsed.value = false;
  localStorage.setItem("orange-run-library-collapsed", "false");
  localStorage.setItem("orange-run-sidebar-collapsed", "false");
}

function openTrash(): void {
  activeNavigation.value = "trash";
  sidebarCollapsed.value = false;
  aiPanelOpen.value = false;
  selectedId.value = null;
  searchQuery.value = "";
  localStorage.setItem("orange-run-sidebar-collapsed", "false");
}

function closeTrash(): void {
  const baseId = selectedKnowledgeBaseId.value ?? knowledgeBases.value[0]?.id;
  if (baseId) selectKnowledgeBase(baseId);
}

function setNoteListMode(mode: NoteListMode): void {
  noteListMode.value = mode;
  localStorage.setItem("orange-run-note-list-mode", mode);
}

function toggleNoteBranch(id: string): void {
  const next = new Set(collapsedNoteIds.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  collapsedNoteIds.value = next;
}

function closeMenus(): void {
  documentMenuOpen.value = false;
  editorAiMenuOpen.value = false;
  contextMenu.value = null;
}

function openContextMenu(kind: ContextMenuKind, id: string, event: MouseEvent): void {
  closeMenus();
  if (kind === "note") selectNote(id);

  const menuWidth = 184;
  const estimatedHeight = 98;
  contextMenu.value = {
    kind,
    id,
    x: clamp(event.clientX, 8, Math.max(8, window.innerWidth - menuWidth - 8)),
    y: clamp(event.clientY, 8, Math.max(8, window.innerHeight - estimatedHeight - 8)),
  };
}

function openKnowledgeBaseContextMenu(id: string, event: MouseEvent): void {
  openContextMenu("knowledgeBase", id, event);
}

function openNoteContextMenu(id: string, event: MouseEvent): void {
  openContextMenu("note", id, event);
}

/** Keep native editing commands only where copy, paste or link actions are useful. */
function handleAppContextMenu(event: MouseEvent): void {
  const target = event.target instanceof Element ? event.target : null;
  const editable = target?.closest("input, textarea, [contenteditable='true']");

  closeMenus();
  if (!editable) event.preventDefault();
}

function toggleMenu(menu: "document" | "editorAi"): void {
  const next = menu === "document"
      ? !documentMenuOpen.value
      : !editorAiMenuOpen.value;
  closeMenus();
  if (menu === "document") documentMenuOpen.value = next;
  else editorAiMenuOpen.value = next;
}

function readStoredNumber(key: string, fallback: number): number {
  const value = Number(localStorage.getItem(key));
  return Number.isFinite(value) && value > 0 ? value : fallback;
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(Math.max(value, minimum), maximum);
}

function maximumSidebarWidth(): number {
  const reserved = aiPanelOpen.value && window.innerWidth > 1280
    ? aiPanelWidth.value + 700
    : 700;
  return Math.max(220, Math.min(380, window.innerWidth - reserved));
}

function maximumAiPanelWidth(): number {
  const reserved = window.innerWidth > 1280 ? sidebarWidth.value + 700 : 300;
  return Math.max(300, Math.min(480, window.innerWidth - reserved));
}

function persistPanelWidths(): void {
  localStorage.setItem("orange-run-sidebar-width-v3", String(Math.round(sidebarWidth.value)));
  localStorage.setItem("orange-run-ai-panel-width-v2", String(Math.round(aiPanelWidth.value)));
}

function startPanelResize(target: ResizeTarget, event: PointerEvent): void {
  event.preventDefault();
  resizeState = {
    target,
    startX: event.clientX,
    startValue: target === "sidebar" ? sidebarWidth.value : aiPanelWidth.value,
  };
  document.body.classList.add("panel-resizing");
  window.addEventListener("pointermove", handlePanelResize);
  window.addEventListener("pointerup", stopPanelResize, { once: true });
}

function handlePanelResize(event: PointerEvent): void {
  if (!resizeState) return;
  if (resizeState.target === "sidebar") {
    sidebarWidth.value = clamp(
      resizeState.startValue + event.clientX - resizeState.startX,
      220,
      maximumSidebarWidth(),
    );
  } else {
    aiPanelWidth.value = clamp(
      resizeState.startValue - (event.clientX - resizeState.startX),
      300,
      maximumAiPanelWidth(),
    );
  }
}

function stopPanelResize(): void {
  resizeState = null;
  document.body.classList.remove("panel-resizing");
  window.removeEventListener("pointermove", handlePanelResize);
  persistPanelWidths();
}

function nudgePanel(target: ResizeTarget, amount: number): void {
  if (target === "sidebar") {
    sidebarWidth.value = clamp(sidebarWidth.value + amount, 220, maximumSidebarWidth());
  } else {
    aiPanelWidth.value = clamp(aiPanelWidth.value + amount, 300, maximumAiPanelWidth());
  }
  persistPanelWidths();
}

function fitPanelsToWindow(): void {
  sidebarWidth.value = clamp(sidebarWidth.value, 220, maximumSidebarWidth());
  aiPanelWidth.value = clamp(aiPanelWidth.value, 300, maximumAiPanelWidth());
}

const selectedKnowledgeBase = computed(() =>
  knowledgeBases.value.find((base) => base.id === selectedKnowledgeBaseId.value),
);

const selectedNote = computed(() =>
  notes.value.find((note) => note.id === selectedId.value && !note.deletedAt),
);

const contextMenuItems = computed<ContextMenuItem[]>(() => {
  const menu = contextMenu.value;
  if (!menu) return [];
  if (menu.kind === "knowledgeBase") {
    // 打开/新建已在栏内直接可点，右键只保留栏内不可见的操作。
    return [
      { id: "rename", label: "重命名", icon: "✎" },
      {
        id: "delete",
        label: "删除知识库",
        icon: "×",
        danger: true,
        disabled: knowledgeBases.value.length <= 1,
      },
    ];
  }

  // 回收站项上已有恢复/删除按钮，不再提供右键菜单。
  return [
    { id: "newChild", label: "新建子文档", icon: "+" },
    { id: "trash", label: "移到回收站", icon: "×", danger: true, separatorBefore: true },
  ];
});

const storeSnapshot = computed<NotesStore>(() => ({
  knowledgeBases: knowledgeBases.value,
  notes: notes.value,
}));

const sortedNotes = computed(() =>
  notes.value
    .filter((note) => note.knowledgeBaseId === selectedKnowledgeBaseId.value && !note.deletedAt)
    .sort((a, b) => {
    if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
    return new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime();
    }),
);

const trashedNotes = computed(() =>
  notes.value
    .filter((note) => Boolean(note.deletedAt))
    .sort((a, b) => new Date(b.deletedAt ?? 0).getTime() - new Date(a.deletedAt ?? 0).getTime()),
);

const characterCount = computed(() => selectedNote.value?.content.trim().length ?? 0);
const shortcutPrefix = /Mac|iPhone|iPad/.test(navigator.platform) ? "⌘" : "Ctrl+";

function createId(): string {
  return crypto.randomUUID?.() ?? `${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function makeNote(title = "无标题笔记", content = "", parentId: string | null = null): Note {
  const now = new Date().toISOString();
  return {
    id: createId(),
    title,
    content,
    knowledgeBaseId: selectedKnowledgeBaseId.value ?? "",
    parentId,
    deletedAt: null,
    pinned: false,
    tags: [],
    createdAt: now,
    updatedAt: now,
  };
}

function makeKnowledgeBase(name: string): KnowledgeBase {
  return {
    id: createId(),
    name: name.trim(),
    createdAt: new Date().toISOString(),
  };
}

function knowledgeBaseNoteCount(id: string): number {
  return notes.value.filter((note) => note.knowledgeBaseId === id && !note.deletedAt).length;
}

function selectKnowledgeBase(id: string): void {
  activeNavigation.value = "library";
  selectedKnowledgeBaseId.value = id;
  selectedId.value = sortedNotes.value[0]?.id ?? null;
  selectedText.value = "";
  searchQuery.value = "";
}

function openCreateKnowledgeBase(): void {
  knowledgeBaseName.value = "";
  knowledgeBaseDialog.value = "create";
  nextTick(() => document.querySelector<HTMLInputElement>("#knowledge-base-name")?.focus());
}

function openRenameKnowledgeBase(): void {
  if (!selectedKnowledgeBase.value) return;
  knowledgeBaseName.value = selectedKnowledgeBase.value.name;
  knowledgeBaseDialog.value = "rename";
  nextTick(() => document.querySelector<HTMLInputElement>("#knowledge-base-name")?.select());
}

function confirmKnowledgeBaseName(): void {
  const name = knowledgeBaseName.value.trim();
  if (!name) return;
  if (knowledgeBaseDialog.value === "create") {
    const base = makeKnowledgeBase(name);
    knowledgeBases.value.push(base);
    selectKnowledgeBase(base.id);
    showToast("知识库已创建");
  } else if (knowledgeBaseDialog.value === "rename" && selectedKnowledgeBase.value) {
    selectedKnowledgeBase.value.name = name;
    showToast("知识库已重命名");
  }
  knowledgeBaseDialog.value = null;
}

function openDeleteKnowledgeBase(): void {
  if (knowledgeBases.value.length === 1) {
    showToast("至少需要保留一个知识库");
    return;
  }
  knowledgeBaseDialog.value = "delete";
}

function deleteKnowledgeBase(): void {
  const baseId = selectedKnowledgeBaseId.value;
  if (!baseId || knowledgeBases.value.length === 1) return;
  notes.value = notes.value.filter((note) => note.knowledgeBaseId !== baseId);
  knowledgeBases.value = knowledgeBases.value.filter((base) => base.id !== baseId);
  selectKnowledgeBase(knowledgeBases.value[0].id);
  knowledgeBaseDialog.value = null;
  showToast("知识库及其中笔记已删除");
}

function closeKnowledgeBaseDialog(): void {
  knowledgeBaseDialog.value = null;
}

function addNote(parentId: string | null = null): void {
  const validParent = parentId
    ? notes.value.find((note) => note.id === parentId && note.knowledgeBaseId === selectedKnowledgeBaseId.value)
    : undefined;
  const note = makeNote("无标题笔记", "", validParent?.id ?? null);
  notes.value.push(note);
  if (validParent) {
    const next = new Set(collapsedNoteIds.value);
    next.delete(validParent.id);
    collapsedNoteIds.value = next;
  }
  selectedId.value = note.id;
  searchQuery.value = "";
  nextTick(() => titleInput.value?.select());
}

function addChildNote(parentId: string): void {
  addNote(parentId);
  showToast("已创建子文档");
}

function selectNote(id: string): void {
  selectedId.value = id;
  selectedText.value = "";
}

function openGlobalSearch(): void {
  closeMenus();
  showGlobalSearchDialog.value = true;
}

function startWindowDrag(event: PointerEvent): void {
  if (event.button !== 0 || !("__TAURI_INTERNALS__" in window)) return;
  // The second press belongs to a double-click; do not start another native drag.
  if (event.detail > 1) {
    event.preventDefault();
    return;
  }
  event.preventDefault();
  void import("@tauri-apps/api/window")
    .then(({ getCurrentWindow }) => getCurrentWindow().startDragging())
    .catch(() => undefined);
}

function toggleWindowMaximize(event: MouseEvent): void {
  if (event.button !== 0 || !("__TAURI_INTERNALS__" in window)) return;
  event.preventDefault();
  void import("@tauri-apps/api/window")
    .then(({ getCurrentWindow }) => getCurrentWindow().toggleMaximize())
    .catch(() => undefined);
}

function revealGlobalSearchResult(knowledgeBaseId: string, noteId: string): void {
  const note = notes.value.find((item) => item.id === noteId && !item.deletedAt);
  if (!note || note.knowledgeBaseId !== knowledgeBaseId) return;

  activeNavigation.value = "library";
  selectedKnowledgeBaseId.value = knowledgeBaseId;
  selectedId.value = noteId;
  searchQuery.value = "";
  selectedText.value = "";
  showGlobalSearchDialog.value = false;
  libraryRailCollapsed.value = false;
  sidebarCollapsed.value = false;
  localStorage.setItem("orange-run-library-collapsed", "false");
  localStorage.setItem("orange-run-sidebar-collapsed", "false");

  const nextCollapsed = new Set(collapsedNoteIds.value);
  let parentId = note.parentId;
  const visited = new Set<string>();
  while (parentId && !visited.has(parentId)) {
    visited.add(parentId);
    nextCollapsed.delete(parentId);
    parentId = notes.value.find((item) => item.id === parentId)?.parentId ?? null;
  }
  collapsedNoteIds.value = nextCollapsed;
}

function markEdited(): void {
  if (selectedNote.value) selectedNote.value.updatedAt = new Date().toISOString();
}

function togglePin(): void {
  if (!selectedNote.value) return;
  selectedNote.value.pinned = !selectedNote.value.pinned;
  markEdited();
  showToast(selectedNote.value.pinned ? "笔记已置顶" : "已取消置顶");
}

function duplicateNote(): void {
  if (!selectedNote.value) return;
  const duplicated = makeNote(
    `${displayTitle(selectedNote.value)} 副本`,
    selectedNote.value.content,
    selectedNote.value.parentId,
  );
  duplicated.tags = [...selectedNote.value.tags];
  notes.value.push(duplicated);
  selectedId.value = duplicated.id;
  showToast("已创建笔记副本");
}

function updateTags(event: Event): void {
  if (!selectedNote.value) return;
  const input = event.target as HTMLInputElement;
  selectedNote.value.tags = [
    ...new Set(
      input.value
        .split(/[,，]/)
        .map((tag) => tag.trim())
        .filter(Boolean),
    ),
  ].slice(0, 8);
  markEdited();
}

function requestDelete(): void {
  if (selectedNote.value) showDeleteDialog.value = true;
}

/** Collects a document subtree iteratively so deeply nested notes cannot overflow the call stack. */
function documentSubtreeIds(rootId: string): Set<string> {
  const ids = new Set([rootId]);
  let changed = true;
  while (changed) {
    changed = false;
    for (const note of notes.value) {
      if (note.parentId && ids.has(note.parentId) && !ids.has(note.id)) {
        ids.add(note.id);
        changed = true;
      }
    }
  }
  return ids;
}

function deleteSelectedNote(): void {
  if (!selectedId.value) return;
  const ids = documentSubtreeIds(selectedId.value);
  const deletedAt = new Date().toISOString();
  for (const note of notes.value) {
    if (ids.has(note.id)) note.deletedAt = deletedAt;
  }
  selectedId.value = sortedNotes.value[0]?.id ?? null;
  showDeleteDialog.value = false;
  showToast(ids.size > 1 ? `文档及 ${ids.size - 1} 篇子文档已移入回收站` : "文档已移入回收站");
}

function restoreTrashedNote(id: string): void {
  const ids = documentSubtreeIds(id);
  for (const note of notes.value) {
    if (ids.has(note.id)) note.deletedAt = null;
  }
  const restored = notes.value.find((note) => note.id === id);
  if (restored?.parentId && notes.value.some((note) => note.id === restored.parentId && note.deletedAt)) {
    restored.parentId = null;
  }
  showToast(ids.size > 1 ? `已恢复 ${ids.size} 篇文档` : "文档已恢复");
}

function requestPermanentDelete(target: "all" | string): void {
  trashDeleteTarget.value = target;
}

function handleContextMenuAction(action: string): void {
  const menu = contextMenu.value;
  if (!menu) return;
  contextMenu.value = null;

  if (menu.kind === "knowledgeBase") {
    selectKnowledgeBase(menu.id);
    if (action === "rename") openRenameKnowledgeBase();
    else if (action === "delete") openDeleteKnowledgeBase();
    return;
  }

  selectNote(menu.id);
  if (action === "newChild") addChildNote(menu.id);
  else if (action === "trash") requestDelete();
}

function confirmPermanentDelete(): void {
  if (!trashDeleteTarget.value) return;
  if (trashDeleteTarget.value === "all") {
    notes.value = notes.value.filter((note) => !note.deletedAt);
    showToast("回收站已清空");
  } else {
    const ids = documentSubtreeIds(trashDeleteTarget.value);
    notes.value = notes.value.filter((note) => !ids.has(note.id));
    showToast("文档已永久删除");
  }
  trashDeleteTarget.value = null;
}

function cancelDelete(): void {
  showDeleteDialog.value = false;
}

function displayTitle(note: Note): string {
  return note.title.trim() || note.content.trim().split("\n")[0].replace(/^#+\s*/, "") || "无标题笔记";
}

function formatFullDate(isoDate: string): string {
  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(isoDate));
}

function showToast(message: string): void {
  toastMessage.value = message;
  window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => (toastMessage.value = ""), 2400);
}

function openSettings(tab: "general" | "ai" | "storage" | "mcp" | "about" = "general"): void {
  settingsInitialTab.value = tab;
  showSettingsDialog.value = true;
}

function updateSelection(selection: { text: string; from: number; to: number }): void {
  selectedText.value = selection.text;
  selectionRange.value = { start: selection.from, end: selection.to };
}

function ensureAiReady(): boolean {
  const ai = settings.value.ai;
  const missing: string[] = [];
  if (!ai.enabled) missing.push("启用 AI 助手");
  if (!ai.baseUrl.trim()) missing.push("接口地址");
  if (!ai.model.trim()) missing.push("模型");
  // 自定义/本地服务（如 Ollama）可以无 Key，其余服务商都需要。
  if (ai.provider !== "custom" && !hasApiKey.value) missing.push("API Key");
  if (missing.length === 0) return true;
  openSettings("ai");
  showToast(`请先在 AI 设置中完成配置：${missing.join("、")}`);
  return false;
}

function toggleAiPanel(): void {
  if (!aiPanelOpen.value && !ensureAiReady()) return;
  aiPanelOpen.value = !aiPanelOpen.value;
}

function openAiWriting(): void {
  if (!ensureAiReady()) return;
  showAiWritingDialog.value = true;
}

function runContextualAi(
  operation: AiOperation,
  label: string,
  target: "selection" | "append" | "document",
  prompt = "",
): void {
  const note = selectedNote.value;
  if (!note || !ensureAiReady()) return;
  if (target === "selection" && !selectedText.value.trim()) {
    showToast("请先选择要处理的文字");
    return;
  }

  const task: AiPanelTask = {
    id: createId(),
    document: { ...note, tags: [...note.tags] },
    operation,
    label,
    prompt,
    selection: target === "selection" ? selectedText.value : "",
    target,
    range: target === "selection"
      ? { from: selectionRange.value.start, to: selectionRange.value.end }
      : undefined,
  };
  aiPanelOpen.value = true;
  // All contextual entries share one queue. Waiting for the panel render also
  // prevents the first task from being lost when the action opens the panel.
  void nextTick(() => aiPanel.value?.acceptTask(task));
}

function handleSelectionAi(action: {
  operation: AiOperation;
  label: string;
  prompt: string;
  text: string;
  from: number;
  to: number;
}): void {
  updateSelection({ text: action.text, from: action.from, to: action.to });
  runContextualAi(action.operation, action.label, "selection", action.prompt);
}

function applyAiPanelResult(payload: AiApplyPayload): void {
  const note = selectedNote.value;
  if (!note || note.id !== payload.documentId) {
    showToast("请切换回对应文章后再应用结果");
    return;
  }
  const currentEditor = richTextEditor.value;
  if (!currentEditor) {
    showToast("编辑器尚未就绪，请稍后重试");
    return;
  }
  if (payload.target === "selection" && payload.range) {
    currentEditor.replaceRange(payload.range.from, payload.range.to, payload.content);
    showToast("已替换原选区");
  } else if (payload.target === "document") {
    currentEditor.replaceDocument(payload.content);
    showToast("已替换当前文章正文");
  } else if (payload.target === "append") {
    currentEditor.appendMarkdown(payload.content);
    showToast("已追加到当前文章");
  } else {
    const position = selectionRange.value.end;
    if (position > 0) currentEditor.replaceRange(position, position, payload.content);
    else currentEditor.appendMarkdown(payload.content);
    showToast("AI 内容已插入正文");
  }
  selectedText.value = "";
  selectionRange.value = { start: 0, end: 0 };
}

function parseAiTags(content: string): string[] {
  return content
    .replace(/[#*`]/g, "")
    .split(/[,，\n、]/)
    .map((tag) => tag.trim().replace(/^[-\d.)\s]+/, ""))
    .filter(Boolean)
    .slice(0, 8);
}

async function generateMetadata(operation: "title" | "tags"): Promise<void> {
  const note = selectedNote.value;
  if (!note || metadataAiBusy.value || !ensureAiReady()) return;
  if (!note.content.trim()) {
    showToast("先写一些正文，AI 才能生成内容");
    return;
  }
  const sourceDocumentId = note.id;
  const version = ++metadataAiVersion;
  metadataAiBusy.value = operation;
  let output = "";
  let streamError = "";
  try {
    await streamAi(createDocumentAiRequest(note, operation), (event) => {
      if (version !== metadataAiVersion || selectedId.value !== sourceDocumentId) return;
      if (event.event === "delta") output += event.content;
      if (event.event === "error") streamError = event.message;
    });
    if (version !== metadataAiVersion || selectedId.value !== sourceDocumentId) return;
    if (streamError) throw new Error(streamError);
    if (!output.trim()) throw new Error("模型没有返回内容");
    if (operation === "title") {
      applyAiTitle(output);
    } else {
      applyAiTags(parseAiTags(output));
    }
  } catch (error) {
    showToast(`AI 生成失败：${String(error)}`);
  } finally {
    if (version === metadataAiVersion) metadataAiBusy.value = null;
  }
}

function appendDocumentAiContent(content: string): void {
  if (!selectedNote.value || !richTextEditor.value) return;
  richTextEditor.value.appendMarkdown(content.trim());
  showToast("AI 草稿已追加到当前文章");
}

function replaceDocumentAiContent(content: string): void {
  if (!selectedNote.value || !richTextEditor.value) return;
  richTextEditor.value.replaceDocument(content.trim());
  showToast("当前文章正文已更新");
}

function applyAiTags(tags: string[]): void {
  if (!selectedNote.value) return;
  selectedNote.value.tags = [...new Set([...selectedNote.value.tags, ...tags])].slice(0, 8);
  markEdited();
  showToast("AI 标签已应用");
}

function applyAiTitle(title: string): void {
  if (!selectedNote.value) return;
  selectedNote.value.title = title.replace(/^#+\s*/, "").replace(/[“”"]/g, "").trim();
  markEdited();
  showToast("AI 标题已应用");
}

async function saveSettingsFromDialog(
  updatedSettings: AppSettings,
  apiKey: string,
): Promise<void> {
  savingSettings.value = true;
  try {
    const view = await saveAppSettings(updatedSettings, apiKey);
    settings.value = view.settings;
    hasApiKey.value = view.hasApiKey;
    showSettingsDialog.value = false;
    showToast("设置已保存");
  } catch (error) {
    showToast(`设置保存失败：${String(error)}`);
  } finally {
    savingSettings.value = false;
  }
}

function handleDirectoryChanged(path: string): void {
  settings.value.documentDirectory = path;
  showToast("文档已复制到新目录");
}

async function importMarkdown(): Promise<void> {
  if (!("__TAURI_INTERNALS__" in window)) {
    showToast("请在桌面应用中使用导入功能");
    return;
  }
  try {
    const imported = await invoke<ImportedMarkdown | null>("import_markdown");
    if (!imported) return;
    const note = makeNote(imported.title, imported.content);
    notes.value.push(note);
    selectedId.value = note.id;
    showToast("Markdown 已导入");
  } catch (error) {
    showToast(`导入失败：${String(error)}`);
  }
}

async function exportMarkdown(): Promise<void> {
  if (!selectedNote.value || !("__TAURI_INTERNALS__" in window)) {
    showToast("请在桌面应用中使用导出功能");
    return;
  }
  try {
    const path = await invoke<string | null>("export_markdown", { note: selectedNote.value });
    if (path) showToast("Markdown 已导出");
  } catch (error) {
    showToast(`导出失败：${String(error)}`);
  }
}

async function persistNotes(): Promise<void> {
  saveState.value = "saving";
  errorMessage.value = "";
  try {
    await saveStore({
      knowledgeBases: knowledgeBases.value,
      notes: notes.value,
    });
    saveState.value = "saved";
  } catch (error) {
    saveState.value = "error";
    errorMessage.value = error instanceof Error ? error.message : String(error);
  }
}

function handleKeydown(event: KeyboardEvent): void {
  const modifier = event.metaKey || event.ctrlKey;
  if (modifier && event.key.toLocaleLowerCase() === "k") {
    event.preventDefault();
    openGlobalSearch();
    return;
  }
  if (modifier && event.key.toLocaleLowerCase() === "n") {
    event.preventDefault();
    addNote();
  }
  if (modifier && event.key.toLocaleLowerCase() === "f") {
    event.preventDefault();
    noteListPane.value?.focusSearch();
  }
  if (modifier && event.key.toLocaleLowerCase() === "s") {
    event.preventDefault();
    void persistNotes();
  }
  if (modifier && event.key === ",") {
    event.preventDefault();
    openSettings("general");
  }
  if (modifier && event.key.toLocaleLowerCase() === "j") {
    event.preventDefault();
    openAiWriting();
  }
  if (event.key === "Escape") {
    closeMenus();
    showGlobalSearchDialog.value = false;
    showDeleteDialog.value = false;
    knowledgeBaseDialog.value = null;
    showSettingsDialog.value = false;
    showAiWritingDialog.value = false;
  }
}

watch(
  [notes, knowledgeBases],
  () => {
    if (!hydrated) return;
    saveState.value = "saving";
    window.clearTimeout(saveTimer);
    saveTimer = window.setTimeout(
      persistNotes,
      settings.value.general.autoSaveDelayMs,
    );
  },
  { deep: true },
);

watch(selectedId, () => {
  metadataAiVersion += 1;
  metadataAiBusy.value = null;
  showAiWritingDialog.value = false;
  selectedText.value = "";
  selectionRange.value = { start: 0, end: 0 };
});

watch(aiPanelOpen, (open) => {
  if (!open) return;
  sidebarWidth.value = clamp(sidebarWidth.value, 220, maximumSidebarWidth());
  aiPanelWidth.value = clamp(aiPanelWidth.value, 300, maximumAiPanelWidth());
  persistPanelWidths();
});

onMounted(async () => {
  window.addEventListener("keydown", handleKeydown);
  window.addEventListener("click", closeMenus);
  window.addEventListener("resize", fitPanelsToWindow);
  if ("__TAURI_INTERNALS__" in window) {
    const { listen } = await import("@tauri-apps/api/event");
    unlistenTrayToast = await listen("window-hidden-to-tray", () => {
      showToast("已最小化到系统托盘，双击托盘图标可重新打开");
    });
  }
  try {
    const settingsView = await loadAppSettings();
    settings.value = settingsView.settings;
    hasApiKey.value = settingsView.hasApiKey;

    const store = await loadStore();
    knowledgeBases.value = store.knowledgeBases;
    notes.value = store.notes;

    if (knowledgeBases.value.length === 0) {
      knowledgeBases.value = [makeKnowledgeBase("我的知识库")];
    }
    const defaultBaseId = knowledgeBases.value[0].id;
    selectedKnowledgeBaseId.value = defaultBaseId;
    for (const note of notes.value) {
      if (!knowledgeBases.value.some((base) => base.id === note.knowledgeBaseId)) {
        note.knowledgeBaseId = defaultBaseId;
      }
      const parent = notes.value.find((candidate) => candidate.id === note.parentId);
      if (!parent || parent.id === note.id || parent.knowledgeBaseId !== note.knowledgeBaseId) {
        note.parentId = null;
      }
    }

    if (notes.value.length === 0) {
      notes.value = [
        makeNote(
          "欢迎使用",
          "## 从这里开始\n\n这是一款本地优先的 **AI 知识库**。\n\n- 正文采用单区所见即所得编辑，无需理解 Markdown 源码\n- 支持标题、富文本、任务清单、引用、代码、链接和表格\n- 每篇笔记仍保存为开放、可迁移的 `.md` 文件\n- 选中文字即可润色、精简、扩写或翻译\n- 标题和标签旁有各自的智能入口，正文工具栏支持续写、校对和写作工作台\n- AI 助手只读取当前文档，不会自动读取其他文档\n- 使用 `⌘/Ctrl + N` 新建笔记，`⌘/Ctrl + K` 全局搜索，`⌘/Ctrl + F` 搜索当前知识库\n\n> 只有主动使用 AI 时，当前文档或选中内容才会发送到你配置的模型服务。",
        ),
      ];
      notes.value[0].tags = ["开始", "AI"];
    }
    selectedId.value = sortedNotes.value[0]?.id ?? null;
    hydrated = true;
    await persistNotes();
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
    saveState.value = "error";
  } finally {
    isLoading.value = false;
  }
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeydown);
  window.removeEventListener("click", closeMenus);
  window.removeEventListener("resize", fitPanelsToWindow);
  window.removeEventListener("pointermove", handlePanelResize);
  document.body.classList.remove("panel-resizing");
  window.clearTimeout(saveTimer);
  window.clearTimeout(toastTimer);
  void unlistenTrayToast?.();
});
</script>

<template>
  <main
    class="app-shell"
    :class="{
      'ai-visible': aiPanelOpen,
      'sidebar-collapsed': sidebarCollapsed,
      'library-collapsed': libraryRailCollapsed,
      'trash-visible': activeNavigation === 'trash',
      'macos-titlebar': isMacOsDesktop,
    }"
    :style="layoutStyle"
    @contextmenu="handleAppContextMenu"
    @scroll.capture="contextMenu = null"
  >
    <header
      v-if="isMacOsDesktop"
      class="app-titlebar"
      data-tauri-drag-region
      aria-label="窗口标题栏"
      @pointerdown="startWindowDrag"
      @dblclick="toggleWindowMaximize"
    ></header>

    <KnowledgeRail
      v-if="!libraryRailCollapsed"
      :knowledge-bases="knowledgeBases"
      :notes="notes"
      :selected-id="selectedKnowledgeBaseId"
      :save-state="saveState"
      :trash-active="activeNavigation === 'trash'"
      :trash-count="trashedNotes.length"
      :shortcut-prefix="shortcutPrefix"
      @select="selectKnowledgeBase"
      @create="openCreateKnowledgeBase"
      @rename="openRenameKnowledgeBase"
      @delete="openDeleteKnowledgeBase"
      @import="importMarkdown"
      @toggle-rail="toggleLibraryRail"
      @open-trash="openTrash"
      @open-global-search="openGlobalSearch"
      @context="openKnowledgeBaseContextMenu"
      @open-settings="openSettings"
    />

    <button
      v-if="libraryRailCollapsed && !sidebarCollapsed"
      class="library-reopen-button"
      type="button"
      title="展开知识库栏"
      aria-label="展开知识库栏"
      @click="toggleLibraryRail"
    >›</button>

    <button
      v-if="libraryRailCollapsed && sidebarCollapsed"
      class="navigation-reopen-button"
      type="button"
      title="展开知识库和文档列表"
      aria-label="展开导航"
      @click="openNavigation"
    >
      <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M8 4v16M13 4v16"/></svg>
    </button>

    <NoteListPane
      v-if="!sidebarCollapsed && activeNavigation === 'library'"
      :key="selectedKnowledgeBaseId ?? 'no-knowledge-base'"
      ref="noteListPane"
      v-model:search-query="searchQuery"
      :notes="sortedNotes"
      :selected-id="selectedId"
      :knowledge-base-name="selectedKnowledgeBase?.name ?? '文档'"
      :mode="noteListMode"
      :collapsed-ids="collapsedNoteIds"
      :loading="isLoading"
      :shortcut-prefix="shortcutPrefix"
      @select="selectNote"
      @add-note="addNote()"
      @add-child="addChildNote"
      @set-mode="setNoteListMode"
      @toggle-branch="toggleNoteBranch"
      @context="openNoteContextMenu"
    />

    <TrashPane
      v-else-if="!sidebarCollapsed"
      :notes="trashedNotes"
      @close="closeTrash"
      @restore="restoreTrashedNote"
      @remove="requestPermanentDelete"
      @empty="requestPermanentDelete('all')"
    />

    <div
      v-if="!sidebarCollapsed"
      class="panel-resizer sidebar-resizer"
      role="separator"
      aria-label="调整侧栏宽度"
      aria-orientation="vertical"
      :aria-valuenow="Math.round(sidebarWidth)"
      tabindex="0"
      @pointerdown="startPanelResize('sidebar', $event)"
      @keydown.left.prevent="nudgePanel('sidebar', -16)"
      @keydown.right.prevent="nudgePanel('sidebar', 16)"
    >
      <button
        class="resizer-collapse-button"
        type="button"
        title="收起侧栏"
        aria-label="收起侧栏"
        @pointerdown.stop
        @click.stop="toggleSidebar"
      >‹</button>
    </div>

    <button
      v-if="sidebarCollapsed && !libraryRailCollapsed"
      class="sidebar-reopen-button"
      type="button"
      title="展开文档列表"
      aria-label="展开文档列表"
      @click="toggleSidebar"
    >›</button>

    <section v-if="selectedNote && activeNavigation === 'library'" class="editor-pane">
      <header class="editor-toolbar">
        <div class="editor-meta">
          <span>{{ formatFullDate(selectedNote.updatedAt) }}</span>
          <span class="divider"></span>
          <span>{{ characterCount }} 字</span>
        </div>

        <div class="editor-toolbar-actions">
          <button
            class="ai-toggle-button"
            :class="{ active: aiPanelOpen }"
            type="button"
            title="打开 AI 助手"
            @click="toggleAiPanel"
          >
            <span>✦</span>AI
          </button>
          <button class="icon-button" :class="{ selected: selectedNote.pinned }" type="button" title="置顶笔记" aria-label="置顶笔记" @click="togglePin">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 4 6 0-1 5 3 3v2H7v-2l3-3-1-5Zm3 10v6" /></svg>
          </button>
          <div class="popup-menu-wrap" @click.stop>
            <button class="icon-button" type="button" title="更多文档操作" aria-label="更多文档操作" @click="toggleMenu('document')">•••</button>
            <div v-if="documentMenuOpen" class="popup-menu document-popup" role="menu">
              <button type="button" role="menuitem" @click="addChildNote(selectedNote.id); closeMenus()">新建子文档</button>
              <button type="button" role="menuitem" @click="duplicateNote(); closeMenus()">创建副本</button>
              <button type="button" role="menuitem" @click="exportMarkdown(); closeMenus()">导出 Markdown</button>
              <span></span>
              <button class="danger" type="button" role="menuitem" @click="requestDelete(); closeMenus()">删除笔记</button>
            </div>
          </div>
        </div>
      </header>

      <article class="editor">
        <div class="title-row">
          <input ref="titleInput" v-model="selectedNote.title" class="title-input" type="text" maxlength="200" placeholder="无标题笔记" aria-label="笔记标题" @input="markEdited" />
          <button
            class="field-ai-button"
            type="button"
            :disabled="metadataAiBusy !== null"
            title="根据当前文章生成标题"
            @click="generateMetadata('title')"
          >
            <span>✦</span>{{ metadataAiBusy === 'title' ? '生成中' : '生成标题' }}
          </button>
        </div>
        <div class="tag-editor">
          <span>标签</span>
          <input :value="selectedNote.tags.join(', ')" type="text" maxlength="160" placeholder="工作, 灵感（用逗号分隔）" @change="updateTags" />
          <button
            class="field-ai-button compact"
            type="button"
            :disabled="metadataAiBusy !== null"
            title="根据当前文章推荐标签"
            @click="generateMetadata('tags')"
          >
            <span>✦</span>{{ metadataAiBusy === 'tags' ? '生成中' : '智能标签' }}
          </button>
        </div>

        <RichTextEditor
          ref="richTextEditor"
          v-model="selectedNote.content"
          :document-id="selectedNote.id"
          @change="markEdited"
          @selection-change="updateSelection"
          @selection-ai="handleSelectionAi"
        >
          <template #actions>
            <div class="markdown-ai-tools">
              <button type="button" title="从当前文章结尾继续写" @click="runContextualAi('continue', '续写当前文章', 'append')"><span>✦</span>续写</button>
              <button type="button" title="打开 AI 写作工作台" @click="openAiWriting"><span>✦</span>写作</button>
              <div class="popup-menu-wrap" @click.stop>
                <button type="button" title="更多 AI 工具" @click="toggleMenu('editorAi')">AI⌄</button>
                <div v-if="editorAiMenuOpen" class="popup-menu ai-tools-popup" role="menu">
                  <button type="button" role="menuitem" @click="runContextualAi('proofread', '校对当前文章', 'document'); closeMenus()">全文校对</button>
                  <button type="button" role="menuitem" @click="runContextualAi('outline', '生成文章大纲', 'append'); closeMenus()">生成大纲</button>
                  <button type="button" role="menuitem" @click="runContextualAi('summarize', '总结当前文章', 'append'); closeMenus()">生成摘要</button>
                  <button type="button" role="menuitem" @click="runContextualAi('todos', '提取行动项', 'append'); closeMenus()">提取行动项</button>
                </div>
              </div>
            </div>
          </template>
        </RichTextEditor>
      </article>

      <div v-if="errorMessage" class="error-banner" role="alert">保存失败：{{ errorMessage }}</div>
    </section>

    <section v-else class="empty-editor">
      <template v-if="activeNavigation === 'trash'">
        <div class="empty-illustration">♲</div>
        <h2>回收站</h2>
        <p>在左侧恢复文档，或永久清理不再需要的内容。</p>
      </template>
      <template v-else>
        <img class="empty-logo" src="/logo.svg" alt="" />
        <h2>写下此刻</h2>
        <p>“{{ selectedKnowledgeBase?.name }}”还没有笔记。</p>
        <button type="button" @click="addNote()">新建笔记</button>
      </template>
    </section>

    <div
      v-if="aiPanelOpen"
      class="panel-resizer ai-resizer"
      role="separator"
      aria-label="调整 AI 面板宽度"
      aria-orientation="vertical"
      :aria-valuenow="Math.round(aiPanelWidth)"
      tabindex="0"
      @pointerdown="startPanelResize('ai', $event)"
      @keydown.left.prevent="nudgePanel('ai', 16)"
      @keydown.right.prevent="nudgePanel('ai', -16)"
    ></div>

    <AiPanel
      ref="aiPanel"
      v-show="aiPanelOpen"
      :enabled="settings.ai.enabled"
      :model="settings.ai.model"
      :models="settings.ai.models"
      :note="selectedNote"
      @close="aiPanelOpen = false"
      @open-settings="openSettings('ai')"
      @apply="applyAiPanelResult"
    />

    <AiWritingDialog
      v-if="showAiWritingDialog && selectedNote"
      :key="selectedNote.id"
      :enabled="settings.ai.enabled"
      :note="selectedNote"
      @close="showAiWritingDialog = false"
      @open-settings="showAiWritingDialog = false; openSettings('ai')"
      @insert="appendDocumentAiContent"
      @replace="replaceDocumentAiContent"
    />

    <AppContextMenu
      v-if="contextMenu"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :items="contextMenuItems"
      @select="handleContextMenuAction"
    />

    <GlobalSearchDialog
      v-if="showGlobalSearchDialog"
      :notes="notes"
      :knowledge-bases="knowledgeBases"
      :shortcut-prefix="shortcutPrefix"
      @close="showGlobalSearchDialog = false"
      @select="revealGlobalSearchResult"
    />

    <div v-if="showDeleteDialog" class="dialog-backdrop" @click.self="cancelDelete">
      <section class="dialog" role="alertdialog" aria-modal="true" aria-labelledby="dialog-title">
        <div class="dialog-icon" aria-hidden="true">!</div>
        <h2 id="dialog-title">移到回收站？</h2>
        <p>“{{ selectedNote ? displayTitle(selectedNote) : '' }}”及其子文档将移到回收站，之后仍可恢复。</p>
        <div class="dialog-actions">
          <button type="button" class="secondary-button" @click="cancelDelete">取消</button>
          <button type="button" class="delete-button" @click="deleteSelectedNote">移到回收站</button>
        </div>
      </section>
    </div>

    <div v-if="trashDeleteTarget" class="dialog-backdrop" @click.self="trashDeleteTarget = null">
      <section class="dialog" role="alertdialog" aria-modal="true" aria-labelledby="permanent-delete-title">
        <div class="dialog-icon" aria-hidden="true">!</div>
        <h2 id="permanent-delete-title">永久删除？</h2>
        <p>{{ trashDeleteTarget === 'all' ? '回收站中的全部文档' : '这篇文档及其子文档' }}将从磁盘移除，无法恢复。</p>
        <div class="dialog-actions">
          <button type="button" class="secondary-button" @click="trashDeleteTarget = null">取消</button>
          <button type="button" class="delete-button" @click="confirmPermanentDelete">永久删除</button>
        </div>
      </section>
    </div>

    <div
      v-if="knowledgeBaseDialog === 'create' || knowledgeBaseDialog === 'rename'"
      class="dialog-backdrop"
      @click.self="closeKnowledgeBaseDialog"
    >
      <form class="dialog knowledge-dialog" @submit.prevent="confirmKnowledgeBaseName">
        <div class="dialog-book-icon" aria-hidden="true">▤</div>
        <h2>{{ knowledgeBaseDialog === 'create' ? '新建知识库' : '重命名知识库' }}</h2>
        <p>知识库可以将不同主题或项目的笔记分开管理。</p>
        <input
          id="knowledge-base-name"
          v-model="knowledgeBaseName"
          type="text"
          maxlength="40"
          placeholder="例如：工作、学习、个人"
          aria-label="知识库名称"
        />
        <div class="dialog-actions">
          <button type="button" class="secondary-button" @click="closeKnowledgeBaseDialog">取消</button>
          <button type="submit" class="primary-button" :disabled="!knowledgeBaseName.trim()">
            {{ knowledgeBaseDialog === 'create' ? '创建' : '保存' }}
          </button>
        </div>
      </form>
    </div>

    <div v-if="knowledgeBaseDialog === 'delete'" class="dialog-backdrop" @click.self="closeKnowledgeBaseDialog">
      <section class="dialog" role="alertdialog" aria-modal="true" aria-labelledby="delete-base-title">
        <div class="dialog-icon" aria-hidden="true">!</div>
        <h2 id="delete-base-title">删除知识库？</h2>
        <p>
          “{{ selectedKnowledgeBase?.name }}”以及其中的 {{ selectedKnowledgeBaseId ? knowledgeBaseNoteCount(selectedKnowledgeBaseId) : 0 }} 篇笔记将被永久删除。
        </p>
        <div class="dialog-actions">
          <button type="button" class="secondary-button" @click="closeKnowledgeBaseDialog">取消</button>
          <button type="button" class="delete-button" @click="deleteKnowledgeBase">删除知识库</button>
        </div>
      </section>
    </div>

    <Transition name="toast">
      <div v-if="toastMessage" class="toast" role="status">{{ toastMessage }}</div>
    </Transition>

    <SettingsDialog
      v-if="showSettingsDialog"
      :settings="settings"
      :has-api-key="hasApiKey"
      :store="storeSnapshot"
      :saving="savingSettings"
      :initial-tab="settingsInitialTab"
      @close="showSettingsDialog = false"
      @save="saveSettingsFromDialog"
      @directory-changed="handleDirectoryChanged"
      @key-cleared="hasApiKey = false"
    />
  </main>
</template>
