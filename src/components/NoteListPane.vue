<script setup lang="ts">
import { computed } from "vue";
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

const visibleNotes = computed(() => notesMatchingQuery(props.notes, props.searchQuery));
const rows = computed(() => flattenNoteTree(
  visibleNotes.value,
  props.searchQuery.trim() ? new Set<string>() : props.collapsedIds,
));

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
</script>

<template>
  <aside class="sidebar document-pane" aria-label="文档列表">
    <header class="document-pane-header">
      <div class="document-pane-title">
        <div><strong>{{ knowledgeBaseName || '文档' }}</strong><small>{{ notes.length }} 篇</small></div>
        <div class="list-mode-switch" aria-label="文档列表显示方式">
          <button :class="{ active: mode === 'cards' }" type="button" title="显示标题与摘要" aria-label="卡片模式" @click="emit('setMode', 'cards')">▤</button>
          <button :class="{ active: mode === 'outline' }" type="button" title="只显示标题与层级" aria-label="层级模式" @click="emit('setMode', 'outline')">☷</button>
        </div>
      </div>
      <div class="document-create-row">
        <button class="document-create" type="button" @click="emit('addNote')"><span>＋</span>新建文档<kbd>{{ shortcutPrefix }}N</kbd></button>
        <button v-if="selectedId" class="child-create" type="button" title="在当前文档下新建子文档" aria-label="新建子文档" @click="emit('addChild', selectedId)">↳＋</button>
      </div>
      <label class="document-search">
        <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/></svg>
        <input :value="searchQuery" type="search" placeholder="搜索当前知识库" aria-label="搜索文档" @input="emit('update:searchQuery', ($event.target as HTMLInputElement).value)" />
        <kbd>{{ shortcutPrefix }}F</kbd>
      </label>
    </header>

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
        <button class="row-child-create" type="button" title="新建子文档" aria-label="新建子文档" @click="emit('addChild', row.note.id)">＋</button>
      </div>

      <div v-if="!loading && rows.length === 0" class="document-list-message">
        <span>没有找到相关文档</span>
        <button v-if="searchQuery" type="button" @click="emit('update:searchQuery', '')">清除搜索</button>
        <button v-else type="button" @click="emit('addNote')">新建第一篇文档</button>
      </div>
    </section>
  </aside>
</template>

<style scoped>
.document-pane{display:flex;min-width:0;min-height:0;flex-direction:column;overflow:hidden;background:#f4f1ea}.document-pane-header{display:grid;flex:0 0 auto;gap:8px;padding:11px 10px 9px;border-bottom:1px solid #e0dbd1}.document-pane-title{display:flex;height:27px;align-items:center;justify-content:space-between;padding:0 2px}.document-pane-title>div:first-child{display:flex;min-width:0;align-items:baseline;gap:7px}.document-pane-title strong{overflow:hidden;color:#4a463f;font-size:14px;text-overflow:ellipsis;white-space:nowrap}.document-pane-title small{color:#999288;font-size:12px;white-space:nowrap}.list-mode-switch{display:flex;padding:2px;border:1px solid #ded9cf;border-radius:7px;background:#ece9e2}.list-mode-switch button{display:grid;width:25px;height:22px;place-items:center;padding:0;border:0;border-radius:5px;color:#89837a;background:transparent;cursor:pointer;font-size:14px}.list-mode-switch button.active{color:#405948;background:#fffefa;box-shadow:0 1px 3px rgb(52 47 38 / 10%)}.document-create-row{display:flex;gap:6px}.document-create{display:flex;height:34px;min-width:0;flex:1;align-items:center;gap:7px;padding:0 10px;border:0;border-radius:8px;color:#fff;background:#4d6654;cursor:pointer;font-size:13px;font-weight:650}.document-create:hover{background:#405948}.document-create span{font-size:16px}.document-create kbd{margin-left:auto;color:rgb(255 255 255 / 60%);font-size:12px}.child-create{width:39px;height:34px;border:1px solid #d7d2c8;border-radius:8px;color:#53675a;background:#fffefa;cursor:pointer;font-size:14px}.child-create:hover{border-color:#aeb9b0;background:#eef3ee}.document-search{display:flex;height:33px;align-items:center;gap:7px;padding:0 9px;border:1px solid #ddd8ce;border-radius:8px;color:#8e887e;background:rgb(255 255 255 / 62%)}.document-search:focus-within{border-color:#819386;box-shadow:0 0 0 2px rgb(77 102 84 / 9%)}.document-search svg{width:14px;height:14px;fill:none;stroke:currentColor;stroke-linecap:round;stroke-width:1.8}.document-search input{min-width:0;flex:1;border:0;outline:0;color:#3c3832;background:transparent;font-size:13px}.document-search input::-webkit-search-cancel-button{display:none}.document-search kbd{color:#a19b91;font-size:12px}.document-list{min-height:0;flex:1;overflow-y:auto;overscroll-behavior:contain;padding:7px}.document-list::-webkit-scrollbar{width:8px}.document-list::-webkit-scrollbar-thumb{border:3px solid transparent;border-radius:8px;background:#c5bfb4;background-clip:padding-box}.document-row{position:relative;display:flex;min-width:0;align-items:flex-start;margin-left:calc(var(--tree-depth) * 10px);border-radius:9px}.document-row:hover{background:rgb(255 255 255 / 54%)}.document-row.active{background:#fffefa;box-shadow:inset 2px 0 #5b7662,0 2px 8px rgb(59 50 37 / 5%)}.branch-toggle,.branch-spacer{display:grid;width:19px;height:30px;flex:0 0 auto;place-items:center;padding:0;border:0;color:#8b857b;background:transparent;font-size:16px}.branch-toggle{cursor:pointer}.branch-toggle:hover{color:#405948}.branch-spacer{color:#b7b0a5;font-size:13px}.document-select{display:block;min-width:0;flex:1;padding:8px 6px 8px 1px;border:0;color:#49453e;background:transparent;cursor:pointer;text-align:left}.document-title-line{display:flex;min-width:0;align-items:baseline;gap:6px}.document-title-line strong{min-width:0;flex:1;overflow:hidden;font-size:14px;text-overflow:ellipsis;white-space:nowrap}.document-title-line time{flex:0 0 auto;color:#9b958b;font-size:12px}.pin{color:#59705f;font-size:10px}.document-select p{margin:4px 0 0;overflow:hidden;color:#827c72;font-size:13px;line-height:1.45;text-overflow:ellipsis;white-space:nowrap}.document-tags{display:flex;gap:5px;margin-top:5px;overflow:hidden}.document-tags span{overflow:hidden;color:#718076;font-size:12px;text-overflow:ellipsis;white-space:nowrap}.row-child-create{width:24px;height:28px;flex:0 0 auto;margin:3px 3px 0 0;padding:0;border:0;border-radius:6px;color:#7b887e;background:transparent;cursor:pointer;font-size:15px;opacity:0}.document-row:hover .row-child-create,.document-row.active .row-child-create{opacity:1}.row-child-create:hover{color:#405948;background:#e8ede8}.mode-outline{padding-top:6px}.mode-outline .document-row{height:31px;align-items:center;border-radius:7px}.mode-outline .document-select{height:31px;padding:0 4px 0 1px}.mode-outline .document-title-line{align-items:center}.mode-outline .document-title-line strong{font-size:13px;font-weight:560}.mode-outline .row-child-create{margin-top:0}.document-list-message{display:grid;min-height:150px;place-content:center;gap:8px;color:#999287;text-align:center;font-size:13px}.document-list-message button{border:0;color:#48614e;background:transparent;cursor:pointer;font-size:13px}
.list-mode-switch button.active{color:var(--accent-strong)}
.document-create{background:var(--accent-solid);box-shadow:0 4px 14px rgb(175 82 18 / 12%)}
.document-create:hover{background:var(--accent-strong)}
.child-create{color:var(--accent-strong)}
.child-create:hover{border-color:var(--accent-border);background:var(--accent-soft)}
.document-search:focus-within{border-color:var(--accent);box-shadow:0 0 0 2px rgb(232 111 22 / 11%)}
.document-row.active{box-shadow:inset 2px 0 var(--accent),0 2px 8px rgb(59 50 37 / 5%)}
.branch-toggle:hover,.row-child-create:hover,.document-list-message button{color:var(--accent-strong)}
.row-child-create:hover{background:var(--accent-soft)}
.pin,.document-tags span{color:#b8672e}
</style>
