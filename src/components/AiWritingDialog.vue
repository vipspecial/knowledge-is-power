<script setup lang="ts">
import DOMPurify from "dompurify";
import { marked } from "marked";
import { computed, onBeforeUnmount, ref } from "vue";
import { createDocumentAiRequest, streamAi } from "../ai";
import { createStreamPacer } from "../streaming";
import type { AiOperation, Note } from "../types";

interface WritingTemplate {
  id: string;
  icon: string;
  name: string;
  description: string;
  operation: AiOperation;
  instruction: string;
  placeholder: string;
}

const props = defineProps<{ enabled: boolean; note: Note }>();
const emit = defineEmits<{
  close: [];
  openSettings: [];
  insert: [content: string];
  replace: [content: string];
}>();

const templates: WritingTemplate[] = [
  {
    id: "article",
    icon: "文",
    name: "完整文章",
    description: "从素材写成结构完整的长文",
    operation: "write",
    instruction: "把当前文档素材写成一篇结构完整的文章。",
    placeholder: "例如：面向产品经理，约 1500 字，专业但易读…",
  },
  {
    id: "outline",
    icon: "纲",
    name: "文章大纲",
    description: "先理清结构和缺失信息",
    operation: "outline",
    instruction: "根据当前文档生成可直接用于写作的详细大纲。",
    placeholder: "例如：三层结构，突出案例和行动建议…",
  },
  {
    id: "meeting",
    icon: "会",
    name: "会议纪要",
    description: "整理结论、决策和待办",
    operation: "write",
    instruction: "把当前文档整理成会议纪要，包含议题、关键讨论、明确决策和待办事项；没有的信息不要补造。",
    placeholder: "可补充会议背景或希望强调的结论…",
  },
  {
    id: "social",
    icon: "帖",
    name: "社交内容",
    description: "改写成易读、易传播的短内容",
    operation: "write",
    instruction: "基于当前文档写成适合社交平台发布的内容，开头有吸引力，表达自然，保留事实边界。",
    placeholder: "例如：小红书风格，不超过 800 字，不使用夸张标题…",
  },
  {
    id: "ideas",
    icon: "想",
    name: "头脑风暴",
    description: "从已有内容延伸具体方向",
    operation: "brainstorm",
    instruction: "围绕当前文档进行头脑风暴。",
    placeholder: "例如：从用户价值、商业模式和风险三个方向思考…",
  },
];

const selectedTemplateId = ref("article");
const requirements = ref("");
const output = ref("");
const errorMessage = ref("");
const busy = ref(false);
let taskVersion = 0;
const streamPacer = createStreamPacer((chunk) => {
  output.value += chunk;
});

const selectedTemplate = computed(
  () => templates.find((template) => template.id === selectedTemplateId.value) ?? templates[0],
);

const renderedOutput = computed(() =>
  DOMPurify.sanitize(marked.parse(output.value, { breaks: true, gfm: true, async: false })),
);

async function generate(): Promise<void> {
  if (!props.enabled || busy.value) return;
  const sourceDocumentId = props.note.id;
  const version = ++taskVersion;
  output.value = "";
  errorMessage.value = "";
  busy.value = true;
  streamPacer.reset();
  const prompt = `${selectedTemplate.value.instruction}\n\n用户补充要求：${requirements.value.trim() || "无"}`;
  const request = createDocumentAiRequest(
    props.note,
    selectedTemplate.value.operation,
    prompt,
  );
  let streamError = "";
  try {
    await streamAi(request, (event) => {
      if (version !== taskVersion || props.note.id !== sourceDocumentId) return;
      if (event.event === "delta") streamPacer.push(event.content);
      if (event.event === "error") streamError = event.message;
    });
    if (version !== taskVersion || props.note.id !== sourceDocumentId) return;
    await streamPacer.flush();
    if (streamError) errorMessage.value = streamError;
    if (!output.value.trim() && !streamError) errorMessage.value = "模型没有返回内容";
  } catch (error) {
    if (version === taskVersion) {
      await streamPacer.flush();
      errorMessage.value = String(error);
    }
  } finally {
    if (version === taskVersion) busy.value = false;
  }
}

function apply(mode: "insert" | "replace"): void {
  if (!output.value.trim() || busy.value) return;
  if (mode === "insert") emit("insert", output.value.trim());
  else emit("replace", output.value.trim());
  emit("close");
}

onBeforeUnmount(() => {
  taskVersion += 1;
  streamPacer.reset();
});
</script>

<template>
  <div class="writing-backdrop" @click.self="emit('close')">
    <section class="writing-dialog" role="dialog" aria-modal="true" aria-labelledby="writing-title">
      <header>
        <div>
          <span>✦</span>
          <div>
            <h2 id="writing-title">AI 写作工作台</h2>
            <p>只使用当前文档“{{ note.title || '无标题文档' }}”作为素材</p>
          </div>
        </div>
        <button type="button" aria-label="关闭 AI 写作工作台" @click="emit('close')">×</button>
      </header>

      <div v-if="enabled" class="writing-content">
        <aside class="writing-templates" aria-label="写作类型">
          <button
            v-for="template in templates"
            :key="template.id"
            :class="{ active: selectedTemplateId === template.id }"
            type="button"
            @click="selectedTemplateId = template.id"
          >
            <span>{{ template.icon }}</span>
            <div><strong>{{ template.name }}</strong><small>{{ template.description }}</small></div>
          </button>
        </aside>

        <main class="writing-main">
          <label>
            <span>补充写作要求</span>
            <textarea
              v-model="requirements"
              rows="4"
              maxlength="4000"
              :placeholder="selectedTemplate.placeholder"
              :disabled="busy"
            ></textarea>
          </label>
          <div class="writing-boundary"><i></i>仅当前文档 · 不读取其他文章</div>
          <button class="generate-button" type="button" :disabled="busy" @click="generate">
            {{ busy ? '正在流式生成…' : `生成${selectedTemplate.name}` }}
          </button>

          <section class="writing-output" aria-live="polite">
            <div v-if="output" class="writing-markdown" v-html="renderedOutput"></div>
            <div v-else-if="busy" class="writing-loading"><i></i><i></i><i></i><span>正在构思并生成内容</span></div>
            <div v-else class="writing-empty">
              <span>✦</span>
              <p>选择写作类型，AI 会基于当前文档生成可继续编辑的草稿。</p>
            </div>
          </section>
          <p v-if="errorMessage" class="writing-error">{{ errorMessage }}</p>
        </main>
      </div>

      <div v-else class="writing-disabled">
        <span>✦</span><h3>请先配置 AI</h3><p>设置模型服务后即可使用场景化写作工具。</p>
        <button type="button" @click="emit('openSettings')">打开 AI 设置</button>
      </div>

      <footer v-if="enabled">
        <span>生成结果不会自动覆盖原文</span>
        <div>
          <button type="button" class="secondary" @click="emit('close')">取消</button>
          <button type="button" class="secondary" :disabled="!output || busy" @click="apply('insert')">追加到正文</button>
          <button type="button" class="primary" :disabled="!output || busy" @click="apply('replace')">设为正文</button>
        </div>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.writing-backdrop{position:fixed;z-index:55;inset:0;display:grid;place-items:center;padding:28px;background:rgb(29 27 23 / 42%);backdrop-filter:blur(7px)}
.writing-dialog{display:flex;width:min(900px,100%);height:min(690px,calc(100vh - 56px));overflow:hidden;flex-direction:column;border:1px solid rgb(255 255 255 / 70%);border-radius:18px;background:#fbfaf7;box-shadow:0 30px 90px rgb(26 23 18 / 30%)}
header{display:flex;min-height:68px;align-items:center;justify-content:space-between;padding:13px 20px;border-bottom:1px solid #e5e1d8}header>div{display:flex;min-width:0;align-items:center;gap:11px}header>div>span{display:grid;width:35px;height:35px;place-items:center;border-radius:10px;color:#fff;background:#506b57}header h2,header p{margin:0}header h2{font-size:17px}header p{margin-top:3px;overflow:hidden;color:#8d877d;font-size:var(--font-sm);text-overflow:ellipsis;white-space:nowrap}header>button{display:grid;width:30px;height:30px;place-items:center;border:0;border-radius:8px;color:#89837a;background:transparent;cursor:pointer;font-size:22px}header>button:hover{background:#eeeae2}
.writing-content{display:grid;min-height:0;flex:1;grid-template-columns:220px minmax(0,1fr)}.writing-templates{display:grid;align-content:start;gap:5px;padding:14px 10px;border-right:1px solid #e2ddd4;background:#f0ede6}.writing-templates>button{display:flex;min-height:57px;align-items:center;gap:10px;padding:8px 9px;border:1px solid transparent;border-radius:10px;color:#656057;background:transparent;cursor:pointer;text-align:left}.writing-templates>button:hover{background:rgb(255 255 255 / 55%)}.writing-templates>button.active{border-color:#dce2dc;color:#36523e;background:#fffefa;box-shadow:0 2px 8px rgb(52 47 38 / 6%)}.writing-templates>button>span{display:grid;width:30px;height:30px;flex:0 0 auto;place-items:center;border-radius:8px;color:#55705d;background:#e6ede7;font-family:"Songti SC","SimSun",serif}.writing-templates div{display:flex;min-width:0;flex-direction:column}.writing-templates strong{font-size:var(--font-md)}.writing-templates small{margin-top:3px;overflow:hidden;color:#969087;font-size:var(--font-xs);text-overflow:ellipsis;white-space:nowrap}
.writing-main{display:flex;min-width:0;min-height:0;flex-direction:column;padding:18px 20px}.writing-main>label{display:grid;gap:7px}.writing-main>label>span{font-size:var(--font-md);font-weight:600}.writing-main textarea{width:100%;resize:none;padding:10px 11px;border:1px solid #dcd7cd;border-radius:9px;outline:0;color:#454139;background:#fff;font-size:var(--font-md);line-height:1.6}.writing-main textarea:focus{border-color:#85988a;box-shadow:0 0 0 3px rgb(77 102 84 / 9%)}.writing-boundary{display:flex;align-items:center;gap:6px;margin:8px 0;color:#7d877f;font-size:var(--font-xs)}.writing-boundary i{width:6px;height:6px;border-radius:50%;background:#6e9077}.generate-button{align-self:flex-start;height:34px;padding:0 14px;border:0;border-radius:8px;color:#fff;background:#4d6654;cursor:pointer;font-size:var(--font-sm);font-weight:600}.generate-button:disabled{opacity:.6;cursor:default}
.writing-output{min-height:0;flex:1;overflow-y:auto;margin-top:12px;padding:16px 18px;border:1px solid #e4dfd6;border-radius:10px;background:#fffefa}.writing-markdown{color:#49453e;font-family:"Songti SC",STSong,Georgia,"SimSun",serif;font-size:var(--font-lg);line-height:1.8;overflow-wrap:anywhere}.writing-markdown :deep(:first-child){margin-top:0}.writing-markdown :deep(:last-child){margin-bottom:0}.writing-markdown :deep(pre){overflow:auto;padding:12px;border-radius:7px;color:#eee;background:#302f2b}.writing-empty,.writing-loading{display:grid;height:100%;place-content:center;justify-items:center;color:#989187;text-align:center}.writing-empty>span{display:grid;width:42px;height:42px;place-items:center;border-radius:13px;color:#55705d;background:#e8eee8;font-size:18px}.writing-empty p{max-width:320px;margin:10px 0 0;font-size:var(--font-sm);line-height:1.65}.writing-loading{grid-template-columns:repeat(3,5px) auto;gap:4px}.writing-loading i{width:5px;height:5px;border-radius:50%;background:#6f8575;animation:pulse 1s infinite}.writing-loading i:nth-child(2){animation-delay:.15s}.writing-loading i:nth-child(3){animation-delay:.3s}.writing-loading span{margin-left:6px;font-size:var(--font-sm)}@keyframes pulse{50%{opacity:.3;transform:translateY(-2px)}}.writing-error{margin:7px 0 0;color:#a14c45;font-size:var(--font-sm)}
footer{display:flex;min-height:58px;align-items:center;justify-content:space-between;padding:10px 19px;border-top:1px solid #e5e1d8}footer>span{color:#989187;font-size:var(--font-xs)}footer>div{display:flex;gap:7px}footer button,.writing-disabled button{height:34px;padding:0 13px;border-radius:8px;cursor:pointer;font-size:var(--font-sm);font-weight:600}footer button:disabled{opacity:.45;cursor:default}.secondary{border:1px solid #dcd7cd;background:#fffefa}.primary,.writing-disabled button{border:1px solid #4d6654;color:#fff;background:#4d6654}
.writing-disabled{display:grid;flex:1;place-content:center;justify-items:center;text-align:center}.writing-disabled>span{display:grid;width:54px;height:54px;place-items:center;border-radius:16px;color:#53705b;background:#e6ede7;font-size:22px}.writing-disabled h3{margin:14px 0 0}.writing-disabled p{margin:7px 0 16px;color:#888178;font-size:var(--font-sm)}
@media(max-width:720px){.writing-backdrop{padding:14px}.writing-content{grid-template-columns:150px minmax(0,1fr)}.writing-templates small{display:none}.writing-main{padding:14px}.writing-dialog{height:calc(100vh - 28px)}footer>span{display:none}}
header>div>span,.generate-button,.primary,.writing-disabled button{border-color:var(--accent-solid);background:var(--accent-solid)}
.writing-templates>button.active{border-color:var(--accent-border);color:var(--accent-strong);background:var(--accent-softest)}
.writing-templates>button>span,.writing-empty>span,.writing-disabled>span{color:var(--accent-strong);background:var(--accent-soft)}
.writing-main textarea:focus{border-color:var(--accent);box-shadow:0 0 0 3px rgb(232 111 22 / 11%)}
.writing-boundary i,.writing-loading i{background:var(--accent)}
</style>
