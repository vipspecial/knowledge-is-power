<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { flattenNoteTree, notesMatchingQuery } from "../noteTree";
import type { Note, NoteListMode } from "../types";

const props = defineProps<{
  notes: Note[];
  selectedId: string | null;
  knowledgeBaseName: string;
  searchQuery: string;
  mode: NoteListMode;
  collapsedIds: ReadonlySet<string>;
  loading: boolean;
  shortcutPrefix: string;
}>();

const emit = defineEmits<{
  "update:searchQuery": [value: string];
  select: [id: string];
  addNote: [];
  addChild: [parentId: string];
  setMode: [mode: NoteListMode];
  toggleBranch: [id: string];
  context: [id: string, event: MouseEvent];
}>();

const searchExpanded = ref(Boolean(props.searchQuery));
const searchInput = ref<HTMLInputElement | null>(null);
const visibleNotes = computed(() => notesMatchingQuery(props.notes, props.searchQuery));
const rows = computed(() => flattenNoteTree(
  visibleNotes.value,
  props.searchQuery.trim() ? new Set<string>() : props.collapsedIds,
));

watch(() => props.knowledgeBaseName, () => {
  searchExpanded.value = false;
});

function focusSearch(): void {
  searchExpanded.value = true;
  void nextTick(() => searchInput.value?.focus());
}

function closeSearch(): void {
  emit("update:searchQuery", "");
  searchExpanded.value = false;
}

function title(note: Note): string {
  return note.title.trim() || note.content.trim().split("\n")[0].replace(/^#+\s*/, "") || "无标题笔记";
}

function preview(note: Note): string {
  return note.content.replace(/[#>*_`~\[\]-]/g, "").replace(/\s+/g, " ").trim() || "还没有内容";
}

function dateLabel(isoDate: string): string {
  const date = new Date(isoDate);
  const now = new Date();
  if (date.toDateString() === now.toDateString()) {
    return new Intl.DateTimeFormat("zh-CN", { hour: "2-digit", minute: "2-digit", hour12: false }).format(date);
  }
  return new Intl.DateTimeFormat("zh-CN", { month: "short", day: "numeric" }).format(date);
}

defineExpose({ focusSearch });
</script>

<template>
  <aside class="sidebar document-pane" aria-label="文档列表">
    <header class="document-pane-header">
      <div class="document-create-actions">
        <button class="document-create" type="button" :title="`新建文档（${shortcutPrefix}N）`" @click="emit('addNote')">
          <span>＋</span>新建文档
        </button>
        <button
          class="document-create-child"
          type="button"
          :disabled="!selectedId"
          :title="selectedId ? '在当前文档下新建子文档' : '请先选择一篇文档'"
          @click="selectedId && emit('addChild', selectedId)"
        >
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 4v8a3 3 0 0 0 3 3h4M16 11v8M12 15h8"/></svg>
          子文档
        </button>
      </div>
    </header>

    <section class="document-pane-navigation" aria-label="当前知识库导航">
      <div class="document-toolbar-row" :class="{ 'search-active': searchExpanded || searchQuery }">
        <span v-if="!searchExpanded && !searchQuery" class="document-toolbar-label">文档</span>
        <button
          v-if="!searchExpanded && !searchQuery"
          class="current-search-toggle"
          type="button"
          title="搜索当前知识库"
          aria-label="搜索当前知识库"
          @click="focusSearch"
        >
          <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/></svg>
        </button>
        <label v-else class="document-search">
          <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/></svg>
          <input
            ref="searchInput"
            :value="searchQuery"
            type="search"
            placeholder="搜索当前知识库"
            aria-label="搜索文档"
            @input="emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
          />
          <kbd>{{ shortcutPrefix }}F</kbd>
          <button type="button" title="关闭搜索" aria-label="关闭当前知识库搜索" @mousedown.prevent @click="closeSearch">×</button>
        </label>

        <div v-if="!searchExpanded && !searchQuery" class="list-mode-switch" aria-label="文档列表显示方式">
          <button :class="{ active: mode === 'cards' }" type="button" title="显示标题与摘要" aria-label="卡片模式" @click="emit('setMode', 'cards')">▤</button>
          <button :class="{ active: mode === 'outline' }" type="button" title="只显示标题与层级" aria-label="层级模式" @click="emit('setMode', 'outline')">☷</button>
        </div>
      </div>

      <div class="document-pane-title">
        <div><strong>{{ knowledgeBaseName || '文档' }}</strong><small>{{ notes.length }} 篇</small></div>
      </div>
    </section>

    <section class="document-list" :class="`mode-${mode}`">
      <div v-if="loading" class="document-list-message">正在打开文档…</div>

      <div
        v-for="row in rows"
        v-else
        :key="row.note.id"
        class="document-row"
        :class="{ active: row.note.id === selectedId }"
        :style="{ '--tree-depth': String(Math.min(row.depth, 6)) }"
        @contextmenu.prevent.stop="emit('context', row.note.id, $event)"
      >
        <button
          v-if="row.hasChildren"
          class="branch-toggle"
          type="button"
          :title="collapsedIds.has(row.note.id) ? '展开子文档' : '收起子文档'"
          :aria-label="collapsedIds.has(row.note.id) ? '展开子文档' : '收起子文档'"
          @click="emit('toggleBranch', row.note.id)"
        >{{ collapsedIds.has(row.note.id) ? '›' : '⌄' }}</button>
        <span v-else class="branch-spacer">·</span>

        <button class="document-select" type="button" @click="emit('select', row.note.id)">
          <div class="document-title-line">
            <span v-if="row.note.pinned" class="pin">◆</span>
            <strong>{{ title(row.note) }}</strong>
            <time v-if="mode === 'cards'" :datetime="row.note.updatedAt">{{ dateLabel(row.note.updatedAt) }}</time>
          </div>
          <template v-if="mode === 'cards'">
            <p>{{ preview(row.note) }}</p>
            <div v-if="row.note.tags.length" class="document-tags">
              <span v-for="tag in row.note.tags.slice(0, 3)" :key="tag">#{{ tag }}</span>
            </div>
          </template>
        </button>
      </div>

      <div v-if="!loading && rows.length === 0" class="document-list-message">
        <span>没有找到相关文档</span>
        <button v-if="searchQuery" type="button" @click="closeSearch">清除搜索</button>
        <button v-else type="button" @click="emit('addNote')">新建第一篇文档</button>
      </div>
    </section>
  </aside>
</template>

<style scoped>
.document-pane {
  display: flex;
  min-width: 0;
  min-height: 0;
  flex-direction: column;
  overflow: hidden;
  background: #f4f1ea;
}

.document-pane-header {
  display: flex;
  height: 68px;
  flex: 0 0 68px;
  align-items: center;
  padding: 0 10px;
  border-bottom: 1px solid #e0dbd1;
}

.document-create-actions {
  display: flex;
  width: 100%;
  height: 34px;
  gap: 6px;
}

.document-create,
.document-create-child {
  display: flex;
  height: 34px;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 0 9px;
  border: 0;
  border-radius: 8px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 650;
}

.document-create {
  min-width: 0;
  flex: 1;
  color: #fff;
  background: var(--accent-solid);
  box-shadow: 0 4px 14px rgb(175 82 18 / 12%);
}

.document-create span {
  font-size: 16px;
}

.document-create:hover {
  background: var(--accent-strong);
}

.document-create-child {
  flex: 0 0 auto;
  border: 1px solid #d9d3c9;
  color: #6d665d;
  background: #fffefa;
}

.document-create-child:hover:not(:disabled) {
  border-color: var(--accent-border);
  color: var(--accent-strong);
  background: var(--accent-softest);
}

.document-create-child:disabled {
  cursor: default;
  opacity: .48;
}

.document-create-child svg {
  width: 15px;
  height: 15px;
  flex: 0 0 auto;
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 1.8;
}

.document-pane-navigation {
  display: flex;
  flex: 0 0 auto;
  flex-direction: column;
  gap: 4px;
  padding: 8px 10px 0;
}

.document-toolbar-row {
  display: flex;
  height: 34px;
  align-items: center;
  justify-content: space-between;
  gap: 7px;
}

.document-toolbar-row.search-active {
  justify-content: stretch;
}

.document-toolbar-label {
  color: #837d74;
  font-size: 13px;
  font-weight: 700;
  letter-spacing: .07em;
}

.current-search-toggle {
  display: grid;
  width: 34px;
  height: 34px;
  place-items: center;
  padding: 0;
  border: 1px solid #ddd8ce;
  border-radius: 8px;
  color: #837d74;
  background: rgb(255 255 255 / 62%);
  cursor: pointer;
}

.document-toolbar-row > .current-search-toggle {
  margin-left: auto;
}

.current-search-toggle:hover {
  border-color: var(--accent-border);
  color: var(--accent-strong);
  background: var(--accent-softest);
}

.current-search-toggle svg,
.document-search > svg {
  width: 15px;
  height: 15px;
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-width: 1.8;
}

.document-search {
  display: flex;
  width: 100%;
  height: 34px;
  align-items: center;
  gap: 7px;
  padding: 0 7px 0 9px;
  border: 1px solid #ddd8ce;
  border-radius: 8px;
  color: #8e887e;
  background: rgb(255 255 255 / 62%);
}

.document-search:focus-within {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px rgb(232 111 22 / 11%);
}

.document-search input {
  min-width: 0;
  flex: 1;
  border: 0;
  outline: 0;
  color: #3c3832;
  background: transparent;
  font-size: 13px;
}

.document-search input::-webkit-search-cancel-button {
  display: none;
}

.document-search kbd {
  color: #a19b91;
  font-size: 12px;
}

.document-search button {
  display: grid;
  width: 22px;
  height: 22px;
  flex: 0 0 auto;
  place-items: center;
  padding: 0;
  border: 0;
  border-radius: 5px;
  color: #948d83;
  background: transparent;
  cursor: pointer;
  font-size: 16px;
}

.document-search button:hover {
  color: var(--accent-strong);
  background: var(--accent-soft);
}

.document-pane-title {
  display: flex;
  height: 31px;
  min-width: 0;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 0 2px;
}

.document-pane-title > div:first-child {
  display: flex;
  min-width: 0;
  align-items: baseline;
  gap: 7px;
}

.document-pane-title strong {
  overflow: hidden;
  color: #4a463f;
  font-size: 14px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.document-pane-title small {
  color: #999288;
  font-size: 12px;
  white-space: nowrap;
}

.list-mode-switch {
  display: flex;
  flex: 0 0 auto;
  padding: 2px;
  border: 1px solid #ded9cf;
  border-radius: 7px;
  background: #ece9e2;
}

.list-mode-switch button {
  display: grid;
  width: 25px;
  height: 21px;
  place-items: center;
  padding: 0;
  border: 0;
  border-radius: 5px;
  color: #89837a;
  background: transparent;
  cursor: pointer;
  font-size: 14px;
}

.list-mode-switch button.active {
  color: var(--accent-strong);
  background: #fffefa;
  box-shadow: 0 1px 3px rgb(52 47 38 / 10%);
}

.document-list {
  min-height: 0;
  flex: 1;
  overflow-y: auto;
  overscroll-behavior: contain;
  padding: 3px 7px 7px;
}

.document-list::-webkit-scrollbar {
  width: 8px;
}

.document-list::-webkit-scrollbar-thumb {
  border: 3px solid transparent;
  border-radius: 8px;
  background: #c5bfb4;
  background-clip: padding-box;
}

.document-row {
  position: relative;
  display: flex;
  min-width: 0;
  align-items: flex-start;
  margin-left: calc(var(--tree-depth) * 10px);
  border-radius: 9px;
}

.document-row:hover {
  background: rgb(255 255 255 / 54%);
}

.document-row.active {
  background: #fffefa;
  box-shadow: inset 2px 0 var(--accent), 0 2px 8px rgb(59 50 37 / 5%);
}

.branch-toggle,
.branch-spacer {
  display: grid;
  width: 19px;
  height: 30px;
  flex: 0 0 auto;
  place-items: center;
  padding: 0;
  border: 0;
  color: #8b857b;
  background: transparent;
  font-size: 16px;
}

.branch-toggle {
  cursor: pointer;
}

.branch-toggle:hover {
  color: var(--accent-strong);
}

.branch-spacer {
  color: #b7b0a5;
  font-size: 13px;
}

.document-select {
  display: block;
  min-width: 0;
  flex: 1;
  padding: 8px 6px 8px 1px;
  border: 0;
  color: #49453e;
  background: transparent;
  cursor: pointer;
  text-align: left;
}

.document-title-line {
  display: flex;
  min-width: 0;
  align-items: baseline;
  gap: 6px;
}

.document-title-line strong {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  font-size: 14px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.document-title-line time {
  flex: 0 0 auto;
  color: #9b958b;
  font-size: 12px;
}

.pin {
  color: #b8672e;
  font-size: 12px;
}

.document-select p {
  margin: 4px 0 0;
  overflow: hidden;
  color: #827c72;
  font-size: 13px;
  line-height: 1.45;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.document-tags {
  display: flex;
  gap: 5px;
  margin-top: 5px;
  overflow: hidden;
}

.document-tags span {
  overflow: hidden;
  color: #b8672e;
  font-size: 12px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mode-outline .document-row {
  height: 31px;
  align-items: center;
  border-radius: 7px;
}

.mode-outline .document-select {
  height: 31px;
  padding: 0 4px 0 1px;
}

.mode-outline .document-title-line {
  align-items: center;
}

.mode-outline .document-title-line strong {
  font-size: 13px;
  font-weight: 560;
}

.document-list-message {
  display: grid;
  min-height: 150px;
  place-content: center;
  gap: 8px;
  color: #999287;
  text-align: center;
  font-size: 13px;
}

.document-list-message button {
  border: 0;
  color: var(--accent-strong);
  background: transparent;
  cursor: pointer;
  font-size: 13px;
}

.document-create:focus-visible,
.document-create-child:focus-visible,
.current-search-toggle:focus-visible,
.document-search button:focus-visible,
.list-mode-switch button:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}
</style>
