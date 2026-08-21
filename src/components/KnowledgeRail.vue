<script setup lang="ts">
import { computed } from "vue";
import type { KnowledgeBase, Note, SaveState } from "../types";

const props = defineProps<{
  knowledgeBases: KnowledgeBase[];
  notes: Note[];
  selectedId: string | null;
  saveState: SaveState;
  documentPaneCollapsed: boolean;
  trashActive: boolean;
  trashCount: number;
}>();

const emit = defineEmits<{
  select: [id: string];
  create: [];
  rename: [];
  delete: [];
  import: [];
  toggleDocuments: [];
  toggleRail: [];
  openTrash: [];
  context: [id: string, event: MouseEvent];
  openSettings: [tab: "general" | "ai" | "storage" | "about"];
}>();

function noteCount(id: string): number {
  return props.notes.filter((note) => note.knowledgeBaseId === id && !note.deletedAt).length;
}

const selectedKnowledgeBase = computed(() =>
  props.knowledgeBases.find((base) => base.id === props.selectedId),
);

const selectedDocumentCount = computed(() =>
  props.selectedId ? noteCount(props.selectedId) : 0,
);
</script>

<template>
  <aside class="library-rail" aria-label="知识库导航">
    <header class="rail-brand">
      <img src="/logo.svg" alt="" />
      <div>
        <strong>拿了桔子跑啊</strong>
        <small>{{ selectedKnowledgeBase?.name ?? "知识库" }} · {{ selectedDocumentCount }} 篇</small>
      </div>
      <button
        class="rail-collapse-button"
        type="button"
        title="收起知识库栏"
        aria-label="收起知识库栏"
        @click="emit('toggleRail')"
      >‹</button>
      <button
        type="button"
        :title="documentPaneCollapsed ? '展开文档列表' : '收起文档列表'"
        :aria-label="documentPaneCollapsed ? '展开文档列表' : '收起文档列表'"
        @click="emit('toggleDocuments')"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16"/></svg>
      </button>
    </header>

    <section class="rail-library-section">
      <header>
        <span>全部知识库</span>
        <button type="button" title="新建知识库" aria-label="新建知识库" @click="emit('create')">+</button>
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
      <button class="trash-link" :class="{ active: trashActive }" type="button" @click="emit('openTrash')">
        <span>♲</span>回收站<small>{{ trashCount }}</small>
      </button>
    </section>

    <footer class="rail-tools">
      <button type="button" @click="emit('import')"><span>↥</span>导入 Markdown</button>
      <button type="button" @click="emit('openSettings', 'storage')"><span>▱</span>文档位置</button>
      <button type="button" @click="emit('openSettings', 'ai')"><span>✦</span>AI 配置</button>
      <button type="button" @click="emit('openSettings', 'general')"><span>⚙</span>设置</button>
      <div class="rail-save-state">
        <i :class="saveState"></i>
        {{ saveState === 'saving' ? '保存中…' : saveState === 'error' ? '保存失败' : '已保存到本机' }}
      </div>
    </footer>
  </aside>
</template>

<style scoped>
.library-rail{display:flex;min-width:0;min-height:0;flex-direction:column;overflow:hidden;border-right:1px solid #ddd8ce;background:#ebe8e0}.rail-brand{display:flex;height:58px;flex:0 0 auto;align-items:center;gap:4px;padding:0 6px 0 8px;border-bottom:1px solid #ddd8ce}.rail-brand img{width:35px;height:35px;flex:0 0 auto;border-radius:10px;box-shadow:0 3px 9px rgb(43 58 47 / 13%)}.rail-brand>div{display:flex;min-width:0;flex:1;flex-direction:column}.rail-brand strong{overflow:hidden;color:#37342e;font-family:"Songti SC",STSong,serif;font-size:12px;text-overflow:ellipsis;white-space:nowrap}.rail-brand small{overflow:hidden;margin-top:3px;color:#989187;font-size:8px;text-overflow:ellipsis;white-space:nowrap}.rail-brand>button,.rail-library-section>header button{display:grid;width:23px;height:25px;flex:0 0 auto;place-items:center;padding:0;border:0;border-radius:7px;color:#767168;background:transparent;cursor:pointer}.rail-brand>button:hover,.rail-library-section>header button:hover{color:#405948;background:rgb(255 255 255 / 58%)}.rail-brand .rail-collapse-button{font-size:18px}.rail-brand svg{width:14px;height:14px;fill:none;stroke:currentColor;stroke-linecap:round;stroke-linejoin:round;stroke-width:1.7}.rail-library-section{display:flex;min-height:0;flex:1;flex-direction:column;padding:10px 7px}.rail-library-section>header{display:flex;height:27px;flex:0 0 auto;align-items:center;justify-content:space-between;padding:0 4px 0 7px;color:#827c72;font-size:10px;font-weight:700;letter-spacing:.07em}.rail-library-section>header button{font-size:19px}.rail-library-list{min-height:0;overflow-y:auto;overscroll-behavior:contain}.rail-library-list::-webkit-scrollbar{width:7px}.rail-library-list::-webkit-scrollbar-thumb{border:2px solid transparent;border-radius:8px;background:#bbb5aa;background-clip:padding-box}.rail-library-row{display:flex;min-width:0;height:34px;align-items:center;margin-bottom:2px;border-radius:8px}.rail-library-row:hover{background:rgb(255 255 255 / 42%)}.rail-library-row.active{background:#fffefa;box-shadow:inset 2px 0 #55705d,0 2px 7px rgb(55 49 40 / 5%)}.rail-library-select{display:flex;min-width:0;height:100%;flex:1;align-items:center;gap:7px;padding:0 5px 0 9px;border:0;color:#5f5a52;background:transparent;cursor:pointer;text-align:left}.rail-library-select svg{width:15px;height:15px;flex:0 0 auto;fill:none;stroke:#748077;stroke-linecap:round;stroke-linejoin:round;stroke-width:1.7}.rail-library-select span{min-width:0;flex:1;overflow:hidden;font-size:11px;text-overflow:ellipsis;white-space:nowrap}.rail-library-select small{color:#9c968c;font-size:9px}.rail-library-actions{display:flex;padding-right:3px}.rail-library-actions button{display:grid;width:20px;height:24px;place-items:center;padding:0;border:0;border-radius:5px;color:#8d877e;background:transparent;cursor:pointer;font-size:11px}.rail-library-actions button:hover{color:#405948;background:#e8e8e1}.trash-link{display:flex;width:100%;height:33px;flex:0 0 auto;align-items:center;gap:8px;margin-top:7px;padding:0 9px;border:0;border-radius:8px;color:#716b62;background:transparent;cursor:pointer;font-size:10px;text-align:left}.trash-link:hover,.trash-link.active{color:#415849;background:rgb(255 255 255 / 58%)}.trash-link span{width:16px;font-size:14px}.trash-link small{margin-left:auto;color:#9d968c;font-size:9px}.rail-tools{display:grid;flex:0 0 auto;gap:2px;padding:8px;border-top:1px solid #d9d4ca}.rail-tools>button{display:flex;height:31px;align-items:center;gap:9px;padding:0 9px;border:0;border-radius:7px;color:#69645b;background:transparent;cursor:pointer;font-size:10px;text-align:left}.rail-tools>button:hover{color:#3f5847;background:rgb(255 255 255 / 60%)}.rail-tools>button span{display:grid;width:17px;place-items:center;color:#617267;font-size:13px}.rail-save-state{display:flex;align-items:center;gap:6px;margin-top:5px;padding:6px 9px 1px;color:#979086;font-size:9px}.rail-save-state i{width:6px;height:6px;border-radius:50%;background:#78927e}.rail-save-state i.saving{background:#c29957;animation:pulse 1s infinite}.rail-save-state i.error{background:#bd5d54}@keyframes pulse{50%{opacity:.35}}
/* Orange is the only product accent; green remains confined to the logo leaf. */
.rail-brand>button:hover,.rail-library-section>header button:hover{color:var(--accent-strong);background:var(--accent-softest)}
.rail-library-row.active{box-shadow:inset 2px 0 var(--accent),0 2px 7px rgb(55 49 40 / 5%)}
.rail-library-select svg,.rail-tools>button span{stroke:var(--accent);color:var(--accent)}
.rail-library-actions button:hover,.trash-link:hover,.trash-link.active,.rail-tools>button:hover{color:var(--accent-strong);background:var(--accent-softest)}
</style>
