<script setup lang="ts">
import type { KnowledgeBase, Note, SaveState } from "../types";

const props = defineProps<{
  knowledgeBases: KnowledgeBase[];
  notes: Note[];
  selectedId: string | null;
  saveState: SaveState;
  trashActive: boolean;
  trashCount: number;
  shortcutPrefix: string;
}>();

const emit = defineEmits<{
  select: [id: string];
  create: [];
  rename: [];
  delete: [];
  import: [];
  toggleRail: [];
  openTrash: [];
  openGlobalSearch: [];
  context: [id: string, event: MouseEvent];
  openSettings: [tab: "general" | "ai" | "storage" | "mcp" | "about"];
}>();

function noteCount(id: string): number {
  return props.notes.filter((note) => note.knowledgeBaseId === id && !note.deletedAt).length;
}
</script>

<template>
  <aside class="library-rail" aria-label="知识库导航">
    <header class="rail-brand">
      <img src="/logo.svg" alt="" />
      <strong>拿了桔子跑啊</strong>
    </header>

    <section class="rail-library-section">
      <button class="rail-global-search" type="button" @click="emit('openGlobalSearch')">
        <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/></svg>
        <span>全局搜索</span>
        <kbd>{{ shortcutPrefix }}K</kbd>
      </button>

      <header>
        <span>全部知识库</span>
        <div class="rail-section-actions">
          <button type="button" title="新建知识库" aria-label="新建知识库" @click="emit('create')">+</button>
          <button class="rail-collapse-button" type="button" title="收起知识库栏" aria-label="收起知识库栏" @click="emit('toggleRail')">‹</button>
        </div>
      </header>

      <div class="rail-library-list">
        <div
          v-for="base in knowledgeBases"
          :key="base.id"
          class="rail-library-row"
          :class="{ active: base.id === selectedId }"
          @contextmenu.prevent.stop="emit('context', base.id, $event)"
        >
          <button class="rail-library-select" type="button" @click="emit('select', base.id)">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 4h13a1 1 0 0 1 1 1v15H7a3 3 0 0 1-3-3V7a3 3 0 0 1 3-3Zm-1 13a3 3 0 0 1 3-3h12"/></svg>
            <span>{{ base.name }}</span>
            <small>{{ noteCount(base.id) }}</small>
          </button>
          <div v-if="base.id === selectedId" class="rail-library-actions">
            <button type="button" title="重命名知识库" aria-label="重命名知识库" @click="emit('rename')">✎</button>
            <button type="button" title="删除知识库" aria-label="删除知识库" @click="emit('delete')">×</button>
          </div>
        </div>
      </div>
    </section>

    <footer class="rail-tools">
      <button type="button" @click="emit('import')"><span>↥</span>导入 Markdown</button>
      <button class="trash-link" :class="{ active: trashActive }" type="button" @click="emit('openTrash')">
        <span>♲</span>回收站<small>{{ trashCount }}</small>
      </button>
      <button type="button" @click="emit('openSettings', 'general')"><span>⚙</span>设置</button>
      <div class="rail-save-state">
        <i :class="saveState"></i>
        {{ saveState === 'saving' ? '保存中…' : saveState === 'error' ? '保存失败' : '已保存到本机' }}
      </div>
    </footer>
  </aside>
</template>

<style scoped>
.library-rail {
  display: flex;
  min-width: 0;
  min-height: 0;
  flex-direction: column;
  overflow: hidden;
  border-right: 1px solid #ddd8ce;
  background: #ebe8e0;
}

.rail-brand {
  display: flex;
  height: 68px;
  flex: 0 0 68px;
  align-items: center;
  gap: 10px;
  padding: 0 10px;
  border-bottom: 1px solid #ddd8ce;
}

.rail-brand img {
  width: 44px;
  height: 44px;
  flex: 0 0 auto;
  border-radius: 13px;
  box-shadow: 0 4px 12px rgb(43 58 47 / 15%);
}

.rail-brand strong {
  overflow: hidden;
  color: #37342e;
  font-family: "Songti SC", STSong, "SimSun", serif;
  font-size:var(--font-lg);
  font-weight: 700;
  letter-spacing: .01em;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rail-library-section {
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
  padding: 8px 7px;
}

.rail-global-search {
  display: flex;
  width: 100%;
  height: 34px;
  flex: 0 0 34px;
  align-items: center;
  gap: 7px;
  padding: 0 9px;
  border: 1px solid #d8d2c8;
  border-radius: 8px;
  color: #716b62;
  background: rgb(255 255 255 / 58%);
  cursor: pointer;
  font-size:var(--font-sm);
  text-align: left;
}

.rail-global-search:hover {
  border-color: var(--accent-border);
  color: var(--accent-strong);
  background: var(--accent-softest);
}

.rail-global-search svg {
  width: 15px;
  height: 15px;
  flex: 0 0 auto;
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-width: 1.8;
}

.rail-global-search span {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rail-global-search kbd {
  color: #9d968c;
  font-size:var(--font-xs);
}

.rail-library-section > header {
  display: flex;
  height: 31px;
  flex: 0 0 31px;
  align-items: center;
  justify-content: space-between;
  margin-top: 4px;
  padding: 0 3px 0 7px;
  color: #827c72;
  font-size:var(--font-sm);
  font-weight: 700;
  letter-spacing: .07em;
}

.rail-section-actions {
  display: flex;
  align-items: center;
  gap: 1px;
}

.rail-library-section > header button {
  display: grid;
  width: 23px;
  height: 25px;
  flex: 0 0 auto;
  place-items: center;
  padding: 0;
  border: 0;
  border-radius: 7px;
  color: #767168;
  background: transparent;
  cursor: pointer;
  font-size: 19px;
}

.rail-library-section > header button:hover {
  color: var(--accent-strong);
  background: var(--accent-softest);
}

.rail-library-section > header .rail-collapse-button {
  font-size: 18px;
}

.rail-library-list {
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
}

.rail-library-list::-webkit-scrollbar {
  width: 7px;
}

.rail-library-list::-webkit-scrollbar-thumb {
  border: 2px solid transparent;
  border-radius: 8px;
  background: #bbb5aa;
  background-clip: padding-box;
}

.rail-library-row {
  display: flex;
  min-width: 0;
  height: 34px;
  align-items: center;
  margin-bottom: 2px;
  border-radius: 8px;
}

.rail-library-row:hover {
  background: rgb(255 255 255 / 42%);
}

.rail-library-row.active {
  background: #fffefa;
  box-shadow: inset 2px 0 var(--accent), 0 2px 7px rgb(55 49 40 / 5%);
}

.rail-library-select {
  display: flex;
  min-width: 0;
  height: 100%;
  flex: 1;
  align-items: center;
  gap: 7px;
  padding: 0 5px 0 9px;
  border: 0;
  color: #5f5a52;
  background: transparent;
  cursor: pointer;
  text-align: left;
}

.rail-library-select svg {
  width: 15px;
  height: 15px;
  flex: 0 0 auto;
  fill: none;
  stroke: var(--accent);
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 1.7;
}

.rail-library-select span {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  font-size:var(--font-sm);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.rail-library-select small,
.trash-link small {
  color: #9c968c;
  font-size:var(--font-xs);
}

.rail-library-actions {
  display: flex;
  padding-right: 3px;
}

.rail-library-actions button {
  display: grid;
  width: 20px;
  height: 24px;
  place-items: center;
  padding: 0;
  border: 0;
  border-radius: 5px;
  color: #8d877e;
  background: transparent;
  cursor: pointer;
  font-size:var(--font-sm);
}

.rail-library-actions button:hover,
.trash-link:hover,
.trash-link.active,
.rail-tools > button:hover {
  color: var(--accent-strong);
  background: var(--accent-softest);
}

.rail-tools {
  display: grid;
  flex: 0 0 auto;
  gap: 2px;
  padding: 8px;
  border-top: 1px solid #d9d4ca;
}

.rail-tools > button {
  display: flex;
  height: 31px;
  align-items: center;
  gap: 9px;
  padding: 0 9px;
  border: 0;
  border-radius: 7px;
  color: #69645b;
  background: transparent;
  cursor: pointer;
  font-size:var(--font-sm);
  text-align: left;
}

.rail-tools > button span {
  display: grid;
  width: 17px;
  place-items: center;
  color: var(--accent);
  font-size:var(--font-lg);
}

.trash-link {
  width: 100%;
}

.trash-link small {
  margin-left: auto;
}

.rail-save-state {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 5px;
  padding: 6px 9px 1px;
  color: #979086;
  font-size:var(--font-xs);
}

.rail-save-state i {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #78927e;
}

.rail-save-state i.saving {
  background: #c29957;
  animation: pulse 1s infinite;
}

.rail-save-state i.error {
  background: #bd5d54;
}

.rail-global-search:focus-visible,
.rail-library-section > header button:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

@keyframes pulse {
  50% { opacity: .35; }
}

@media (max-width: 980px) {
  .rail-brand {
    gap: 8px;
    padding: 0 8px;
  }

  .rail-brand img {
    width: 40px;
    height: 40px;
    border-radius: 12px;
  }

  .rail-brand strong {
    font-size:var(--font-md);
  }
}
</style>
