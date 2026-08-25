<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
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
  menuAction: [action: string, id: string];
}>();

const createMenuOpen = ref(false);
const rowMenuId = ref<string | null>(null);
const searchExpanded = ref(Boolean(props.searchQuery));
const searchInput = ref<HTMLInputElement | null>(null);
const visibleNotes = computed(() => notesMatchingQuery(props.notes, props.searchQuery));
const rows = computed(() => flattenNoteTree(
  visibleNotes.value,
  props.searchQuery.trim() ? new Set<string>() : props.collapsedIds,
));

watch(() => props.knowledgeBaseName, () => {
  searchExpanded.value = false;
  createMenuOpen.value = false;
});

onMounted(() => window.addEventListener("click", closeCreateMenu));
onBeforeUnmount(() => window.removeEventListener("click", closeCreateMenu));

function closeCreateMenu(): void {
  createMenuOpen.value = false;
  rowMenuId.value = null;
}

function rowMenuAction(action: string, id: string): void {
  rowMenuId.value = null;
  emit("menuAction", action, id);
}

function createRootNote(): void {
  closeCreateMenu();
  emit("addNote");
}

function createChildNote(): void {
  if (!props.selectedId) return;
  closeCreateMenu();
  emit("addChild", props.selectedId);
}

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
    <section class="document-pane-navigation" aria-label="当前知识库导航">
      <div class="document-toolbar-row" :class="{ 'search-active': searchExpanded || searchQuery }">
        <div
          v-if="!searchExpanded && !searchQuery"
          class="document-pane-title"
          :title="knowledgeBaseName || '文档'"
        >
          <strong>{{ knowledgeBaseName || '文档' }}</strong>
        </div>
        <button
          v-if="!searchExpanded && !searchQuery"
          class="document-tool-button"
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

        <button
          v-if="!searchExpanded && !searchQuery"
          class="document-tool-button"
          type="button"
          :title="mode === 'cards' ? '切换为仅标题' : '切换为标题与摘要'"
          :aria-label="mode === 'cards' ? '切换为仅标题' : '切换为标题与摘要'"
          @click="emit('setMode', mode === 'cards' ? 'outline' : 'cards')"
        >
          <svg v-if="mode === 'cards'" viewBox="0 0 24 24" aria-hidden="true"><rect x="4" y="5" width="16" height="5" rx="1"/><rect x="4" y="14" width="16" height="5" rx="1"/></svg>
          <svg v-else class="mode-list-icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M8 6h12M8 12h12M8 18h12"/><circle cx="4.5" cy="6" r=".8"/><circle cx="4.5" cy="12" r=".8"/><circle cx="4.5" cy="18" r=".8"/></svg>
        </button>

        <div v-if="!searchExpanded && !searchQuery" class="document-create-menu" @click.stop @keydown.esc="closeCreateMenu">
          <button
            class="document-tool-button"
            type="button"
            title="新建文档"
            aria-label="新建文档"
            aria-haspopup="menu"
            :aria-expanded="createMenuOpen"
            @click="createMenuOpen = !createMenuOpen"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 3h8l4 4v14H6zM14 3v5h5M12 11v6M9 14h6"/></svg>
          </button>
          <div v-if="createMenuOpen" class="document-create-popup" role="menu">
            <button type="button" role="menuitem" @click="createRootNote">
              <span>＋</span>
              <span>新建文档<kbd>{{ shortcutPrefix }}N</kbd></span>
            </button>
            <button type="button" role="menuitem" :disabled="!selectedId" @click="createChildNote">
              <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 4v8a3 3 0 0 0 3 3h4M16 11v8M12 15h8"/></svg>
              <span>新建子文档</span>
            </button>
          </div>
        </div>
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

        <div class="row-menu-wrap" @click.stop>
          <button
            class="row-menu-button"
            type="button"
            title="文档操作"
            aria-label="文档操作"
            aria-haspopup="menu"
            :aria-expanded="rowMenuId === row.note.id"
            @click="rowMenuId = rowMenuId === row.note.id ? null : row.note.id"
          >•••</button>
          <div v-if="rowMenuId === row.note.id" class="row-menu-popup" role="menu">
            <button type="button" role="menuitem" @click="rowMenuAction('aiMetadata', row.note.id)">AI 整理标题和标签</button>
            <button type="button" role="menuitem" @click="rowMenuAction('pin', row.note.id)">{{ row.note.pinned ? '取消置顶' : '置顶笔记' }}</button>
            <button type="button" role="menuitem" @click="rowMenuAction('export', row.note.id)">导出 Markdown</button>
            <span></span>
            <button class="danger" type="button" role="menuitem" @click="rowMenuAction('trash', row.note.id)">删除笔记</button>
          </div>
        </div>
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

.document-pane-navigation {
  flex: 0 0 auto;
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

.document-tool-button {
  display: grid;
  width: 34px;
  height: 34px;
  flex: 0 0 34px;
  place-items: center;
  padding: 0;
  border: 1px solid #ddd8ce;
  border-radius: 8px;
  color: #837d74;
  background: rgb(255 255 255 / 62%);
  cursor: pointer;
}

.document-pane-title + .document-tool-button {
  margin-left: auto;
}

.document-tool-button:hover {
  border-color: var(--accent-border);
  color: var(--accent-strong);
  background: var(--accent-softest);
}

.document-tool-button svg,
.document-search > svg {
  width: 15px;
  height: 15px;
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-width: 1.8;
}

.document-tool-button svg rect {
  fill: none;
}

.document-tool-button .mode-list-icon circle {
  fill: currentColor;
  stroke: none;
}

.document-create-menu {
  position: relative;
  width: 34px;
  height: 34px;
  flex: 0 0 34px;
}

.document-create-popup {
  position: absolute;
  z-index: 24;
  top: 40px;
  right: 0;
  display: grid;
  width: 176px;
  gap: 2px;
  padding: 6px;
  border: 1px solid #ded8ce;
  border-radius: 10px;
  background: #fffefa;
  box-shadow: 0 12px 32px rgb(46 40 31 / 18%);
}

.document-create-popup button {
  display: flex;
  min-width: 0;
  height: 31px;
  align-items: center;
  gap: 9px;
  padding: 0 9px;
  border: 0;
  border-radius: 7px;
  color: #5d574f;
  background: transparent;
  cursor: pointer;
  font-size:var(--font-sm);
  text-align: left;
}

.document-create-popup button:hover:not(:disabled) {
  color: var(--accent-strong);
  background: var(--accent-softest);
}

.document-create-popup button:disabled {
  cursor: default;
  opacity: .45;
}

.document-create-popup button > span:first-child,
.document-create-popup button > svg {
  width: 17px;
  height: 17px;
  flex: 0 0 auto;
}

.document-create-popup button > svg {
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 1.8;
}

.document-create-popup kbd {
  margin-left: auto;
  color: #a39d92;
  font-size:var(--font-xs);
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
  font-size:var(--font-sm);
}

.document-search input::-webkit-search-cancel-button {
  display: none;
}

.document-search kbd {
  color: #a19b91;
  font-size:var(--font-xs);
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
  min-width: 0;
  height: 34px;
  flex: 1;
  align-items: center;
  gap: 7px;
  padding: 0 2px;
}

.document-pane-title strong {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  color: #4a463f;
  font-size:var(--font-md);
  text-overflow: ellipsis;
  white-space: nowrap;
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

/* 折叠按钮与标题行同高：卡片模式下与标题首行对齐，仅标题模式下随行居中。 */
.branch-toggle,
.branch-spacer {
  display: grid;
  width: 19px;
  height: 21px;
  flex: 0 0 auto;
  place-items: center;
  padding: 0;
  border: 0;
  color: #8b857b;
  background: transparent;
  font-size: 15px;
  line-height: 1;
}

.mode-cards .branch-toggle,
.mode-cards .branch-spacer {
  margin-top: 8px;
}

.branch-toggle {
  cursor: pointer;
}

.branch-toggle:hover {
  color: var(--accent-strong);
}

.branch-spacer {
  color: #b7b0a5;
  font-size:var(--font-xs);
}

/* 行内「•••」菜单：悬停文档行时出现，平时隐藏。 */
.row-menu-wrap {
  position: absolute;
  z-index: 26;
  top: 5px;
  right: 4px;
  opacity: 0;
  transition: opacity 120ms ease;
}

.document-row:hover .row-menu-wrap,
.row-menu-wrap:focus-within {
  opacity: 1;
}

.row-menu-button {
  display: grid;
  height: 21px;
  min-width: 23px;
  place-items: center;
  padding: 0 4px;
  border: 0;
  border-radius: 6px;
  color: #8b857b;
  background: rgb(255 254 250 / 92%);
  box-shadow: 0 1px 5px rgb(59 50 37 / 16%);
  cursor: pointer;
  font-size: var(--font-xs);
  letter-spacing: 1px;
}

.row-menu-button:hover,
.row-menu-wrap.menu-open .row-menu-button {
  color: var(--accent-strong);
}

.row-menu-popup {
  position: absolute;
  top: calc(100% + 5px);
  right: 0;
  display: grid;
  width: 172px;
  gap: 2px;
  padding: 6px;
  border: 1px solid #ded8ce;
  border-radius: 10px;
  background: #fffefa;
  box-shadow: 0 12px 32px rgb(46 40 31 / 18%);
}

.row-menu-popup button {
  height: 29px;
  padding: 0 9px;
  border: 0;
  border-radius: 7px;
  color: #5d574f;
  background: transparent;
  cursor: pointer;
  font-size: var(--font-sm);
  text-align: left;
}

.row-menu-popup button:hover {
  color: var(--accent-strong);
  background: var(--accent-softest);
}

.row-menu-popup button.danger:hover {
  color: #a64b43;
  background: #f7eae7;
}

.row-menu-popup > span {
  height: 1px;
  margin: 3px 6px;
  background: #ece8df;
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
  font-size:var(--font-md);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.document-title-line time {
  flex: 0 0 auto;
  color: #9b958b;
  font-size:var(--font-xs);
}

.pin {
  color: #b8672e;
  font-size:var(--font-xs);
}

.document-select p {
  margin: 4px 0 0;
  overflow: hidden;
  color: #827c72;
  font-size:var(--font-sm);
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
  font-size:var(--font-xs);
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
  font-size:var(--font-sm);
  font-weight: 600;
}

.document-list-message {
  display: grid;
  min-height: 150px;
  place-content: center;
  gap: 8px;
  color: #999287;
  text-align: center;
  font-size:var(--font-sm);
}

.document-list-message button {
  border: 0;
  color: var(--accent-strong);
  background: transparent;
  cursor: pointer;
  font-size:var(--font-sm);
}

.document-tool-button:focus-visible,
.document-search button:focus-visible,
.document-create-popup button:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}
</style>
