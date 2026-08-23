<script setup lang="ts">
import type { Note } from "../types";

defineProps<{ notes: Note[] }>();

const emit = defineEmits<{
  close: [];
  restore: [id: string];
  remove: [id: string];
  empty: [];
}>();

function title(note: Note): string {
  return note.title.trim() || "无标题文档";
}

function deletedLabel(value: string | null): string {
  if (!value) return "刚刚删除";
  return new Intl.DateTimeFormat("zh-CN", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(new Date(value));
}
</script>

<template>
  <aside class="sidebar trash-pane" aria-label="回收站">
    <header class="trash-header">
      <div>
        <button type="button" title="返回知识库" aria-label="返回知识库" @click="emit('close')">‹</button>
        <strong>回收站</strong>
        <small>{{ notes.length }} 篇</small>
      </div>
      <button v-if="notes.length" class="empty-trash" type="button" @click="emit('empty')">清空</button>
    </header>

    <p class="trash-hint">删除的文档保留在本地，恢复前不会出现在知识库中。</p>

    <section class="trash-list">
      <article
        v-for="note in notes"
        :key="note.id"
        class="trash-item"
      >
        <div>
          <strong>{{ title(note) }}</strong>
          <time :datetime="note.deletedAt ?? undefined">{{ deletedLabel(note.deletedAt) }}</time>
        </div>
        <p>{{ note.content.replace(/[#>*_`~\[\]-]/g, '').replace(/\s+/g, ' ').trim() || '没有正文' }}</p>
        <footer>
          <button type="button" @click="emit('restore', note.id)">恢复</button>
          <button class="danger" type="button" @click="emit('remove', note.id)">永久删除</button>
        </footer>
      </article>

      <div v-if="notes.length === 0" class="trash-empty">
        <span>♲</span>
        <strong>回收站是空的</strong>
        <p>删除的文档会暂存在这里。</p>
      </div>
    </section>
  </aside>
</template>

<style scoped>
.trash-pane{display:flex;min-width:0;min-height:0;flex-direction:column;overflow:hidden;background:#f4f1ea}.trash-header{display:flex;height:52px;flex:0 0 auto;align-items:center;justify-content:space-between;padding:0 11px;border-bottom:1px solid #dfdad0}.trash-header>div{display:flex;min-width:0;align-items:center;gap:7px}.trash-header>div>button{display:grid;width:25px;height:25px;place-items:center;padding:0;border:0;border-radius:7px;color:#6f6960;background:transparent;cursor:pointer;font-size:20px}.trash-header>div>button:hover{background:#e8e5de}.trash-header strong{font-size:14px}.trash-header small{color:#999288;font-size:12px}.empty-trash{height:27px;padding:0 8px;border:1px solid #e0cac6;border-radius:7px;color:#984b44;background:#fff7f5;cursor:pointer;font-size:12px}.trash-hint{flex:0 0 auto;margin:0;padding:9px 13px;color:#918b81;background:#eeebe4;font-size:12px;line-height:1.5}.trash-list{min-height:0;flex:1;overflow-y:auto;padding:8px}.trash-item{margin-bottom:6px;padding:10px;border:1px solid #e0dbd1;border-radius:10px;background:#fffefa}.trash-item>div{display:flex;align-items:center;gap:7px}.trash-item strong{min-width:0;flex:1;overflow:hidden;color:#46423b;font-size:13px;text-overflow:ellipsis;white-space:nowrap}.trash-item time{color:#9a9489;font-size:11px;white-space:nowrap}.trash-item p{margin:5px 0 8px;overflow:hidden;color:#898379;font-size:12px;text-overflow:ellipsis;white-space:nowrap}.trash-item footer{display:flex;justify-content:flex-end;gap:5px}.trash-item footer button{height:25px;padding:0 8px;border:1px solid #d5ddd6;border-radius:6px;color:#45604d;background:#f1f5f1;cursor:pointer;font-size:12px}.trash-item footer button.danger{border-color:#ead2ce;color:#9a4c45;background:#fff5f3}.trash-empty{display:grid;height:100%;place-content:center;justify-items:center;color:#999288;text-align:center}.trash-empty span{font-size:30px}.trash-empty strong{margin-top:8px;color:#69645b;font-size:14px}.trash-empty p{margin:5px 0 0;font-size:12px}
.trash-item footer button{border-color:var(--accent-border);color:var(--accent-strong);background:var(--accent-softest)}
</style>
