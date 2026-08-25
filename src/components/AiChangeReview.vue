<script setup lang="ts">
import DOMPurify from "dompurify";
import { marked } from "marked";
import { computed, ref } from "vue";
import { composeAiChanges, createAiChangeGroups } from "../aiDiff";

const props = defineProps<{
  label: string;
  original: string;
  revised: string;
}>();

const emit = defineEmits<{
  close: [];
  apply: [content: string];
}>();

const groups = computed(() => createAiChangeGroups(props.original, props.revised));
const changedGroups = computed(() => groups.value.filter((group) => group.changed));
const accepted = ref<Set<number>>(new Set(changedGroups.value.map((group) => group.id)));
const allAccepted = computed(() => changedGroups.value.every((group) => accepted.value.has(group.id)));

function renderBlock(lines: string[]): string {
  if (!lines.length) return "<p class=\"empty-change\">无内容</p>";
  return DOMPurify.sanitize(marked.parse(lines.join("\n"), { breaks: true, gfm: true, async: false }));
}

function toggleGroup(id: number): void {
  const next = new Set(accepted.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  accepted.value = next;
}

function toggleAll(): void {
  accepted.value = allAccepted.value
    ? new Set()
    : new Set(changedGroups.value.map((group) => group.id));
}

function applySelected(): void {
  emit("apply", composeAiChanges(groups.value, accepted.value));
}
</script>

<template>
  <div class="change-review-backdrop" @click.self="emit('close')">
    <section class="change-review" role="dialog" aria-modal="true" aria-labelledby="change-review-title">
      <header>
        <div>
          <span>AI 修改预览</span>
          <h2 id="change-review-title">{{ label }}</h2>
          <p>选择要采用的改动，未选部分保留原文。</p>
        </div>
        <button type="button" aria-label="关闭修改预览" @click="emit('close')">×</button>
      </header>

      <div class="change-review-toolbar">
        <strong>{{ changedGroups.length }} 处改动</strong>
        <button v-if="changedGroups.length" type="button" @click="toggleAll">{{ allAccepted ? '全部取消' : '全部选择' }}</button>
      </div>

      <div class="change-review-list">
        <p v-if="!changedGroups.length" class="no-changes">AI 版本与原文没有可见差异。</p>
        <article v-for="(group, index) in changedGroups" :key="group.id" :class="{ accepted: accepted.has(group.id) }">
          <label>
            <input :checked="accepted.has(group.id)" type="checkbox" @change="toggleGroup(group.id)" />
            <span>改动 {{ index + 1 }}</span>
          </label>
          <div class="change-columns">
            <section class="change-before">
              <small>原文</small>
              <div v-html="renderBlock(group.original)"></div>
            </section>
            <section class="change-after">
              <small>AI 版本</small>
              <div v-html="renderBlock(group.revised)"></div>
            </section>
          </div>
        </article>
      </div>

      <footer>
        <span>应用后仍可使用撤销恢复</span>
        <div>
          <button class="cancel" type="button" @click="emit('close')">取消</button>
          <button class="apply" type="button" :disabled="!accepted.size" @click="applySelected">应用所选修改</button>
        </div>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.change-review-backdrop{position:fixed;z-index:180;inset:0;display:grid;place-items:center;padding:28px;background:rgb(29 27 23 / 42%);backdrop-filter:blur(6px)}
.change-review{display:flex;width:min(920px,100%);height:min(720px,calc(100vh - 56px));min-height:0;flex-direction:column;overflow:hidden;border:1px solid rgb(255 255 255 / 65%);border-radius:17px;background:#fbfaf7;box-shadow:0 30px 90px rgb(28 25 20 / 30%)}
.change-review>header{display:flex;flex:0 0 auto;align-items:flex-start;justify-content:space-between;padding:21px 24px 18px;border-bottom:1px solid #e7e2d8}.change-review>header span{color:var(--accent-strong);font-size:var(--font-xs);font-weight:700}.change-review h2{margin:4px 0 0;color:#35312b;font-size:var(--font-lg)}.change-review header p{margin:5px 0 0;color:#888177;font-size:var(--font-sm)}.change-review>header>button{display:grid;width:30px;height:30px;place-items:center;border:0;border-radius:8px;color:#8c857c;background:transparent;cursor:pointer;font-size:var(--font-lg)}.change-review>header>button:hover{background:#eeeae2}
.change-review-toolbar{display:flex;flex:0 0 auto;align-items:center;justify-content:space-between;padding:10px 24px;border-bottom:1px solid #ebe6dc;background:#f5f2eb}.change-review-toolbar strong{font-size:var(--font-sm)}.change-review-toolbar button{border:0;color:var(--accent-strong);background:transparent;cursor:pointer;font-size:var(--font-sm);font-weight:700}
.change-review-list{min-height:0;flex:1;overflow-y:auto;padding:16px 24px 22px}.change-review-list article{margin-bottom:13px;overflow:hidden;border:1px solid #dfd9cf;border-radius:11px;background:#fff}.change-review-list article.accepted{border-color:var(--accent-border);box-shadow:0 0 0 1px var(--accent-soft)}.change-review-list label{display:flex;height:34px;align-items:center;gap:8px;padding:0 12px;border-bottom:1px solid #ebe6dc;color:#6d675f;background:#f8f5ef;font-size:var(--font-sm);font-weight:700}.change-review-list input{accent-color:var(--accent)}
.no-changes{display:grid;min-height:220px;place-items:center;margin:0;color:#918a80;font-size:var(--font-md)}
.change-columns{display:grid;grid-template-columns:1fr 1fr}.change-columns>section{min-width:0;padding:11px 13px 14px}.change-columns>section+section{border-left:1px solid #ebe6dc}.change-columns small{display:block;margin-bottom:7px;font-size:var(--font-xs);font-weight:700}.change-before{background:#fff9f7}.change-before small{color:#9b6258}.change-after{background:#f8fcf8}.change-after small{color:#52725b}.change-columns :deep(p),.change-columns :deep(ul),.change-columns :deep(ol),.change-columns :deep(pre){margin:.35em 0;color:#4d4840;font-size:var(--font-sm);line-height:1.62}.change-columns :deep(ul),.change-columns :deep(ol){padding-left:1.45em}.change-columns :deep(.empty-change){color:#aaa399;font-style:italic}
.change-review>footer{display:flex;min-height:62px;flex:0 0 auto;align-items:center;justify-content:space-between;padding:11px 24px;border-top:1px solid #e7e2d8}.change-review>footer>span{color:#989187;font-size:var(--font-xs)}.change-review>footer>div{display:flex;gap:8px}.change-review footer button{height:34px;padding:0 14px;border-radius:8px;cursor:pointer;font-size:var(--font-sm);font-weight:700}.change-review .cancel{border:1px solid #ddd7ce;color:#625d55;background:#fff}.change-review .apply{border:1px solid var(--accent-solid);color:#fff;background:var(--accent-solid)}.change-review .apply:disabled{cursor:default;opacity:.5}
@media(max-width:720px){.change-review-backdrop{padding:12px}.change-review{height:calc(100vh - 24px)}.change-columns{grid-template-columns:1fr}.change-columns>section+section{border-top:1px solid #ebe6dc;border-left:0}.change-review>footer>span{display:none}}
</style>
