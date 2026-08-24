<script setup lang="ts">
import Highlight from "@tiptap/extension-highlight";
import Placeholder from "@tiptap/extension-placeholder";
import { Table, TableCell, TableHeader, TableRow } from "@tiptap/extension-table";
import TaskItem from "@tiptap/extension-task-item";
import TaskList from "@tiptap/extension-task-list";
import { Markdown } from "@tiptap/markdown";
import StarterKit from "@tiptap/starter-kit";
import { EditorContent, useEditor } from "@tiptap/vue-3";
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { MermaidCodeBlock } from "../mermaidCodeBlock";
import type { AiOperation } from "../types";

interface SelectionPayload {
  text: string;
  from: number;
  to: number;
}

interface SelectionAiAction extends SelectionPayload {
  operation: AiOperation;
  label: string;
  prompt: string;
}

const props = defineProps<{
  modelValue: string;
  documentId: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
  change: [];
  "selection-change": [selection: SelectionPayload];
  "selection-ai": [action: SelectionAiAction];
}>();

const moreOpen = ref(false);
const selectionMoreOpen = ref(false);
const formatVersion = ref(0);
const lastNonEmptySelection = ref<SelectionPayload>({ text: "", from: 0, to: 0 });
const selectionBubbleVisible = ref(false);
const selectionBubblePosition = ref({ left: 0, top: 0 });
const selectionBubbleStyle = computed(() => ({
  left: `${selectionBubblePosition.value.left}px`,
  top: `${selectionBubblePosition.value.top}px`,
}));

const editor = useEditor({
  content: props.modelValue,
  contentType: "markdown",
  extensions: [
    StarterKit.configure({
      heading: { levels: [1, 2, 3] },
      codeBlock: false,
      link: {
        openOnClick: false,
        enableClickSelection: true,
        markdownLinks: true,
      },
    }),
    MermaidCodeBlock,
    Highlight,
    Placeholder.configure({ placeholder: "写下此刻…" }),
    TaskList,
    TaskItem.configure({ nested: true }),
    Table.configure({ resizable: true }),
    TableRow,
    TableHeader,
    TableCell,
    Markdown.configure({
      markedOptions: { breaks: true, gfm: true },
    }),
  ],
  editorProps: {
    attributes: {
      class: "rich-document",
      spellcheck: "true",
      "aria-label": "笔记正文",
    },
  },
  onUpdate: ({ editor: currentEditor }) => {
    const markdown = currentEditor.getMarkdown();
    if (markdown !== props.modelValue) emit("update:modelValue", markdown);
    emit("change");
  },
  onSelectionUpdate: ({ editor: currentEditor }) => {
    const { from, to } = currentEditor.state.selection;
    const selection = {
      from,
      to,
      text: from === to ? "" : currentEditor.state.doc.textBetween(from, to, "\n").trim(),
    };
    if (selection.text) lastNonEmptySelection.value = selection;
    emit("selection-change", selection);
    if (selection.text) void nextTick(updateSelectionBubble);
    else selectionBubbleVisible.value = false;
    formatVersion.value += 1;
  },
  onTransaction: () => {
    formatVersion.value += 1;
  },
});

let currentDocumentId = props.documentId;
let editorScrollElement: HTMLElement | null = null;

/** Synchronize note changes and AI replacements without resetting the caret on ordinary typing. */
watch(
  () => [props.documentId, props.modelValue] as const,
  ([documentId, markdown]) => {
    if (!editor.value) return;
    const documentChanged = documentId !== currentDocumentId;
    const editorMarkdown = editor.value.getMarkdown();
    if (documentChanged || markdown !== editorMarkdown) {
      currentDocumentId = documentId;
      editor.value.commands.setContent(markdown, {
        contentType: "markdown",
        emitUpdate: false,
      });
      emit("selection-change", { text: "", from: 0, to: 0 });
    }
  },
);

function active(name: string, attributes?: Record<string, unknown>): boolean {
  // Read the transaction counter so toolbar states refresh with the current selection.
  void formatVersion.value;
  return editor.value?.isActive(name, attributes) ?? false;
}

function focusCommand(command: (chain: ReturnType<NonNullable<typeof editor.value>["chain"]>) => void): void {
  if (!editor.value) return;
  const chain = editor.value.chain().focus();
  command(chain);
}

function setLink(): void {
  const currentHref = editor.value?.getAttributes("link").href as string | undefined;
  const href = window.prompt("输入链接地址", currentHref ?? "https://");
  if (href === null || !editor.value) return;
  if (!href.trim()) {
    editor.value.chain().focus().extendMarkRange("link").unsetLink().run();
    return;
  }
  editor.value.chain().focus().extendMarkRange("link").setLink({ href: href.trim() }).run();
}

function closeMore(): void {
  moreOpen.value = false;
}

function insertMermaid(): void {
  if (!editor.value) return;
  editor.value
    .chain()
    .focus()
    .insertContent("```mermaid\ngraph TD\n    A[开始] --> B[处理] --> C[结束]\n```", {
      contentType: "markdown",
    })
    .run();
}

function triggerSelectionAi(operation: AiOperation, label: string, prompt = ""): void {
  if (!editor.value) return;
  const { from, to } = editor.value.state.selection;
  const text = editor.value.state.doc.textBetween(from, to, "\n").trim();
  const selection = text ? { text, from, to } : lastNonEmptySelection.value;
  if (!selection.text) return;
  selectionMoreOpen.value = false;
  selectionBubbleVisible.value = false;
  emit("selection-ai", { operation, label, prompt, ...selection });
  // Keep the ProseMirror range intact while hiding the bubble after dispatch.
  window.setTimeout(() => editor.value?.commands.blur(), 0);
}

function updateSelectionBubble(): void {
  const domSelection = window.getSelection();
  if (!domSelection || domSelection.isCollapsed || domSelection.rangeCount === 0) {
    selectionBubbleVisible.value = false;
    return;
  }
  const range = domSelection.getRangeAt(0);
  if (!editor.value?.view.dom.contains(range.commonAncestorContainer)) {
    selectionBubbleVisible.value = false;
    return;
  }
  const rect = range.getBoundingClientRect();
  const editorRect = editorScrollElement?.getBoundingClientRect();
  if (
    rect.width === 0 ||
    rect.height === 0 ||
    (editorRect && (rect.bottom < editorRect.top || rect.top > editorRect.bottom))
  ) {
    selectionBubbleVisible.value = false;
    return;
  }
  const bubbleHalfWidth = 96;
  const left = Math.min(
    window.innerWidth - bubbleHalfWidth - 8,
    Math.max(bubbleHalfWidth + 8, rect.left + rect.width / 2),
  );
  const top = rect.top > 54 ? rect.top - 8 : rect.bottom + 42;
  selectionBubblePosition.value = { left, top };
  selectionBubbleVisible.value = true;
}

function replaceRange(from: number, to: number, markdown: string): void {
  if (!editor.value) return;
  const maximum = editor.value.state.doc.content.size;
  const safeFrom = Math.max(0, Math.min(from, maximum));
  const safeTo = Math.max(safeFrom, Math.min(to, maximum));
  editor.value
    .chain()
    .focus()
    .insertContentAt({ from: safeFrom, to: safeTo }, markdown, {
      contentType: "markdown",
      updateSelection: true,
    })
    .run();
}

function replaceDocument(markdown: string): void {
  editor.value?.commands.setContent(markdown, {
    contentType: "markdown",
    emitUpdate: true,
  });
  editor.value?.commands.focus("start");
}

function appendMarkdown(markdown: string): void {
  if (!editor.value) return;
  const separator = editor.value.getText().trim() ? "\n\n" : "";
  editor.value
    .chain()
    .focus("end")
    .insertContent(`${separator}${markdown}`, { contentType: "markdown" })
    .run();
}

function handleWindowClick(): void {
  closeMore();
  selectionMoreOpen.value = false;
  if (!window.getSelection()?.toString()) selectionBubbleVisible.value = false;
}

onMounted(() => {
  window.addEventListener("click", handleWindowClick);
  window.addEventListener("resize", updateSelectionBubble);
  editorScrollElement = editor.value?.view.dom.closest<HTMLElement>(".rich-editor-surface") ?? null;
  editorScrollElement?.addEventListener("scroll", updateSelectionBubble, { passive: true });
});
onBeforeUnmount(() => {
  window.removeEventListener("click", handleWindowClick);
  window.removeEventListener("resize", updateSelectionBubble);
  editorScrollElement?.removeEventListener("scroll", updateSelectionBubble);
});

defineExpose({ replaceRange, replaceDocument, appendMarkdown });
</script>

<template>
  <section class="rich-text-editor">
    <Teleport to="body">
      <div
        v-if="selectionBubbleVisible"
        class="selection-ai-bubble"
        :style="selectionBubbleStyle"
        @click.stop
      >
        <span>✦</span>
        <button type="button" @mousedown.prevent.stop="triggerSelectionAi('polish', '润色选区')">润色</button>
        <button type="button" @mousedown.prevent.stop="triggerSelectionAi('translate', '翻译选区')">翻译</button>
        <div class="selection-more-wrap">
          <button type="button" @mousedown.prevent.stop="selectionMoreOpen = !selectionMoreOpen">更多⌄</button>
          <div v-if="selectionMoreOpen" class="selection-more-menu" role="menu">
            <button type="button" role="menuitem" @mousedown.prevent.stop="triggerSelectionAi('shorten', '精简选区')">精简表达</button>
            <button type="button" role="menuitem" @mousedown.prevent.stop="triggerSelectionAi('expand', '扩写选区')">扩写内容</button>
            <button type="button" role="menuitem" @mousedown.prevent.stop="triggerSelectionAi('explain', '解释选区')">解释内容</button>
            <button type="button" role="menuitem" @mousedown.prevent.stop="triggerSelectionAi('todos', '提取行动项')">提取行动项</button>
            <span></span>
            <button type="button" role="menuitem" @mousedown.prevent.stop="triggerSelectionAi('polish', '改为专业表达', '改为专业、准确、克制的表达，保留原意。')">专业表达</button>
            <button type="button" role="menuitem" @mousedown.prevent.stop="triggerSelectionAi('polish', '改为自然口语', '改为自然、简洁、易懂的口语表达，保留原意。')">自然口语</button>
          </div>
        </div>
      </div>
    </Teleport>

    <div class="rich-format-toolbar" aria-label="正文格式工具栏" @click.stop>
      <button
        type="button"
        title="正文"
        :class="{ active: active('paragraph') }"
        @click="focusCommand((chain) => chain.setParagraph().run())"
      >正文</button>
      <button
        type="button"
        title="一级标题"
        :class="{ active: active('heading', { level: 1 }) }"
        @click="focusCommand((chain) => chain.toggleHeading({ level: 1 }).run())"
      >H1</button>
      <button
        type="button"
        title="二级标题"
        :class="{ active: active('heading', { level: 2 }) }"
        @click="focusCommand((chain) => chain.toggleHeading({ level: 2 }).run())"
      >H2</button>
      <button
        type="button"
        title="三级标题"
        :class="{ active: active('heading', { level: 3 }) }"
        @click="focusCommand((chain) => chain.toggleHeading({ level: 3 }).run())"
      >H3</button>
      <span class="toolbar-divider"></span>
      <button type="button" title="粗体" :class="{ active: active('bold') }" @click="focusCommand((chain) => chain.toggleBold().run())"><strong>B</strong></button>
      <button type="button" title="斜体" :class="{ active: active('italic') }" @click="focusCommand((chain) => chain.toggleItalic().run())"><em>I</em></button>
      <button type="button" title="下划线" :class="{ active: active('underline') }" @click="focusCommand((chain) => chain.toggleUnderline().run())"><u>U</u></button>
      <button type="button" title="删除线" :class="{ active: active('strike') }" @click="focusCommand((chain) => chain.toggleStrike().run())"><s>S</s></button>
      <button type="button" title="高亮" :class="{ active: active('highlight') }" @click="focusCommand((chain) => chain.toggleHighlight().run())">高亮</button>
      <span class="toolbar-divider"></span>
      <button type="button" title="无序列表" :class="{ active: active('bulletList') }" @click="focusCommand((chain) => chain.toggleBulletList().run())">• 列表</button>
      <button type="button" title="有序列表" :class="{ active: active('orderedList') }" @click="focusCommand((chain) => chain.toggleOrderedList().run())">1. 列表</button>
      <button type="button" title="任务清单" :class="{ active: active('taskList') }" @click="focusCommand((chain) => chain.toggleTaskList().run())">☑ 任务</button>

      <div class="rich-more-wrap">
        <button type="button" title="更多格式" :class="{ active: moreOpen }" @click="moreOpen = !moreOpen">更多⌄</button>
        <div v-if="moreOpen" class="rich-format-menu" role="menu">
          <button type="button" role="menuitem" :class="{ active: active('blockquote') }" @click="focusCommand((chain) => chain.toggleBlockquote().run()); closeMore()">引用</button>
          <button type="button" role="menuitem" :class="{ active: active('code') }" @click="focusCommand((chain) => chain.toggleCode().run()); closeMore()">行内代码</button>
          <button type="button" role="menuitem" :class="{ active: active('codeBlock') }" @click="focusCommand((chain) => chain.toggleCodeBlock().run()); closeMore()">代码块</button>
          <button type="button" role="menuitem" @click="insertMermaid(); closeMore()">Mermaid 图表</button>
          <button type="button" role="menuitem" :class="{ active: active('link') }" @click="setLink(); closeMore()">链接</button>
          <button type="button" role="menuitem" @click="focusCommand((chain) => chain.setHorizontalRule().run()); closeMore()">分割线</button>
          <button type="button" role="menuitem" @click="focusCommand((chain) => chain.insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()); closeMore()">插入表格</button>
          <span></span>
          <button type="button" role="menuitem" :disabled="!editor?.isActive('table')" @click="focusCommand((chain) => chain.addRowAfter().run()); closeMore()">表格增加一行</button>
          <button type="button" role="menuitem" :disabled="!editor?.isActive('table')" @click="focusCommand((chain) => chain.addColumnAfter().run()); closeMore()">表格增加一列</button>
          <button type="button" role="menuitem" :disabled="!editor?.isActive('table')" @click="focusCommand((chain) => chain.deleteTable().run()); closeMore()">删除表格</button>
        </div>
      </div>

      <div class="rich-toolbar-actions">
        <slot name="actions"></slot>
      </div>
      <span class="rich-editor-badge">富文本</span>
    </div>

    <div class="rich-editor-surface">
      <EditorContent :editor="editor" class="rich-editor-content" />
    </div>
  </section>
</template>

<style scoped>
.rich-text-editor{display:flex;min-width:0;min-height:0;flex:1;flex-direction:column}.selection-ai-bubble{position:fixed;z-index:120;display:flex;align-items:center;gap:3px;padding:5px;border:1px solid rgb(255 255 255 / 14%);border-radius:9px;color:#fff;background:#37332e;box-shadow:0 10px 28px rgb(35 29 22 / 24%);transform:translate(-50%,-100%)}.selection-ai-bubble>span{padding:0 4px;color:#ffb66d;font-size:14px}.selection-ai-bubble button{height:27px;padding:0 8px;border:0;border-radius:6px;color:#f6f1e9;background:transparent;cursor:pointer;font-size:13px;white-space:nowrap}.selection-ai-bubble button:hover{background:rgb(255 255 255 / 11%)}.selection-more-wrap{position:relative}.selection-more-menu{position:absolute;z-index:40;top:33px;right:0;width:142px;padding:5px;border:1px solid #4e4942;border-radius:8px;background:#37332e;box-shadow:0 12px 30px rgb(20 17 14 / 30%)}.selection-more-menu button{display:flex;width:100%;justify-content:flex-start}.selection-more-menu>span{display:block;height:1px;margin:4px;background:#514c45}.rich-format-toolbar{position:relative;z-index:4;display:flex;min-height:39px;flex:0 0 auto;align-items:center;gap:3px;padding:5px 8px;border:1px solid #e2ded5;border-radius:10px 10px 0 0;background:#faf8f3;overflow:visible}.rich-format-toolbar>button,.rich-more-wrap>button{height:27px;padding:0 7px;border:0;border-radius:6px;color:#6e685f;background:transparent;cursor:pointer;font-size:13px;white-space:nowrap}.rich-format-toolbar>button:hover,.rich-format-toolbar>button.active,.rich-more-wrap>button:hover,.rich-more-wrap>button.active{color:var(--accent-strong);background:var(--accent-soft)}.toolbar-divider{width:1px;height:15px;margin:0 2px;background:#ddd8ce}.rich-more-wrap{position:relative}.rich-format-menu{position:absolute;z-index:20;top:32px;left:0;width:168px;padding:6px;border:1px solid #ded9cf;border-radius:9px;background:#fffefa;box-shadow:0 12px 30px rgb(43 37 28 / 14%)}.rich-format-menu button{display:flex;width:100%;height:29px;align-items:center;padding:0 9px;border:0;border-radius:6px;color:#605b53;background:transparent;cursor:pointer;font-size:13px;text-align:left}.rich-format-menu button:hover,.rich-format-menu button.active{color:var(--accent-strong);background:var(--accent-softest)}.rich-format-menu button:disabled{opacity:.4;cursor:default}.rich-format-menu>span{display:block;height:1px;margin:4px;background:#ebe7df}.rich-toolbar-actions{display:flex;min-width:0;align-items:center;gap:3px;margin-left:auto}.rich-editor-badge{margin:0 3px 0 5px;color:#aaa399;font-size:12px;white-space:nowrap}.rich-editor-surface{min-width:0;min-height:0;flex:1;overflow-y:auto;overscroll-behavior:contain;border:1px solid #e2ded5;border-top:0;border-radius:0 0 10px 10px;background:#fffefa}.rich-editor-surface::-webkit-scrollbar{width:9px}.rich-editor-surface::-webkit-scrollbar-thumb{border:3px solid transparent;border-radius:8px;background:#c7c1b6;background-clip:padding-box}.rich-editor-content{min-height:100%}.rich-editor-content :deep(.rich-document){width:min(100%,1180px);min-height:100%;margin:0 auto;padding:22px clamp(24px,4vw,64px) 80px;outline:0;color:#3f3b34;font-family:"Songti SC",STSong,Georgia,"Noto Serif CJK SC","SimSun",serif;font-size:17px;line-height:1.82;overflow-wrap:anywhere}.rich-editor-content :deep(.rich-document>*:first-child){margin-top:0}.rich-editor-content :deep(.rich-document p){margin:.72em 0}.rich-editor-content :deep(.rich-document h1),.rich-editor-content :deep(.rich-document h2),.rich-editor-content :deep(.rich-document h3){margin:1.35em 0 .55em;color:#2e2a25;line-height:1.35}.rich-editor-content :deep(.rich-document h1){padding-bottom:.28em;border-bottom:1px solid #e8e3d9;font-size:2em}.rich-editor-content :deep(.rich-document h2){font-size:1.55em}.rich-editor-content :deep(.rich-document h3){font-size:1.25em}.rich-editor-content :deep(.rich-document ul),.rich-editor-content :deep(.rich-document ol){margin:.7em 0;padding-left:1.65em}.rich-editor-content :deep(.rich-document li){margin:.28em 0}.rich-editor-content :deep(.rich-document blockquote){margin:1em 0;padding:.35em 1em;border-left:3px solid var(--accent-border);color:#6a6258;background:var(--accent-softest)}.rich-editor-content :deep(.rich-document code){padding:.12em .34em;border-radius:4px;background:#f0ece4;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;font-size:.88em}.rich-editor-content :deep(.rich-document pre){overflow-x:auto;margin:1em 0;padding:14px 16px;border-radius:9px;color:#eee9df;background:#332f2a}.rich-editor-content :deep(.rich-document pre code){padding:0;color:inherit;background:transparent}.rich-editor-content :deep(.rich-document a){color:var(--accent-strong);text-decoration:underline;text-underline-offset:3px}.rich-editor-content :deep(.rich-document mark){padding:.04em .12em;border-radius:2px;color:inherit;background:#ffe0a8}.rich-editor-content :deep(.rich-document hr){margin:1.8em 0;border:0;border-top:1px solid #ddd7cc}.rich-editor-content :deep(.rich-document ul[data-type="taskList"]){padding:0;list-style:none}.rich-editor-content :deep(.rich-document ul[data-type="taskList"] li){display:flex;align-items:flex-start;gap:8px}.rich-editor-content :deep(.rich-document ul[data-type="taskList"] li>label){padding-top:.25em}.rich-editor-content :deep(.rich-document ul[data-type="taskList"] li>div){min-width:0;flex:1}.rich-editor-content :deep(.rich-document table){width:100%;margin:1em 0;border-collapse:collapse;table-layout:fixed}.rich-editor-content :deep(.rich-document th),.rich-editor-content :deep(.rich-document td){position:relative;min-width:80px;padding:8px 10px;border:1px solid #dcd6ca;vertical-align:top}.rich-editor-content :deep(.rich-document th){background:#f5f1ea;font-weight:700}.rich-editor-content :deep(.rich-document .selectedCell::after){position:absolute;inset:0;content:"";pointer-events:none;background:rgb(232 111 22 / 10%)}.rich-editor-content :deep(.rich-document .column-resize-handle){position:absolute;top:0;right:-2px;bottom:-2px;width:4px;background:var(--accent)}.rich-editor-content :deep(.rich-document p.is-editor-empty:first-child::before){float:left;height:0;color:#bdb6aa;content:"写下此刻…";pointer-events:none}@media(max-width:1180px){.rich-format-toolbar>button:nth-of-type(1),.rich-format-toolbar>button:nth-of-type(4),.rich-format-toolbar>button:nth-of-type(9){display:none}.rich-editor-content :deep(.rich-document){padding-right:28px;padding-left:28px}}@media(max-width:860px){.rich-format-toolbar>button:nth-of-type(6),.rich-format-toolbar>button:nth-of-type(7),.rich-format-toolbar>button:nth-of-type(8){display:none}.rich-editor-badge{display:none}}
</style>

<style scoped>
/* TipTap list items and table cells contain paragraphs internally. Reset the
   document-level paragraph rhythm in these compact structures. */
.rich-editor-content :deep(.rich-document li > p),
.rich-editor-content :deep(.rich-document th > p),
.rich-editor-content :deep(.rich-document td > p),
.rich-editor-content :deep(.rich-document ul[data-type="taskList"] li > div > p) {
  margin: 0;
}

.rich-editor-content :deep(.rich-document li > p + p),
.rich-editor-content :deep(.rich-document th > p + p),
.rich-editor-content :deep(.rich-document td > p + p),
.rich-editor-content :deep(.rich-document ul[data-type="taskList"] li > div > p + p) {
  margin-top: 0.45em;
}

.rich-editor-content :deep(.rich-document li > ul),
.rich-editor-content :deep(.rich-document li > ol),
.rich-editor-content :deep(.rich-document ul[data-type="taskList"] li > div > ul) {
  margin: 0.28em 0 0;
}

.rich-editor-content :deep(.rich-document ul[data-type="taskList"] > li) {
  align-items: stretch;
  gap: 9px;
  margin: 0.16em 0;
}

.rich-editor-content :deep(.rich-document ul[data-type="taskList"] li > label) {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  align-self: stretch;
  padding: 0;
  cursor: pointer;
  user-select: none;
}

.rich-editor-content :deep(.rich-document ul[data-type="taskList"] input[type="checkbox"]) {
  width: 16px;
  height: 16px;
  margin: 0;
  accent-color: var(--accent-solid);
  cursor: pointer;
}

.rich-editor-content :deep(.rich-document ul[data-type="taskList"] li > div) {
  min-width: 0;
}

.rich-editor-content :deep(.rich-document ul[data-type="taskList"] li[data-checked="true"] > div) {
  color: #8d877d;
  text-decoration: line-through;
  text-decoration-color: #bbb3a8;
}

.rich-editor-content :deep(.rich-document blockquote > p:first-child) {
  margin-top: 0;
}

.rich-editor-content :deep(.rich-document blockquote > p:last-child) {
  margin-bottom: 0;
}

.rich-editor-content :deep(.mermaid-code-block:not(.is-mermaid)) .mermaid-diagram {
  display: none;
}

.rich-editor-content :deep(.mermaid-code-block.is-mermaid) {
  display: grid;
  gap: 8px;
}

.rich-editor-content :deep(.mermaid-diagram) {
  overflow-x: auto;
  padding: 14px 16px;
  border: 1px solid #e0dbd1;
  border-radius: 9px;
  background: #fff;
  text-align: center;
}

.rich-editor-content :deep(.mermaid-diagram svg) {
  max-width: 100%;
  height: auto;
}

.rich-editor-content :deep(.mermaid-diagram.has-error) {
  display: flex;
  min-height: 0;
  align-items: center;
  justify-content: center;
  padding: 10px 14px;
}

.rich-editor-content :deep(.mermaid-diagram.has-error span) {
  color: #a0785f;
  font-size: 13px;
}

.rich-editor-content :deep(.mermaid-code-block.is-mermaid pre) {
  margin: 0;
  font-size: 13px;
}
</style>
