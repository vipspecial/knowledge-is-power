<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import type { KnowledgeBase, Note } from "../types";

interface SearchResult {
  note: Note;
  knowledgeBaseName: string;
  snippet: string;
  rank: number;
}

const props = defineProps<{
  notes: Note[];
  knowledgeBases: KnowledgeBase[];
  shortcutPrefix: string;
}>();

const emit = defineEmits<{
  close: [];
  select: [knowledgeBaseId: string, noteId: string];
}>();

const query = ref("");
const activeIndex = ref(0);
const input = ref<HTMLInputElement | null>(null);

const knowledgeBaseNames = computed(() =>
  new Map(props.knowledgeBases.map((base) => [base.id, base.name])),
);

const results = computed<SearchResult[]>(() => {
  const normalized = query.value.trim().toLocaleLowerCase();
  if (!normalized) return [];

  return props.notes
    .filter((note) => !note.deletedAt)
    .map((note) => {
      const title = note.title.toLocaleLowerCase();
      const tags = note.tags.join(" ").toLocaleLowerCase();
      const content = note.content.toLocaleLowerCase();
      const rank = title === normalized
        ? 0
        : title.startsWith(normalized)
          ? 1
          : title.includes(normalized)
            ? 2
            : tags.includes(normalized)
              ? 3
              : content.includes(normalized)
                ? 4
                : -1;
      return {
        note,
        knowledgeBaseName: knowledgeBaseNames.value.get(note.knowledgeBaseId) ?? "未知知识库",
        snippet: makeSnippet(note.content, normalized),
        rank,
      };
    })
    .filter((item) => item.rank >= 0)
    .sort((left, right) => left.rank - right.rank
      || new Date(right.note.updatedAt).getTime() - new Date(left.note.updatedAt).getTime())
    .slice(0, 60);
});

watch(query, () => {
  activeIndex.value = 0;
});

watch(results, (items) => {
  if (activeIndex.value >= items.length) activeIndex.value = Math.max(0, items.length - 1);
});

onMounted(() => {
  void nextTick(() => input.value?.focus());
});

function displayTitle(note: Note): string {
  return note.title.trim() || "无标题文档";
}

function compactMarkdown(value: string): string {
  return value
    .replace(/<!--[^]*?-->/g, " ")
    .replace(/[`#>*_~\[\]-]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

function makeSnippet(content: string, normalizedQuery: string): string {
  const plain = compactMarkdown(content);
  if (!plain) return "暂无正文";
  const index = plain.toLocaleLowerCase().indexOf(normalizedQuery);
  const characters = [...plain];
  if (index < 0 || characters.length <= 120) return characters.slice(0, 120).join("");

  // Convert the JavaScript string offset to a character offset so emoji and
  // other surrogate pairs are never split around the matched text.
  const characterIndex = [...plain.slice(0, index)].length;
  const start = Math.max(0, characterIndex - 38);
  const end = Math.min(characters.length, start + 120);
  return `${start > 0 ? "…" : ""}${characters.slice(start, end).join("")}${end < characters.length ? "…" : ""}`;
}

function moveSelection(amount: number): void {
  if (!results.value.length) return;
  activeIndex.value = (activeIndex.value + amount + results.value.length) % results.value.length;
  void nextTick(() => {
    document.querySelector<HTMLElement>(`[data-search-index="${activeIndex.value}"]`)
      ?.scrollIntoView({ block: "nearest" });
  });
}

function selectResult(result: SearchResult): void {
  emit("select", result.note.knowledgeBaseId, result.note.id);
}

function selectActive(): void {
  const result = results.value[activeIndex.value];
  if (result) selectResult(result);
}
</script>

<template>
  <div class="global-search-backdrop" @click.self="emit('close')">
    <section
      class="global-search-dialog"
      role="dialog"
      aria-modal="true"
      aria-label="全局搜索"
      @keydown.esc.prevent="emit('close')"
      @keydown.down.prevent="moveSelection(1)"
      @keydown.up.prevent="moveSelection(-1)"
      @keydown.enter.prevent="selectActive"
    >
      <header class="global-search-input">
        <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/></svg>
        <input
          ref="input"
          v-model="query"
          type="search"
          autocomplete="off"
          placeholder="搜索全部知识库"
          aria-label="搜索全部知识库"
        />
        <kbd>Esc</kbd>
      </header>

      <div class="global-search-body">
        <div v-if="!query.trim()" class="global-search-empty">
          <span class="search-orbit" aria-hidden="true">⌕</span>
          <h2 id="global-search-title">搜索全部知识</h2>
          <p>输入标题、正文或标签，回收站内容不会出现在结果中。</p>
        </div>

        <div v-else-if="results.length === 0" class="global-search-empty">
          <span class="search-orbit" aria-hidden="true">0</span>
          <h2>没有找到相关文档</h2>
          <p>换个关键词，或检查当前文档是否已移到回收站。</p>
        </div>

        <div v-else class="global-search-results" role="listbox" aria-label="全局搜索结果">
          <p class="global-search-count">找到 {{ results.length }} 篇文档</p>
          <button
            v-for="(result, index) in results"
            :key="result.note.id"
            :data-search-index="index"
            :class="{ active: index === activeIndex }"
            type="button"
            role="option"
            :aria-selected="index === activeIndex"
            @mousemove="activeIndex = index"
            @click="selectResult(result)"
          >
            <span class="result-mark" aria-hidden="true">文</span>
            <span class="result-copy">
              <span class="result-heading">
                <strong>{{ displayTitle(result.note) }}</strong>
                <small>{{ result.knowledgeBaseName }}</small>
              </span>
              <span class="result-snippet">{{ result.snippet }}</span>
              <span v-if="result.note.tags.length" class="result-tags">
                <i v-for="tag in result.note.tags.slice(0, 3)" :key="tag">#{{ tag }}</i>
              </span>
            </span>
            <span class="result-open" aria-hidden="true">↵</span>
          </button>
        </div>
      </div>

      <footer>
        <span><kbd>↑</kbd><kbd>↓</kbd> 选择</span>
        <span><kbd>Enter</kbd> 打开</span>
        <span>{{ shortcutPrefix }}K 再次唤起</span>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.global-search-backdrop{position:fixed;z-index:100;inset:0;display:flex;align-items:flex-start;justify-content:center;padding:clamp(70px,12vh,128px) 18px 18px;background:rgb(43 38 30 / 28%);backdrop-filter:blur(3px)}.global-search-dialog{display:flex;width:min(680px,100%);max-height:min(620px,72vh);flex-direction:column;overflow:hidden;border:1px solid #d9d3c8;border-radius:15px;background:#fbfaf7;box-shadow:0 24px 70px rgb(43 35 24 / 24%),0 3px 12px rgb(43 35 24 / 10%)}.global-search-input{display:flex;height:62px;flex:0 0 62px;align-items:center;gap:11px;padding:0 16px;border-bottom:1px solid #e4dfd5}.global-search-input:focus-within{box-shadow:inset 0 -2px var(--accent)}.global-search-input svg{width:20px;height:20px;flex:0 0 auto;fill:none;stroke:var(--accent-strong);stroke-linecap:round;stroke-width:1.8}.global-search-input input{min-width:0;flex:1;border:0;outline:0;color:#35312b;background:transparent;font-size:17px}.global-search-input input::-webkit-search-cancel-button{display:none}.global-search-input kbd,.global-search-dialog footer kbd{padding:2px 6px;border:1px solid #ddd7cc;border-radius:5px;color:#8f887e;background:#f2eee7;font-size:var(--font-xs);box-shadow:0 1px 0 #d4cec3}.global-search-body{min-height:260px;overflow-y:auto;overscroll-behavior:contain}.global-search-empty{display:grid;min-height:280px;place-content:center;justify-items:center;padding:36px;text-align:center}.search-orbit{display:grid;width:48px;height:48px;place-items:center;margin-bottom:14px;border:1px solid var(--accent-border);border-radius:50%;color:var(--accent-strong);background:var(--accent-softest);font-family:ui-monospace,monospace;font-size:18px}.global-search-empty h2{margin:0;color:#47423b;font-family:"Songti SC",STSong,"SimSun",serif;font-size:18px}.global-search-empty p{max-width:360px;margin:8px 0 0;color:#8c857b;font-size:var(--font-sm);line-height:1.6}.global-search-count{margin:0;padding:10px 15px 7px;color:#969087;font-size:var(--font-xs)}.global-search-results{padding:0 7px 8px}.global-search-results>button{display:flex;width:100%;min-height:72px;align-items:flex-start;gap:10px;padding:10px;border:0;border-radius:9px;color:#49443c;background:transparent;cursor:pointer;text-align:left}.global-search-results>button.active{background:var(--accent-softest);box-shadow:inset 2px 0 var(--accent)}.global-search-results>button:focus-visible{outline:2px solid var(--accent);outline-offset:-2px}.result-mark{display:grid;width:28px;height:28px;flex:0 0 auto;place-items:center;border-radius:8px;color:var(--accent-strong);background:var(--accent-soft);font-family:"Songti SC",STSong,"SimSun",serif;font-size:var(--font-sm)}.result-copy{display:flex;min-width:0;flex:1;flex-direction:column;gap:4px}.result-heading{display:flex;min-width:0;align-items:baseline;gap:8px}.result-heading strong{min-width:0;overflow:hidden;font-size:var(--font-md);text-overflow:ellipsis;white-space:nowrap}.result-heading small{flex:0 0 auto;color:#9a9389;font-size:var(--font-xs)}.result-snippet{display:-webkit-box;overflow:hidden;color:#777168;font-size:var(--font-sm);line-height:1.45;-webkit-box-orient:vertical;-webkit-line-clamp:2}.result-tags{display:flex;gap:6px;overflow:hidden}.result-tags i{color:#b8672e;font-size:var(--font-xs);font-style:normal;white-space:nowrap}.result-open{align-self:center;color:#aaa399;font-size:var(--font-sm);opacity:0}.global-search-results>button.active .result-open{opacity:1}.global-search-dialog footer{display:flex;height:38px;flex:0 0 38px;align-items:center;gap:16px;padding:0 15px;border-top:1px solid #e4dfd5;color:#928b81;background:#f4f1eb;font-size:var(--font-xs)}.global-search-dialog footer span:last-child{margin-left:auto}@media(max-width:640px){.global-search-backdrop{padding:48px 10px 10px}.global-search-dialog{max-height:82vh}.global-search-dialog footer span:last-child{display:none}}
</style>
