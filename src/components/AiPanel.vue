<script setup lang="ts">
import DOMPurify from "dompurify";
import { marked } from "marked";
import { computed, nextTick, onBeforeUnmount, ref } from "vue";
import { createDocumentAiRequest, streamAi } from "../ai";
import type { AiOperation, AiRequest, Note } from "../types";

interface AiMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  operation: AiOperation;
  sources: string[];
  pending?: boolean;
  error?: boolean;
}

const props = defineProps<{
  enabled: boolean;
  note?: Note;
}>();

const emit = defineEmits<{
  close: [];
  openSettings: [];
  insert: [content: string];
}>();

const messages = ref<AiMessage[]>([]);
const chatInput = ref("");
const busy = ref(false);
const messageList = ref<HTMLElement | null>(null);

const quickPrompts = [
  { label: "快速摘要", prompt: "用一段摘要和三个要点概括当前文档。" },
  { label: "发现缺口", prompt: "检查当前文档有哪些信息缺口、逻辑跳跃或需要澄清的地方。" },
  { label: "行动建议", prompt: "根据当前文档列出可执行的下一步；没有依据的事项不要补造。" },
];

let streamBuffer: string[] = [];
let streamTimer: number | undefined;
let streamTarget: AiMessage | null = null;
let streamWaiters: Array<() => void> = [];

const contextLabel = computed(() =>
  props.note ? `独立会话 · ${props.note.title || "无标题文档"}` : "未选择文档",
);

function renderMessage(content: string): string {
  return DOMPurify.sanitize(marked.parse(content, { breaks: true, gfm: true, async: false }));
}

function createId(): string {
  return crypto.randomUUID?.() ?? `${Date.now()}-${Math.random()}`;
}

async function scrollToBottom(): Promise<void> {
  await nextTick();
  if (messageList.value) messageList.value.scrollTop = messageList.value.scrollHeight;
}

function finishStreamQueue(): void {
  streamTimer = undefined;
  const waiters = streamWaiters;
  streamWaiters = [];
  for (const resolve of waiters) resolve();
}

function pumpStreamQueue(): void {
  if (!streamTarget || streamBuffer.length === 0) {
    finishStreamQueue();
    return;
  }
  const count = Math.min(streamBuffer.length, Math.max(1, Math.ceil(streamBuffer.length / 24)));
  streamTarget.content += streamBuffer.splice(0, count).join("");
  void scrollToBottom();
  streamTimer = window.setTimeout(pumpStreamQueue, 18);
}

function enqueueStreamDelta(target: AiMessage, content: string): void {
  streamTarget = target;
  streamBuffer.push(...Array.from(content));
  if (streamTimer === undefined) streamTimer = window.setTimeout(pumpStreamQueue, 0);
}

function waitForStreamQueue(): Promise<void> {
  if (streamBuffer.length === 0 && streamTimer === undefined) return Promise.resolve();
  return new Promise((resolve) => streamWaiters.push(resolve));
}

function clearStreamQueue(): void {
  if (streamTimer !== undefined) window.clearTimeout(streamTimer);
  streamBuffer = [];
  streamTarget = null;
  finishStreamQueue();
}

async function run(prompt: string): Promise<void> {
  if (busy.value || !props.enabled) return;
  if (!props.note) return;
  const sourceDocumentId = props.note.id;
  const assistant: AiMessage = {
    id: createId(),
    role: "assistant",
    content: "",
    operation: "chat",
    sources: [props.note.title || "无标题笔记"],
    pending: true,
  };
  messages.value.push({
    id: createId(),
    role: "user",
    content: prompt,
    operation: "chat",
    sources: [],
  });
  messages.value.push(assistant);
  busy.value = true;
  chatInput.value = "";
  clearStreamQueue();
  streamTarget = assistant;
  await scrollToBottom();

  const request: AiRequest = createDocumentAiRequest(props.note, "chat", prompt);

  let streamError = "";
  try {
    await streamAi(request, (event) => {
      if (props.note?.id !== sourceDocumentId) return;
      if (event.event === "delta") enqueueStreamDelta(assistant, event.content);
      if (event.event === "error") {
        streamError = event.message;
      }
    });
    await waitForStreamQueue();
    assistant.pending = false;
    if (streamError) {
      assistant.content = assistant.content
        ? `${assistant.content}\n\n> ${streamError}`
        : streamError;
      assistant.error = true;
    }
    if (!assistant.content.trim()) {
      assistant.content = "模型没有返回内容，请检查接口协议和模型配置。";
      assistant.error = true;
    }
  } catch (error) {
    await waitForStreamQueue();
    assistant.content = assistant.content
      ? `${assistant.content}\n\n> 生成中断：${String(error)}`
      : String(error);
    assistant.error = true;
    assistant.pending = false;
  } finally {
    streamTarget = null;
    busy.value = false;
    await scrollToBottom();
  }
}

function sendChat(): void {
  const value = chatInput.value.trim();
  if (value && props.note) void run(value);
}

onBeforeUnmount(clearStreamQueue);
</script>

<template>
  <aside class="ai-panel">
    <header class="ai-header">
      <div>
        <span class="ai-mark">✦</span>
        <div>
          <strong>文档助手</strong>
          <small>仅处理当前文章</small>
        </div>
      </div>
      <button type="button" aria-label="关闭 AI 助手" @click="emit('close')">×</button>
    </header>

    <template v-if="enabled">
      <section class="ai-context">
        <span></span>
        <p>{{ contextLabel }}</p>
      </section>

      <section class="ai-quick" aria-label="当前文档快捷提问">
        <button
          v-for="item in quickPrompts"
          :key="item.label"
          type="button"
          :disabled="busy || !note"
          @click="run(item.prompt)"
        >
          {{ item.label }}
        </button>
      </section>

      <section ref="messageList" class="ai-messages" aria-live="polite">
        <div v-if="messages.length === 0" class="ai-welcome">
          <div>✦</div>
          <h3>问当前文档</h3>
          <p>这里专注阅读、理解和追问。改写、标题、标签与成文入口已放到编辑位置旁边。</p>
          <button type="button" :disabled="!note" @click="run('这篇文档最重要的三个结论是什么？')">提炼核心结论</button>
          <button type="button" :disabled="!note" @click="run('以审稿人的角度，提出三个值得追问的问题。')">提出追问</button>
        </div>

        <article
          v-for="message in messages"
          :key="message.id"
          class="ai-message"
          :class="[message.role, { error: message.error }]"
        >
          <div v-if="message.role === 'assistant'" class="message-avatar">✦</div>
          <div class="message-content">
            <div v-if="message.content" class="message-markdown" v-html="renderMessage(message.content)"></div>
            <div v-else class="typing"><i></i><i></i><i></i></div>
            <div v-if="message.role === 'assistant' && message.content && !message.error" class="message-actions">
              <button type="button" @click="emit('insert', message.content)">插入正文</button>
            </div>
            <div v-if="message.role === 'assistant' && message.operation === 'chat' && message.sources.length" class="message-sources">
              <span v-for="(source, index) in message.sources" :key="`${source}-${index}`">[{{ index + 1 }}] {{ source }}</span>
            </div>
          </div>
        </article>
      </section>

      <footer class="ai-composer">
        <textarea
          v-model="chatInput"
          :disabled="busy || !note"
          rows="2"
          placeholder="询问当前文档…"
          aria-label="询问当前文档"
          @keydown.enter.exact.prevent="sendChat"
        ></textarea>
        <div>
          <span>{{ busy ? '正在生成…' : 'Enter 发送 · Shift+Enter 换行' }}</span>
          <button type="button" :disabled="busy || !note || !chatInput.trim()" aria-label="发送" @click="sendChat">↑</button>
        </div>
      </footer>
    </template>

    <section v-else class="ai-disabled">
      <div>✦</div>
      <h3>配置 AI 助手</h3>
      <p>设置模型服务后，右侧可进行当前文档问答；写作、改写、标题和标签会出现在对应编辑位置。</p>
      <button type="button" @click="emit('openSettings')">打开 AI 设置</button>
    </section>
  </aside>
</template>

<style scoped>
.ai-panel{display:flex;min-width:0;min-height:0;flex-direction:column;background:#f7f5f0}.ai-header{display:flex;height:62px;flex:0 0 auto;align-items:center;justify-content:space-between;padding:0 15px;border-bottom:1px solid #e4e0d7}.ai-header>div{display:flex;align-items:center;gap:9px}.ai-mark{display:grid;width:31px;height:31px;place-items:center;border-radius:9px;color:#f8f4ea;background:linear-gradient(145deg,#5b7863,#354c3c);font-size:14px}.ai-header>div>div{display:flex;flex-direction:column}.ai-header strong{font-size:13px}.ai-header small{margin-top:2px;color:#969086;font-size:9px}.ai-header>button{display:grid;width:28px;height:28px;place-items:center;border:0;border-radius:7px;color:#8b857b;background:transparent;cursor:pointer;font-size:20px}.ai-header>button:hover{background:#ebe7df}.ai-context{display:flex;height:34px;flex:0 0 auto;align-items:center;gap:7px;padding:0 14px;border-bottom:1px solid #eae6de;color:#827c72}.ai-context span{width:6px;height:6px;border-radius:50%;background:#729079}.ai-context p{min-width:0;overflow:hidden;font-size:9px;text-overflow:ellipsis;white-space:nowrap}.ai-actions{display:grid;grid-template-columns:repeat(3,1fr);gap:5px;padding:10px 11px;border-bottom:1px solid #e6e2d9}.ai-actions button{display:flex;height:31px;align-items:center;justify-content:center;gap:5px;border:1px solid #e0dcd3;border-radius:7px;color:#615d55;background:#fffefa;cursor:pointer;font-size:10px}.ai-actions button:hover:not(:disabled){border-color:#bdcbbf;color:#3f6249;background:#f0f5f0}.ai-actions button:disabled{opacity:.45;cursor:default}.ai-actions button span{color:#55705d;font-size:11px}.ai-messages{min-height:0;flex:1;overflow-y:auto;overscroll-behavior:contain;padding:15px 13px}.ai-messages::-webkit-scrollbar{width:8px}.ai-messages::-webkit-scrollbar-thumb{border:2px solid transparent;border-radius:8px;background:#c9c3b8;background-clip:padding-box}.ai-welcome{display:grid;justify-items:center;padding:34px 15px;text-align:center}.ai-welcome>div{display:grid;width:43px;height:43px;place-items:center;margin-bottom:11px;border-radius:13px;color:#52705b;background:#e6eee7;font-size:19px}.ai-welcome h3{margin:0;color:#48443d;font-size:14px}.ai-welcome p{margin:8px 0 17px;color:#8b857c;font-size:10px;line-height:1.6}.ai-welcome button{width:100%;margin-bottom:6px;padding:8px 10px;border:1px solid #e0dcd3;border-radius:7px;color:#69645b;background:#fffefa;cursor:pointer;font-size:10px;text-align:left}.ai-welcome button:hover{border-color:#bdcbbf;color:#3e5f48}.ai-message{display:flex;gap:7px;margin-bottom:14px}.ai-message.user{justify-content:flex-end}.message-avatar{display:grid;width:23px;height:23px;flex:0 0 auto;place-items:center;border-radius:7px;color:#fff;background:#58715f;font-size:9px}.message-content{min-width:0;max-width:calc(100% - 30px)}.user .message-content{padding:8px 10px;border-radius:10px 10px 3px 10px;color:#f9f7f1;background:#536c5b;font-size:10px;line-height:1.55}.assistant .message-content{flex:1;padding:10px 11px;border:1px solid #e3dfd6;border-radius:3px 10px 10px;background:#fffefa;box-shadow:0 2px 8px rgb(52 47 38 / 4%)}.assistant.error .message-content{border-color:#e5c6c1;background:#fff4f2}.message-markdown{color:#4d4941;font-size:11px;line-height:1.65;overflow-wrap:anywhere}.message-markdown :deep(p){margin:.4em 0}.message-markdown :deep(p:first-child){margin-top:0}.message-markdown :deep(p:last-child){margin-bottom:0}.message-markdown :deep(ul),.message-markdown :deep(ol){margin:.5em 0;padding-left:1.5em}.message-markdown :deep(code){padding:.1em .3em;border-radius:3px;background:#efebe4;font-family:ui-monospace,monospace;font-size:.9em}.message-actions{display:flex;gap:5px;margin-top:9px;padding-top:8px;border-top:1px solid #eeeae2}.message-actions button{padding:4px 7px;border:1px solid #dce3dc;border-radius:5px;color:#45624d;background:#f2f6f2;cursor:pointer;font-size:9px}.message-sources{display:flex;flex-wrap:wrap;gap:4px;margin-top:8px}.message-sources span{max-width:100%;overflow:hidden;padding:3px 6px;border-radius:4px;color:#66746a;background:#edf1ed;font-size:8px;text-overflow:ellipsis;white-space:nowrap}.typing{display:flex;gap:3px;padding:5px 1px}.typing i{width:5px;height:5px;border-radius:50%;background:#7d8e81;animation:typing 1s infinite}.typing i:nth-child(2){animation-delay:.15s}.typing i:nth-child(3){animation-delay:.3s}@keyframes typing{50%{opacity:.3;transform:translateY(-2px)}}.ai-composer{flex:0 0 auto;margin:0 10px 10px;padding:8px 9px;border:1px solid #dcd8cf;border-radius:10px;background:#fffefa;box-shadow:0 4px 16px rgb(50 45 36 / 6%)}.ai-composer textarea{width:100%;min-height:39px;resize:none;border:0;outline:0;color:#47433c;background:transparent;font-size:11px;line-height:1.5}.ai-composer>div{display:flex;align-items:center;justify-content:space-between}.ai-composer span{color:#aaa399;font-size:8px}.ai-composer button{display:grid;width:25px;height:25px;place-items:center;border:0;border-radius:7px;color:#fff;background:#4d6654;cursor:pointer;font-size:14px}.ai-composer button:disabled{background:#b9bcb6;cursor:default}.ai-disabled{display:grid;place-content:center;justify-items:center;height:100%;padding:30px;text-align:center}.ai-disabled>div{display:grid;width:55px;height:55px;place-items:center;border-radius:16px;color:#506d58;background:#e5ece6;font-size:23px}.ai-disabled h3{margin:15px 0 0;font-size:15px}.ai-disabled p{margin:8px 0 18px;color:#89837a;font-size:10px;line-height:1.65}.ai-disabled button{padding:8px 13px;border:0;border-radius:8px;color:white;background:#4d6654;cursor:pointer;font-size:10px}

.ai-actions button.featured{grid-column:1/-1;color:#fff;border-color:#506c58;background:#506c58}
.ai-actions button.featured:hover:not(:disabled){color:#fff;border-color:#3f5b48;background:#435f4c}
.ai-actions button.featured span{color:#fff}
.article-writer{display:grid;gap:8px;margin:10px 11px 0;padding:11px;border:1px solid #d9dfd9;border-radius:9px;background:#f0f5f0}
.article-writer>div{display:flex;align-items:center;justify-content:space-between}.article-writer strong{font-size:11px}.article-writer>div button{border:0;color:#7f887f;background:transparent;cursor:pointer;font-size:17px}
.article-writer textarea{width:100%;resize:vertical;padding:8px;border:1px solid #d6dcd6;border-radius:7px;outline:0;color:#48443d;background:#fff;font-size:10px;line-height:1.5}.article-writer textarea:focus{border-color:#819786;box-shadow:0 0 0 2px rgb(77 102 84 / 9%)}
.article-writer p{margin:0;color:#828a82;font-size:9px;line-height:1.5}.article-writer>button{height:30px;border:0;border-radius:7px;color:#fff;background:#4d6654;cursor:pointer;font-size:10px;font-weight:650}.article-writer>button:disabled{opacity:.5;cursor:default}

/* AI 是辅助区：压缩固定控件，把高度优先留给对话内容，同时保持正文可读。 */
.ai-header{height:52px;padding:0 13px}.ai-mark{width:28px;height:28px;border-radius:8px;font-size:13px}.ai-header>div{min-width:0}.ai-header>div>div{min-width:0}.ai-header strong,.ai-header small{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ai-header strong{font-size:12px}.ai-header small{font-size:9px}.ai-context{height:30px;padding:0 12px}.ai-context p{font-size:10px}
.ai-actions{grid-template-columns:repeat(4,minmax(0,1fr));gap:5px;padding:8px 9px}.ai-actions button{height:30px;padding:0 3px;font-size:10px}.ai-actions button.featured{grid-column:span 2}
.ai-messages{padding:12px 11px}.ai-welcome{padding:22px 10px}.ai-welcome>div{width:38px;height:38px;margin-bottom:9px}.ai-welcome h3{font-size:14px}.ai-welcome p{margin:7px 0 13px;font-size:11px}.ai-welcome button{padding:8px 9px;font-size:11px}
.message-avatar{width:25px;height:25px}.message-content{max-width:calc(100% - 32px)}.user .message-content{font-size:11px}.message-markdown{font-size:12px;line-height:1.7}.message-actions button{font-size:10px}.ai-composer{margin:0 9px 9px;padding:8px 9px}.ai-composer textarea{min-height:42px;font-size:12px}.ai-composer span{font-size:9px}.ai-composer button{width:27px;height:27px}
.article-writer{gap:7px;margin:8px 9px 0;padding:10px}.article-writer strong{font-size:12px}.article-writer textarea{font-size:11px}.article-writer p{font-size:10px}.article-writer>button{font-size:11px}
.ai-quick{display:flex;gap:5px;padding:8px 9px;border-bottom:1px solid #e6e2d9;overflow-x:auto}.ai-quick button{height:29px;flex:1;padding:0 8px;border:1px solid #dfe3de;border-radius:7px;color:#52665a;background:#fffefa;cursor:pointer;font-size:10px;white-space:nowrap}.ai-quick button:hover:not(:disabled){border-color:#aebcaf;background:#eef3ee}.ai-quick button:disabled{opacity:.45;cursor:default}
.ai-mark{background:linear-gradient(145deg,#ff9a2e,var(--accent-strong))}
.ai-context span,.typing i{background:var(--accent)}
.ai-actions button:hover:not(:disabled),.ai-welcome button:hover,.ai-quick button:hover:not(:disabled){border-color:var(--accent-border);color:var(--accent-strong);background:var(--accent-softest)}
.ai-actions button span,.ai-welcome>div,.ai-disabled>div{color:var(--accent-strong)}
.ai-welcome>div,.ai-disabled>div,.article-writer{background:var(--accent-softest)}
.message-avatar,.user .message-content,.ai-composer button,.ai-disabled button,.article-writer>button,.ai-actions button.featured{background:var(--accent-solid);border-color:var(--accent-solid)}
.ai-actions button.featured:hover:not(:disabled){border-color:var(--accent-strong);background:var(--accent-strong)}
.message-actions button,.message-sources span{border-color:var(--accent-border);color:var(--accent-strong);background:var(--accent-softest)}
.article-writer textarea:focus{border-color:var(--accent);box-shadow:0 0 0 2px rgb(232 111 22 / 11%)}
</style>
