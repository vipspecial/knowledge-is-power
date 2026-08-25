<script setup lang="ts">
import { getVersion } from "@tauri-apps/api/app";
import { computed, onMounted, ref, watch } from "vue";
import packageInfo from "../../package.json";
import {
  aiProviderPresets,
  findAiProvider,
  type AiProviderRegion,
} from "../aiProviders";
import {
  chooseDocumentDirectory,
  cloneAppSettings,
  clearApiKey,
  getMcpSetupInfo,
  listAiModels,
  setMcpEnabled,
  testAiConnection,
} from "../settings";
import {
  checkForUpdates,
  downloadAndInstallUpdate,
  relaunchApp,
  type Update,
} from "../updater";
import type { AiProtocol, AppSettings, McpSetupInfo, NotesStore } from "../types";

const props = defineProps<{
  settings: AppSettings;
  hasApiKey: boolean;
  credentialError?: string;
  store: NotesStore;
  saving: boolean;
  initialTab?: SettingsTab;
}>();

const emit = defineEmits<{
  close: [];
  save: [settings: AppSettings, apiKey: string];
  directoryChanged: [path: string];
  keyCleared: [];
}>();

type SettingsTab = "general" | "ai" | "storage" | "mcp" | "about";

const activeTab = ref<SettingsTab>(props.initialTab ?? "general");
const draft = ref<AppSettings>(cloneAppSettings(props.settings));
const apiKey = ref("");
const showApiKey = ref(false);
const directoryLoading = ref(false);
const directoryMessage = ref("");
const testState = ref<"idle" | "testing" | "success" | "error">("idle");
const testMessage = ref("");
const appVersion = ref("");
const modelCandidate = ref("");
const discoveredModels = ref<string[]>([]);
const modelsLoading = ref(false);
const modelMessage = ref("");
const mcpSetup = ref<McpSetupInfo | null>(null);
const mcpLoading = ref(true);
const mcpSaving = ref(false);
const mcpMessage = ref("");
const mcpCopied = ref(false);
const updateState = ref<"idle" | "checking" | "available" | "downloading" | "restart" | "upToDate" | "error">("idle");
const updateMessage = ref("");
const updateVersion = ref("");
const updateNotes = ref("");
const updateProgress = ref(0);
let pendingUpdate: Update | null = null;
const originalProvider = props.settings.ai.provider;
const providerRegion = ref<AiProviderRegion>(findAiProvider(draft.value.ai.provider).region);

const visibleProviders = computed(() =>
  aiProviderPresets.filter((provider) => provider.region === providerRegion.value),
);
const selectedProvider = computed(() => findAiProvider(draft.value.ai.provider));
const availableProtocols = computed(() => selectedProvider.value.protocols);
const originalCredentialScope = credentialScope(originalProvider, props.settings.ai.baseUrl);
const credentialScopeChanged = computed(() =>
  credentialScope(draft.value.ai.provider, draft.value.ai.baseUrl) !== originalCredentialScope,
);
const mcpInstallInfo = computed(() => {
  const setup = mcpSetup.value;
  if (!setup?.executablePath || !setup.accessFilePath || !draft.value.documentDirectory) return "";
  return [
    "请在当前 AI 工具中安装并启用以下 stdio MCP 服务，同时保留已有 MCP 配置。",
    "",
    `服务名称：${setup.serviceName}`,
    `可执行文件：${setup.executablePath}`,
    "启动参数：",
    "--mcp",
    "--directory",
    draft.value.documentDirectory,
    "--access-file",
    setup.accessFilePath,
    "",
    "该服务仅用于列出知识库、搜索文档和读取指定文档。完成后请刷新 MCP 连接。",
  ].join("\n");
});

const providerRegions: readonly { id: AiProviderRegion; label: string }[] = [
  { id: "china", label: "国内服务" },
  { id: "global", label: "国外服务" },
  { id: "custom", label: "自定义 API" },
];
const settingsTabCopy: Record<SettingsTab, { title: string; description: string }> = {
  general: { title: "通用设置", description: "调整文档保存行为。" },
  ai: { title: "AI 助手", description: "选择国内外主流服务，或接入自定义 API。" },
  storage: { title: "文档存储", description: "笔记会以开放文件保存在指定目录。" },
  mcp: { title: "MCP", description: "通过标准 MCP 协议开放知识库的只读能力。" },
  about: { title: "关于", description: "本地优先的 AI 知识库应用。" },
};
const activeTabCopy = computed(() => settingsTabCopy[activeTab.value]);

function protocolLabel(protocol: AiProtocol): string {
  if (protocol === "responses") return "Responses API";
  if (protocol === "anthropic") return "Anthropic Messages";
  return "Chat Completions";
}

function credentialScope(provider: string, baseUrl: string): string {
  try {
    return `${provider}:${new URL(baseUrl).host.toLowerCase()}`;
  } catch {
    return `${provider}:${baseUrl.trim().toLowerCase().replace(/\/+$/, "")}`;
  }
}

function resetConnectionTest(): void {
  testState.value = "idle";
  testMessage.value = "";
}

function resetModelDiscovery(): void {
  discoveredModels.value = [];
  modelCandidate.value = "";
  modelMessage.value = "";
}

function selectProvider(providerId: string): void {
  const provider = findAiProvider(providerId);
  draft.value.ai.provider = provider.id;
  draft.value.ai.baseUrl = provider.baseUrl;
  draft.value.ai.model = provider.model;
  draft.value.ai.models = [provider.model];
  draft.value.ai.protocol = provider.protocols[0];
  apiKey.value = "";
  showApiKey.value = false;
  resetConnectionTest();
  resetModelDiscovery();
}

function selectProviderFromEvent(event: Event): void {
  selectProvider((event.target as HTMLSelectElement).value);
}

function selectProviderRegion(region: AiProviderRegion): void {
  if (providerRegion.value === region) return;
  providerRegion.value = region;
  const firstProvider = aiProviderPresets.find((provider) => provider.region === region);
  if (firstProvider) selectProvider(firstProvider.id);
}

function validateProviderKeyChange(): boolean {
  if (!draft.value.ai.enabled || !credentialScopeChanged.value || !props.hasApiKey || apiKey.value.trim()) {
    return true;
  }
  testState.value = "error";
  testMessage.value = "服务商或 API 域名变化后请填写新 Key；无 Key 服务请先移除旧密钥。";
  return false;
}

function addConfiguredModel(value = modelCandidate.value): void {
  const model = value.trim();
  if (!model) {
    modelMessage.value = "请先填写或选择模型名称。";
    return;
  }
  if (model.length > 200) {
    modelMessage.value = "模型名称不能超过 200 个字符。";
    return;
  }
  if (!draft.value.ai.models.includes(model)) {
    if (draft.value.ai.models.length >= 20) {
      modelMessage.value = "最多配置 20 个模型。";
      return;
    }
    draft.value.ai.models.push(model);
  }
  draft.value.ai.model = model;
  modelCandidate.value = "";
  modelMessage.value = `已加入并切换到 ${model}`;
}

function removeConfiguredModel(model: string): void {
  if (draft.value.ai.models.length <= 1) {
    modelMessage.value = "至少保留一个模型。";
    return;
  }
  draft.value.ai.models = draft.value.ai.models.filter((item) => item !== model);
  if (draft.value.ai.model === model) {
    draft.value.ai.model = draft.value.ai.models[0];
  }
  modelMessage.value = `已移除 ${model}`;
}

async function fetchModels(): Promise<void> {
  if (!validateProviderKeyChange()) {
    modelMessage.value = testMessage.value;
    return;
  }
  modelsLoading.value = true;
  modelMessage.value = "正在获取模型列表…";
  try {
    discoveredModels.value = await listAiModels(draft.value.ai, apiKey.value);
    modelMessage.value = discoveredModels.value.length
      ? `已获取 ${discoveredModels.value.length} 个模型，点击即可加入或切换。`
      : "服务已响应，但没有返回可用模型。";
  } catch (error) {
    discoveredModels.value = [];
    modelMessage.value = `获取失败：${String(error)}`;
  } finally {
    modelsLoading.value = false;
  }
}

function submit(): void {
  if (!validateProviderKeyChange()) return;
  emit("save", cloneAppSettings(draft.value), apiKey.value);
}

async function chooseDirectory(): Promise<void> {
  directoryLoading.value = true;
  directoryMessage.value = "";
  try {
    const path = await chooseDocumentDirectory(props.store);
    if (path) {
      draft.value.documentDirectory = path;
      emit("directoryChanged", path);
      await refreshMcpSetup();
      directoryMessage.value = "文档已复制，新目录已启用。";
    }
  } catch (error) {
    directoryMessage.value = `目录设置失败：${String(error)}`;
  } finally {
    directoryLoading.value = false;
  }
}

async function testConnection(): Promise<void> {
  if (!validateProviderKeyChange()) return;
  testState.value = "testing";
  testMessage.value = "正在连接模型…";
  try {
    const message = await testAiConnection(draft.value.ai, apiKey.value);
    testState.value = "success";
    testMessage.value = message.trim() || "连接成功";
  } catch (error) {
    testState.value = "error";
    testMessage.value = String(error);
  }
}

async function removeApiKey(): Promise<void> {
  try {
    await clearApiKey();
    apiKey.value = "";
    emit("keyCleared");
    testState.value = "idle";
    testMessage.value = "已移除本地加密存储的 API Key";
  } catch (error) {
    testState.value = "error";
    testMessage.value = `移除失败：${String(error)}`;
  }
}

async function checkForAppUpdates(): Promise<void> {
  updateState.value = "checking";
  updateMessage.value = "正在检查更新…";
  updateVersion.value = "";
  updateNotes.value = "";
  updateProgress.value = 0;
  pendingUpdate = null;
  try {
    const update = await checkForUpdates();
    if (update) {
      pendingUpdate = update;
      updateVersion.value = update.version;
      updateNotes.value = update.body ?? "";
      updateState.value = "available";
      updateMessage.value = `发现新版本 ${update.version}，可下载安装`;
    } else {
      updateState.value = "upToDate";
      updateMessage.value = "已是最新版本";
    }
  } catch (error) {
    updateState.value = "error";
    updateMessage.value = `检查更新失败：${String(error)}`;
  }
}

async function downloadAndInstallUpdateClick(): Promise<void> {
  if (!pendingUpdate || updateState.value === "downloading" || updateState.value === "restart") return;
  const update = pendingUpdate;
  updateState.value = "downloading";
  updateMessage.value = "正在下载更新…";
  try {
    await downloadAndInstallUpdate(update, (percent) => {
      updateProgress.value = percent;
    });
    updateState.value = "restart";
    updateMessage.value = "更新已安装，重启后生效";
  } catch (error) {
    updateState.value = "error";
    updateMessage.value = `更新失败：${String(error)}`;
  }
}

async function restartApp(): Promise<void> {
  try {
    await relaunchApp();
  } catch (error) {
    updateState.value = "error";
    updateMessage.value = `重启失败：${String(error)}`;
  }
}

function selectTab(tab: SettingsTab): void {
  activeTab.value = tab;
}

async function refreshMcpSetup(): Promise<void> {
  mcpLoading.value = true;
  try {
    mcpSetup.value = await getMcpSetupInfo();
    mcpMessage.value = "";
  } catch (error) {
    mcpMessage.value = `无法读取 MCP 设置：${String(error)}`;
  } finally {
    mcpLoading.value = false;
  }
}

async function toggleMcp(event: Event): Promise<void> {
  const enabled = (event.target as HTMLInputElement).checked;
  mcpSaving.value = true;
  mcpCopied.value = false;
  try {
    mcpSetup.value = await setMcpEnabled(enabled);
    mcpMessage.value = enabled
      ? "MCP 已开启，可以复制安装信息。"
      : "MCP 已关闭，已有连接将被拒绝。";
  } catch (error) {
    (event.target as HTMLInputElement).checked = mcpSetup.value?.enabled ?? false;
    mcpMessage.value = `MCP 设置失败：${String(error)}`;
  } finally {
    mcpSaving.value = false;
  }
}

async function copyMcpInstallInfo(): Promise<void> {
  const content = mcpInstallInfo.value;
  if (!content) {
    mcpMessage.value = "请在桌面应用中开启 MCP 后再复制安装信息。";
    return;
  }
  try {
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(content);
    } else {
      const textarea = document.createElement("textarea");
      textarea.value = content;
      textarea.style.position = "fixed";
      textarea.style.opacity = "0";
      document.body.appendChild(textarea);
      textarea.select();
      const copied = document.execCommand("copy");
      textarea.remove();
      if (!copied) throw new Error("浏览器拒绝写入剪贴板");
    }
    mcpCopied.value = true;
    mcpMessage.value = "安装信息已复制，粘贴给支持 MCP 的 AI 即可完成接入。";
  } catch (error) {
    mcpCopied.value = false;
    mcpMessage.value = `复制失败：${String(error)}`;
  }
}

watch(
  () => [draft.value.ai.baseUrl, draft.value.ai.model, draft.value.ai.protocol, apiKey.value],
  resetConnectionTest,
);

onMounted(async () => {
  appVersion.value = "__TAURI_INTERNALS__" in window ? await getVersion() : packageInfo.version;
  await refreshMcpSetup();
});
</script>

<template>
  <div class="settings-backdrop" @click.self="emit('close')">
    <section class="settings-dialog" role="dialog" aria-modal="true" aria-labelledby="settings-title">
      <aside class="settings-nav">
        <div class="settings-brand">
          <img src="/logo.svg" alt="" />
          <div>
            <strong id="settings-title">设置</strong>
            <span>应用与服务配置</span>
          </div>
        </div>
        <nav>
          <button :class="{ active: activeTab === 'general' }" type="button" @click="selectTab('general')">
            <span>⌘</span>通用
          </button>
          <button :class="{ active: activeTab === 'ai' }" type="button" @click="selectTab('ai')">
            <span>✦</span>AI 助手
          </button>
          <button :class="{ active: activeTab === 'storage' }" type="button" @click="selectTab('storage')">
            <span>▱</span>文档存储
          </button>
          <button :class="{ active: activeTab === 'mcp' }" type="button" @click="selectTab('mcp')">
            <span>⌁</span>MCP
          </button>
          <button :class="{ active: activeTab === 'about' }" type="button" @click="selectTab('about')">
            <span>i</span>关于
          </button>
        </nav>
      </aside>

      <form class="settings-content" @submit.prevent="submit">
        <header>
          <div>
            <h2>{{ activeTabCopy.title }}</h2>
            <p>{{ activeTabCopy.description }}</p>
          </div>
          <button class="settings-close" type="button" aria-label="关闭设置" @click="emit('close')">×</button>
        </header>

        <div class="settings-scroll">
          <section v-if="activeTab === 'general'" class="settings-section">
            <div class="setting-row vertical">
              <div>
                <label for="save-delay">自动保存等待时间</label>
                <p>停止输入后 {{ draft.general.autoSaveDelayMs }} 毫秒保存到本地文件。</p>
              </div>
              <input id="save-delay" v-model.number="draft.general.autoSaveDelayMs" type="range" min="300" max="3000" step="50" />
            </div>
          </section>

          <section v-else-if="activeTab === 'ai'" class="settings-section ai-settings">
            <label class="switch-row">
              <span>
                <strong>启用 AI 助手</strong>
                <small>在标题、标签、选区和正文旁显示场景化入口，右侧保留当前文档问答。</small>
              </span>
              <input v-model="draft.ai.enabled" type="checkbox" />
              <i></i>
            </label>

            <div class="ai-config" :class="{ disabled: !draft.ai.enabled }">
              <div class="provider-row">
                <div class="provider-regions" role="tablist" aria-label="AI 服务类型">
                  <button
                    v-for="region in providerRegions"
                    :key="region.id"
                    :class="{ active: providerRegion === region.id }"
                    :disabled="!draft.ai.enabled"
                    type="button"
                    role="tab"
                    :aria-selected="providerRegion === region.id"
                    @click="selectProviderRegion(region.id)"
                  >
                    {{ region.label }}
                  </button>
                </div>
                <label class="field">
                  <span>模型服务</span>
                  <select
                    :value="draft.ai.provider"
                    :disabled="!draft.ai.enabled"
                    @change="selectProviderFromEvent"
                  >
                    <option v-for="provider in visibleProviders" :key="provider.id" :value="provider.id">
                      {{ provider.name }}
                    </option>
                  </select>
                </label>
              </div>
              <small class="provider-desc">{{ selectedProvider.description }}</small>

              <div class="ai-row">
                <label class="field">
                  <span>API 地址</span>
                  <input v-model.trim="draft.ai.baseUrl" :disabled="!draft.ai.enabled" type="url" required placeholder="https://api.example.com/v1" />
                </label>
                <label class="field">
                  <span>接口协议</span>
                  <select v-model="draft.ai.protocol" :disabled="!draft.ai.enabled">
                    <option v-for="protocol in availableProtocols" :key="protocol" :value="protocol">
                      {{ protocolLabel(protocol) }}
                    </option>
                  </select>
                </label>
              </div>

              <label class="field api-key-field">
                <span>API Key</span>
                <div>
                  <input
                    v-model="apiKey"
                    :disabled="!draft.ai.enabled"
                    :type="showApiKey ? 'text' : 'password'"
                    :placeholder="hasApiKey ? '已保存；当前服务未变时可留空' : '填写 API Key；本地服务可留空'"
                    autocomplete="off"
                  />
                  <button type="button" @click="showApiKey = !showApiKey">{{ showApiKey ? '隐藏' : '显示' }}</button>
                </div>
                <small>密钥经本机绑定的 AES-256-GCM 加密后存储在应用数据目录，不会明文写入设置文件或文档目录，也不会请求系统钥匙串权限。</small>
                <small v-if="credentialError" class="provider-key-warning">密钥存储不可用：{{ credentialError }}</small>
                <small v-if="credentialScopeChanged && hasApiKey" class="provider-key-warning">服务商或 API 域名已切换，请输入新 Key；无 Key 服务请先移除旧密钥。</small>
                <button v-if="hasApiKey" class="clear-key" type="button" @click="removeApiKey">移除已保存密钥</button>
              </label>

              <div class="field model-field">
                <span>模型</span>
                <div class="model-active-row">
                  <select v-model="draft.ai.model" :disabled="!draft.ai.enabled" aria-label="默认模型">
                    <option v-for="model in draft.ai.models" :key="model" :value="model">{{ model }}</option>
                  </select>
                  <button type="button" :disabled="!draft.ai.enabled || modelsLoading" @click="fetchModels">
                    {{ modelsLoading ? '正在获取…' : '获取模型列表' }}
                  </button>
                </div>
                <div v-if="discoveredModels.length" class="discovered-models" aria-label="获取到的模型列表">
                  <button
                    v-for="model in discoveredModels"
                    :key="model"
                    type="button"
                    :class="{ added: draft.ai.models.includes(model), current: draft.ai.model === model }"
                    :disabled="!draft.ai.enabled"
                    :title="draft.ai.models.includes(model) ? `切换到 ${model}` : `加入 ${model}`"
                    @click="addConfiguredModel(model)"
                  >
                    {{ model }}
                  </button>
                </div>
                <div class="model-add-row">
                  <input
                    v-model.trim="modelCandidate"
                    :disabled="!draft.ai.enabled"
                    type="text"
                    placeholder="接口不支持获取列表时，可手动输入模型 ID"
                    @keydown.enter.prevent="addConfiguredModel()"
                  />
                  <button type="button" :disabled="!draft.ai.enabled" @click="addConfiguredModel()">加入</button>
                </div>
                <div class="configured-models" aria-label="已配置模型">
                  <div v-for="model in draft.ai.models" :key="model" class="model-chip" :class="{ active: draft.ai.model === model }">
                    <button type="button" :disabled="!draft.ai.enabled" :title="`切换到 ${model}`" @click="draft.ai.model = model">{{ model }}</button>
                    <button type="button" :disabled="!draft.ai.enabled" :aria-label="`移除模型 ${model}`" @click="removeConfiguredModel(model)">×</button>
                  </div>
                </div>
                <small v-if="modelMessage" class="model-message">{{ modelMessage }}</small>
              </div>

              <div class="ai-row">
                <label class="field">
                  <span>创造性 {{ draft.ai.temperature.toFixed(1) }}</span>
                  <input v-model.number="draft.ai.temperature" :disabled="!draft.ai.enabled" type="range" min="0" max="2" step="0.1" />
                </label>
                <label class="field">
                  <span>文档读取上限</span>
                  <select v-model.number="draft.ai.maxContextChars" :disabled="!draft.ai.enabled">
                    <option :value="10000">约 10,000 字符</option>
                    <option :value="30000">约 30,000 字符</option>
                    <option :value="60000">约 60,000 字符</option>
                    <option :value="120000">约 120,000 字符</option>
                  </select>
                </label>
              </div>
            </div>

            <div class="connection-test">
              <button type="button" :disabled="!draft.ai.enabled || testState === 'testing'" @click="testConnection">
                {{ testState === 'testing' ? '正在测试…' : '测试连接' }}
              </button>
              <span v-if="testMessage" :class="testState">{{ testMessage }}</span>
            </div>
          </section>

          <section v-else-if="activeTab === 'storage'" class="settings-section storage-settings">
            <div class="storage-card">
              <div class="folder-icon">▰</div>
              <div>
                <strong>当前文档目录</strong>
                <code>{{ draft.documentDirectory || '首次启动时自动创建默认文档目录' }}</code>
              </div>
            </div>
            <button class="choose-directory" type="button" :disabled="directoryLoading" @click="chooseDirectory">
              {{ directoryLoading ? '正在复制文档…' : '选择其他目录' }}
            </button>
            <p v-if="directoryMessage" class="directory-message">{{ directoryMessage }}</p>
            <div class="storage-info">
              <strong>目录结构</strong>
              <p>每个知识库对应一个文件夹，每篇笔记都是可直接打开的 <code>.md</code> 文件。</p>
              <p>更换目录时会把当前知识库复制到新位置，原目录不会被删除。</p>
            </div>
          </section>

          <section v-else-if="activeTab === 'mcp'" class="settings-section mcp-settings">
            <label class="switch-row" :class="{ disabled: mcpLoading || mcpSaving }">
              <span>
                <strong>启用 MCP</strong>
                <small>默认关闭。开启后提供只读检索，不允许创建、修改或删除文档。</small>
              </span>
              <input
                :checked="mcpSetup?.enabled ?? false"
                :disabled="mcpLoading || mcpSaving"
                type="checkbox"
                @change="toggleMcp"
              />
              <i></i>
            </label>

            <div class="mcp-scope-card" :class="{ disabled: !mcpSetup?.enabled }">
              <div class="mcp-scope-icon" aria-hidden="true">⌁</div>
              <div>
                <strong>开放范围</strong>
                <p>当前文档目录中的全部知识库，不包含回收站。</p>
                <code>{{ draft.documentDirectory || '尚未设置文档目录' }}</code>
              </div>
            </div>

            <div class="mcp-install-card" :class="{ disabled: !mcpSetup?.enabled }">
              <div class="mcp-install-heading">
                <div>
                  <strong>安装</strong>
                  <p>复制一次安装信息，即可接入任何支持 MCP 的工具。</p>
                </div>
              </div>

              <p class="mcp-bundled-note">MCP 已随桌面应用内置，无需执行 npm 全局安装；应用升级时会一起更新。</p>

              <div class="mcp-copy-row">
                <span class="mcp-install-mark" aria-hidden="true">MCP</span>
                <div>
                  <strong>通用安装信息</strong>
                  <p>{{ mcpSetup?.serviceName }} · 复制后粘贴给目标 AI，或导入其 MCP 设置。</p>
                </div>
                <button
                  class="mcp-copy-button"
                  type="button"
                  :disabled="!mcpSetup?.enabled || !mcpInstallInfo"
                  @click="copyMcpInstallInfo"
                >{{ mcpCopied ? '已复制' : '复制安装信息' }}</button>
              </div>

              <ol class="mcp-steps">
                <li><span aria-hidden="true">①</span><p><strong>开启 MCP</strong>授权当前文档目录的只读访问。</p></li>
                <li><span aria-hidden="true">②</span><p><strong>复制信息</strong>应用自动带上路径与连接参数。</p></li>
                <li><span aria-hidden="true">③</span><p><strong>粘贴安装</strong>交给支持 MCP 的 AI 完成接入。</p></li>
              </ol>
            </div>

            <p v-if="mcpMessage" class="mcp-message" role="status">{{ mcpMessage }}</p>
            <p class="mcp-security-note">关闭 MCP 后，已经安装的连接也会立即失效。MCP 不会读取 AI 密钥。</p>
          </section>

          <section v-else class="settings-section about-settings">
            <div class="about-product">
              <img src="/logo.svg" alt="应用 Logo" />
              <div>
                <h3>拿了桔子跑啊</h3>
                <p>版本 {{ appVersion || '…' }} · Tauri 2 · Rust · Vue 3</p>
              </div>
            </div>
            <div class="update-card" :class="{ available: updateState === 'available' || updateState === 'restart' }">
              <div>
                <strong>版本更新</strong>
                <p>{{ updateMessage || '检查新版本并在应用内完成下载与安装。' }}</p>
              </div>
              <button
                v-if="updateState === 'idle' || updateState === 'error' || updateState === 'upToDate'"
                type="button"
                @click="checkForAppUpdates"
              >
                检查更新
              </button>
              <button
                v-else-if="updateState === 'available'"
                type="button"
                @click="downloadAndInstallUpdateClick"
              >
                下载并安装
              </button>
              <button
                v-else-if="updateState === 'restart'"
                type="button"
                @click="restartApp"
              >
                重启应用
              </button>
              <span v-else class="update-busy">{{ updateState === 'checking' ? '检查中…' : updateProgress > 0 ? `${updateProgress}%` : '下载中…' }}</span>
              <div v-if="updateState === 'downloading' && updateProgress > 0" class="update-progress"><i :style="{ width: `${updateProgress}%` }"></i></div>
              <p v-if="updateNotes && (updateState === 'available' || updateState === 'downloading')" class="update-notes">{{ updateNotes }}</p>
            </div>
            <p class="privacy-note">你的文档默认保存在本机；只有主动使用 AI 时，当前文档或选中内容才会发送到你配置的模型服务。</p>
          </section>
        </div>

        <footer>
          <span>设置保存在本机</span>
          <div>
            <button class="cancel" type="button" @click="emit('close')">取消</button>
            <button class="save" type="submit" :disabled="saving">{{ saving ? '保存中…' : '保存设置' }}</button>
          </div>
        </footer>
      </form>
    </section>
  </div>
</template>

<style scoped>
.settings-backdrop{position:fixed;z-index:50;inset:0;display:grid;place-items:center;padding:28px;background:rgb(29 28 24 / 38%);backdrop-filter:blur(7px)}
.settings-dialog{display:grid;grid-template-columns:188px minmax(0,1fr);width:min(850px,100%);height:min(650px,calc(100vh - 56px));overflow:hidden;border:1px solid rgb(255 255 255 / 65%);border-radius:18px;background:#fbfaf7;box-shadow:0 30px 90px rgb(28 25 20 / 28%)}
.settings-nav{padding:21px 13px;border-right:1px solid #ded9cf;background:#eeebe4}.settings-brand{display:flex;align-items:center;gap:10px;padding:0 7px 20px}.settings-brand img{width:35px;height:35px;border-radius:10px}.settings-brand div{display:flex;flex-direction:column}.settings-brand strong{font-size:16px}.settings-brand span{margin-top:2px;color:#979187;font-size:13px}.settings-nav nav{display:grid;gap:4px}.settings-nav nav button{display:flex;height:36px;align-items:center;gap:9px;padding:0 10px;border:0;border-radius:8px;color:#69645b;background:transparent;cursor:pointer;text-align:left;font-size:14px}.settings-nav nav button span{display:grid;width:19px;place-items:center;color:#818a82}.settings-nav nav button:hover{background:rgb(255 255 255 / 50%)}.settings-nav nav button.active{color:#344c3b;background:#fffefa;box-shadow:0 2px 8px rgb(52 47 38 / 7%);font-weight:650}
.settings-content{display:flex;min-width:0;min-height:0;flex-direction:column}.settings-content>header{display:flex;min-height:80px;align-items:center;justify-content:space-between;padding:18px 26px;border-bottom:1px solid #e8e4db}.settings-content h2,.settings-content p{margin:0}.settings-content h2{font-size:19px}.settings-content header p{margin-top:4px;color:#8d877d;font-size:13px}.settings-close{display:grid;width:30px;height:30px;place-items:center;border:0;border-radius:8px;color:#8b857b;background:transparent;cursor:pointer;font-size:22px}.settings-close:hover{background:#eeeae2}.settings-scroll{min-height:0;flex:1;overflow-y:auto;padding:24px 27px}.settings-section{display:grid;gap:14px}.setting-row{display:flex;align-items:center;justify-content:space-between;gap:30px;padding:17px 0;border-bottom:1px solid #ece8df}.setting-row.vertical{display:grid;gap:15px}.setting-row label,.field>span{color:#3f3b34;font-size:15px;font-weight:650}.setting-row p{margin-top:5px;color:#969086;font-size:13px}.setting-row select,.field select,.field input:not([type=range]){height:37px;padding:0 10px;border:1px solid #dcd7cd;border-radius:8px;outline:0;color:#49453e;background:#fff;font-size:14px}.setting-row input[type=range],.field input[type=range]{width:100%;accent-color:#4d6654}
.switch-row{display:flex;align-items:center;padding:15px;border:1px solid #dfe4dd;border-radius:11px;background:#f4f7f3;cursor:pointer}.switch-row>span{display:flex;min-width:0;flex:1;flex-direction:column}.switch-row strong{font-size:15px}.switch-row small{margin-top:4px;color:#7f887f;font-size:13px}.switch-row input{position:absolute;opacity:0}.switch-row i{position:relative;width:37px;height:21px;border-radius:14px;background:#c5c7c1;transition:background .15s}.switch-row i:after{position:absolute;top:3px;left:3px;width:15px;height:15px;border-radius:50%;background:#fff;box-shadow:0 1px 4px rgb(0 0 0 / 20%);content:"";transition:transform .15s}.switch-row input:checked+i{background:#52705b}.switch-row input:checked+i:after{transform:translateX(16px)}.field{display:grid;gap:7px}.ai-config{display:grid;gap:13px}.ai-config.disabled{opacity:.58}.ai-row{display:grid;grid-template-columns:1fr 1fr;gap:12px}.provider-row{display:grid;grid-template-columns:auto minmax(0,1fr);gap:12px;align-items:end}.provider-desc{color:#989187;font-size:13px}.field small{color:#989187;font-size:13px;line-height:1.45}.api-key-field>div{display:flex}.api-key-field input{min-width:0;flex:1;border-radius:8px 0 0 8px!important}.api-key-field>div>button{padding:0 11px;border:1px solid #dcd7cd;border-left:0;border-radius:0 8px 8px 0;color:#69645b;background:#f3f0e9;cursor:pointer;font-size:13px}.clear-key{justify-self:start;padding:0;border:0;color:#a34f47;background:transparent;cursor:pointer;font-size:13px}.connection-test{display:flex;align-items:center;gap:11px;margin-top:7px}.connection-test button,.choose-directory{height:35px;padding:0 13px;border:1px solid #cfd8d0;border-radius:8px;color:#3e5b47;background:#eef3ee;cursor:pointer;font-size:13px;font-weight:650}.connection-test button:disabled,.choose-directory:disabled{opacity:.55;cursor:default}.connection-test span{min-width:0;overflow:hidden;color:#7e786f;font-size:13px;text-overflow:ellipsis;white-space:nowrap}.connection-test span.success{color:#3e7650}.connection-test span.error{color:#a34b43}
.provider-regions{display:flex;gap:4px;padding:3px;border-radius:9px;background:#f1ede6}.provider-regions button{height:31px;padding:0 12px;border:0;border-radius:7px;color:#7b746a;background:transparent;cursor:pointer;font-size:13px}.provider-regions button.active{color:var(--accent-strong);background:#fff;box-shadow:0 2px 7px rgb(61 47 31 / 8%);font-weight:700}.provider-regions button:disabled{cursor:default}.provider-key-warning{color:#a65a3e!important}
.model-active-row,.model-add-row{display:flex;gap:7px}.model-active-row select,.model-add-row input{min-width:0;flex:1}.model-active-row button,.model-add-row button{height:37px;flex:0 0 auto;padding:0 11px;border:1px solid var(--accent-border);border-radius:8px;color:var(--accent-strong);background:var(--accent-softest);cursor:pointer;font-size:13px;font-weight:650}.model-active-row button:disabled,.model-add-row button:disabled{opacity:.55;cursor:default}.discovered-models{display:flex;flex-wrap:wrap;gap:5px;max-height:128px;overflow-y:auto;padding:8px;border:1px solid #e5e0d7;border-radius:8px;background:#faf8f4}.discovered-models button{height:27px;padding:0 9px;border:1px solid #ded9d0;border-radius:7px;color:#6f685f;background:#fff;cursor:pointer;font-size:12px}.discovered-models button:hover{border-color:var(--accent-border);color:var(--accent-strong)}.discovered-models button.added{border-color:var(--accent-border);background:var(--accent-softest);color:var(--accent-strong)}.discovered-models button.current{font-weight:700}.discovered-models button:disabled{cursor:default;opacity:.55}.configured-models{display:flex;flex-wrap:wrap;gap:5px}.model-chip{display:flex;min-width:0;max-width:100%;overflow:hidden;border:1px solid #ded9d0;border-radius:7px;background:#f7f4ee}.model-chip.active{border-color:var(--accent-border);background:var(--accent-softest)}.model-chip button{height:27px;border:0;color:#6f685f;background:transparent;cursor:pointer;font-size:12px}.model-chip button:first-child{min-width:0;overflow:hidden;padding:0 8px;text-overflow:ellipsis;white-space:nowrap}.model-chip button:last-child{width:24px;flex:0 0 auto;border-left:1px solid rgb(0 0 0 / 6%);color:#9a7567}.model-chip.active button:first-child{color:var(--accent-strong);font-weight:700}.model-message{color:var(--accent-strong)!important}
.model-chip button:disabled{cursor:default;opacity:.55}
.storage-card{display:flex;align-items:center;gap:13px;padding:16px;border:1px solid #e2ded5;border-radius:11px;background:#fff}.folder-icon{display:grid;width:40px;height:40px;place-items:center;border-radius:10px;color:#5c735f;background:#e9efe9}.storage-card>div:last-child{display:grid;min-width:0;gap:5px}.storage-card strong,.storage-info strong{font-size:14px}.storage-card code{overflow:hidden;color:#777168;font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:13px;text-overflow:ellipsis;white-space:nowrap}.choose-directory{justify-self:start}.storage-info{margin-top:10px;padding:16px;border-radius:10px;color:#767067;background:#f2efe8}.storage-info p{margin-top:7px;font-size:13px;line-height:1.55}.about-settings{padding:14px 4px}.about-product{display:flex;align-items:center;gap:14px;text-align:left}.about-product img{width:58px;height:58px;border-radius:16px;box-shadow:0 10px 30px rgb(50 66 54 / 16%)}.about-product h3{margin:0;font-size:17px}.about-product p,.privacy-note{margin-top:5px;color:#7f796f;font-size:13px;line-height:1.65}.update-card{position:relative;display:grid;grid-template-columns:1fr auto;gap:5px 16px;margin-top:7px;padding:16px;border:1px solid #e3ded4;border-radius:12px;background:#fff;text-align:left}.update-card>div:first-child{min-width:0}.update-card strong{font-size:14px}.update-card p{margin:5px 0 0;color:#878076;font-size:13px;line-height:1.55}.update-card>button{height:33px;align-self:center;padding:0 12px;border:1px solid var(--accent-border);border-radius:8px;color:var(--accent-strong);background:var(--accent-softest);cursor:pointer;font-size:13px;font-weight:700}.update-card>button:disabled{opacity:.58;cursor:default}.update-card.available{border-color:var(--accent-border);background:#fffaf5}
.switch-row.disabled{cursor:default;opacity:.6}
.mcp-scope-card{display:flex;align-items:flex-start;gap:12px;padding:14px;border:1px solid #e3ded4;border-radius:11px;background:#fff;transition:opacity .15s}.mcp-scope-card.disabled,.mcp-install-card.disabled{opacity:.55}.mcp-scope-icon{display:grid;width:36px;height:36px;flex:0 0 auto;place-items:center;border-radius:10px;color:var(--accent-strong);background:var(--accent-soft);font-size:20px}.mcp-scope-card>div:last-child{display:grid;min-width:0;gap:4px}.mcp-scope-card strong,.mcp-install-heading strong,.mcp-copy-row strong{font-size:14px}.mcp-scope-card p,.mcp-install-heading p,.mcp-copy-row p{margin:0;color:#827b72;font-size:13px;line-height:1.5}.mcp-scope-card code{overflow:hidden;color:#6f6960;font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:12px;text-overflow:ellipsis;white-space:nowrap}.mcp-install-card{display:grid;gap:12px;padding:15px;border:1px solid #e3ded4;border-radius:11px;background:#fff;transition:opacity .15s}.mcp-install-heading{display:flex;align-items:center;justify-content:space-between;gap:14px}.mcp-install-heading>div{display:grid;gap:4px}.mcp-copy-row{display:grid;grid-template-columns:42px minmax(0,1fr) auto;align-items:center;gap:12px;padding:13px;border:1px solid #e7e1d8;border-radius:10px;background:#faf8f4}.mcp-copy-row>div{display:grid;min-width:0;gap:3px}.mcp-install-mark{display:grid;width:42px;height:42px;place-items:center;border-radius:11px;color:var(--accent-strong);background:var(--accent-soft);font-size:11px;font-weight:800;letter-spacing:.04em}.mcp-copy-button{height:34px;flex:0 0 auto;padding:0 13px;border:1px solid var(--accent-border);border-radius:8px;color:var(--accent-strong);background:var(--accent-softest);cursor:pointer;font-size:13px;font-weight:700}.mcp-copy-button:disabled{cursor:default;opacity:.55}.mcp-steps{display:grid;grid-template-columns:repeat(3,1fr);gap:9px;margin:0;padding:0;list-style:none}.mcp-steps li{display:flex;gap:7px;align-items:flex-start}.mcp-steps li>span{flex:0 0 auto;color:var(--accent-strong);font-size:16px;line-height:1.2}.mcp-steps p{margin:0;color:#858077;font-size:12px;line-height:1.45}.mcp-steps strong{display:block;color:#5c564e;font-size:12px}.mcp-message{color:var(--accent-strong)!important;font-size:13px!important}.mcp-security-note{padding:11px 13px;border-radius:9px;color:#7c756c;background:#f3efe8;font-size:12px!important;line-height:1.55}
.mcp-bundled-note{margin:0;color:#777168;font-size:12px;line-height:1.5}
.update-busy{align-self:center;color:var(--accent-strong);font-size:13px;font-weight:700}.update-progress{grid-column:1/-1;height:4px!important;margin-top:7px;overflow:hidden;border-radius:99px;background:#eee8df}.update-progress i{display:block;height:100%;border-radius:inherit;background:var(--accent-solid);transition:width .18s}.update-notes{grid-column:1/-1;max-height:64px;overflow:auto;white-space:pre-line}.privacy-note{max-width:500px;margin:3px 0 0!important}
.directory-message{margin:0;color:#54715c!important;font-size:13px!important}
.settings-content>footer{display:flex;min-height:63px;align-items:center;justify-content:space-between;padding:12px 25px;border-top:1px solid #e8e4db}.settings-content footer>span{color:#aaa399;font-size:13px}.settings-content footer div{display:flex;gap:8px}.settings-content footer button{height:34px;padding:0 14px;border-radius:8px;cursor:pointer;font-size:13px;font-weight:650}.settings-content footer .cancel{border:1px solid #ddd8cf;background:#fffefa}.settings-content footer .save{border:1px solid #4d6654;color:#fff;background:#4d6654}.settings-content footer .save:disabled{opacity:.55}
@media(max-width:760px){.settings-dialog{grid-template-columns:145px minmax(0,1fr)}.settings-nav{padding:18px 8px}.settings-brand{padding-left:5px}.settings-scroll{padding:20px}.ai-row{grid-template-columns:1fr}.provider-row{grid-template-columns:1fr}}
@media(max-width:760px){.mcp-steps{grid-template-columns:1fr}.mcp-copy-row{grid-template-columns:42px minmax(0,1fr)}.mcp-copy-button{grid-column:1/-1}}
.settings-nav nav button.active{color:var(--accent-strong);background:var(--accent-softest)}
.setting-row input[type=range],.field input[type=range]{accent-color:var(--accent)}
.switch-row{border-color:#ead8c8;background:#fffaf4}
.switch-row input:checked+i{background:var(--accent-solid)}
.connection-test button,.choose-directory{border-color:var(--accent-border);color:var(--accent-strong);background:var(--accent-softest)}
.folder-icon{color:var(--accent-strong);background:var(--accent-soft)}
.directory-message{color:var(--accent-strong)!important}
.settings-content footer .save{border-color:var(--accent-solid);background:var(--accent-solid)}
</style>
