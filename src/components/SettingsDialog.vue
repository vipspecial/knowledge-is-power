<script setup lang="ts">
import { onMounted, ref } from "vue";
import {
  chooseDocumentDirectory,
  cloneAppSettings,
  clearApiKey,
  testAiConnection,
} from "../settings";
import type { AppSettings, NotesStore } from "../types";
import {
  checkForAppUpdate,
  getCurrentAppVersion,
  installCheckedUpdate,
  updaterAvailable,
  type AppUpdateInfo,
} from "../updater";

const props = defineProps<{
  settings: AppSettings;
  hasApiKey: boolean;
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

type SettingsTab = "general" | "ai" | "storage" | "about";

const activeTab = ref<SettingsTab>(props.initialTab ?? "general");
const draft = ref<AppSettings>(cloneAppSettings(props.settings));
const apiKey = ref("");
const showApiKey = ref(false);
const directoryLoading = ref(false);
const directoryMessage = ref("");
const testState = ref<"idle" | "testing" | "success" | "error">("idle");
const testMessage = ref("");
const appVersion = ref("");
const updateInfo = ref<AppUpdateInfo | null>(null);
const updateState = ref<"idle" | "checking" | "current" | "available" | "downloading" | "error">("idle");
const updateMessage = ref("");
const updateProgress = ref<number | null>(null);

function submit(): void {
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
      directoryMessage.value = "文档已复制，新目录已启用。";
    }
  } catch (error) {
    directoryMessage.value = `目录设置失败：${String(error)}`;
  } finally {
    directoryLoading.value = false;
  }
}

async function testConnection(): Promise<void> {
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
  await clearApiKey();
  apiKey.value = "";
  emit("keyCleared");
  testState.value = "idle";
  testMessage.value = "已从本地应用配置中移除 API Key";
}

function selectTab(tab: SettingsTab): void {
  activeTab.value = tab;
  if (tab === "about" && updateState.value === "idle") void checkUpdate(false);
}

async function checkUpdate(showCurrentMessage = true): Promise<void> {
  if (!updaterAvailable()) {
    updateState.value = "error";
    updateMessage.value = "请在安装后的桌面应用中检查更新";
    return;
  }

  updateState.value = "checking";
  updateMessage.value = "正在连接 GitHub Releases…";
  try {
    updateInfo.value = await checkForAppUpdate();
    if (updateInfo.value) {
      updateState.value = "available";
      updateMessage.value = `发现新版本 ${updateInfo.value.version}`;
    } else {
      updateState.value = "current";
      updateMessage.value = showCurrentMessage ? "当前已是最新版本" : "已是最新版本";
    }
  } catch (error) {
    updateState.value = "error";
    updateMessage.value = `检查失败：${String(error)}`;
  }
}

async function installUpdate(): Promise<void> {
  updateState.value = "downloading";
  updateProgress.value = 0;
  updateMessage.value = "正在下载更新…";
  try {
    await installCheckedUpdate((percent) => {
      updateProgress.value = percent;
      updateMessage.value = percent === 100 ? "安装完成，正在重新启动…" : "正在下载并验证更新…";
    });
  } catch (error) {
    updateState.value = "error";
    updateMessage.value = `更新失败：${String(error)}`;
  }
}

onMounted(async () => {
  appVersion.value = await getCurrentAppVersion();
  if (activeTab.value === "about") await checkUpdate(false);
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
          <button :class="{ active: activeTab === 'about' }" type="button" @click="selectTab('about')">
            <span>i</span>关于
          </button>
        </nav>
      </aside>

      <form class="settings-content" @submit.prevent="submit">
        <header>
          <div>
            <h2>
              {{ activeTab === 'general' ? '通用设置' : activeTab === 'ai' ? 'AI 助手' : activeTab === 'storage' ? '文档存储' : '关于' }}
            </h2>
            <p v-if="activeTab === 'general'">调整文档保存行为。</p>
            <p v-else-if="activeTab === 'ai'">配置兼容的模型服务，不再请求系统钥匙串权限。</p>
            <p v-else-if="activeTab === 'storage'">笔记会以开放文件保存在指定目录。</p>
            <p v-else>本地优先的 AI 知识库应用。</p>
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

            <div class="field-grid" :class="{ disabled: !draft.ai.enabled }">
              <label class="field full">
                <span>接口协议</span>
                <select v-model="draft.ai.protocol" :disabled="!draft.ai.enabled">
                  <option value="chatCompletions">Chat Completions（兼容性最佳）</option>
                  <option value="responses">Responses API（OpenAI）</option>
                </select>
              </label>
              <label class="field full">
                <span>API 地址</span>
                <input v-model.trim="draft.ai.baseUrl" :disabled="!draft.ai.enabled" type="url" placeholder="https://api.openai.com/v1" />
                <small>也支持 OpenAI-compatible 服务，例如 Ollama 的 http://localhost:11434/v1。</small>
              </label>
              <label class="field full">
                <span>模型名称</span>
                <input v-model.trim="draft.ai.model" :disabled="!draft.ai.enabled" type="text" placeholder="gpt-5.6" />
              </label>
              <label class="field full api-key-field">
                <span>API Key</span>
                <div>
                  <input
                    v-model="apiKey"
                    :disabled="!draft.ai.enabled"
                    :type="showApiKey ? 'text' : 'password'"
                    :placeholder="hasApiKey ? '已安全保存；留空表示不修改' : 'sk-…（Ollama 可留空）'"
                    autocomplete="off"
                  />
                  <button type="button" @click="showApiKey = !showApiKey">{{ showApiKey ? '隐藏' : '显示' }}</button>
                </div>
                <small v-if="hasApiKey">密钥已保存在本机应用配置中，不会访问系统钥匙串。</small>
                <small>本地配置文件仅限当前系统用户访问，但不具备系统钥匙串的加密保护。</small>
                <button v-if="hasApiKey" class="clear-key" type="button" @click="removeApiKey">移除已保存密钥</button>
              </label>
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

          <section v-else class="settings-section about-settings">
            <div class="about-product">
              <img src="/logo.svg" alt="应用 Logo" />
              <div>
                <h3>拿了桔子跑啊</h3>
                <p>版本 {{ appVersion || '…' }} · Tauri 2 · Rust · Vue 3</p>
              </div>
            </div>
            <div class="update-card" :class="updateState">
              <div>
                <strong>{{ updateInfo ? `可更新至 ${updateInfo.version}` : '应用更新' }}</strong>
                <p>{{ updateMessage || '从 GitHub Releases 获取正式版本。' }}</p>
              </div>
              <button
                v-if="updateState === 'available'"
                type="button"
                @click="installUpdate"
              >下载并安装</button>
              <button
                v-else
                type="button"
                :disabled="updateState === 'checking' || updateState === 'downloading'"
                @click="checkUpdate()"
              >{{ updateState === 'checking' ? '正在检查…' : updateState === 'downloading' ? '正在更新…' : '检查更新' }}</button>
              <div v-if="updateState === 'downloading'" class="update-progress">
                <i :style="{ width: `${updateProgress ?? 18}%` }"></i>
              </div>
              <p v-if="updateInfo?.notes" class="update-notes">{{ updateInfo.notes }}</p>
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
.settings-nav{padding:21px 13px;border-right:1px solid #ded9cf;background:#eeebe4}.settings-brand{display:flex;align-items:center;gap:10px;padding:0 7px 20px}.settings-brand img{width:35px;height:35px;border-radius:10px}.settings-brand div{display:flex;flex-direction:column}.settings-brand strong{font-size:15px}.settings-brand span{margin-top:2px;color:#979187;font-size:10px}.settings-nav nav{display:grid;gap:4px}.settings-nav nav button{display:flex;height:36px;align-items:center;gap:9px;padding:0 10px;border:0;border-radius:8px;color:#69645b;background:transparent;cursor:pointer;text-align:left;font-size:12px}.settings-nav nav button span{display:grid;width:19px;place-items:center;color:#818a82}.settings-nav nav button:hover{background:rgb(255 255 255 / 50%)}.settings-nav nav button.active{color:#344c3b;background:#fffefa;box-shadow:0 2px 8px rgb(52 47 38 / 7%);font-weight:650}
.settings-content{display:flex;min-width:0;min-height:0;flex-direction:column}.settings-content>header{display:flex;min-height:80px;align-items:center;justify-content:space-between;padding:18px 26px;border-bottom:1px solid #e8e4db}.settings-content h2,.settings-content p{margin:0}.settings-content h2{font-size:19px}.settings-content header p{margin-top:4px;color:#8d877d;font-size:11px}.settings-close{display:grid;width:30px;height:30px;place-items:center;border:0;border-radius:8px;color:#8b857b;background:transparent;cursor:pointer;font-size:22px}.settings-close:hover{background:#eeeae2}.settings-scroll{min-height:0;flex:1;overflow-y:auto;padding:24px 27px}.settings-section{display:grid;gap:14px}.setting-row{display:flex;align-items:center;justify-content:space-between;gap:30px;padding:17px 0;border-bottom:1px solid #ece8df}.setting-row.vertical{display:grid;gap:15px}.setting-row label,.field>span{color:#3f3b34;font-size:13px;font-weight:650}.setting-row p{margin-top:5px;color:#969086;font-size:11px}.setting-row select,.field select,.field input:not([type=range]){height:37px;padding:0 10px;border:1px solid #dcd7cd;border-radius:8px;outline:0;color:#49453e;background:#fff;font-size:12px}.setting-row input[type=range],.field input[type=range]{width:100%;accent-color:#4d6654}
.switch-row{display:flex;align-items:center;padding:15px;border:1px solid #dfe4dd;border-radius:11px;background:#f4f7f3;cursor:pointer}.switch-row>span{display:flex;min-width:0;flex:1;flex-direction:column}.switch-row strong{font-size:13px}.switch-row small{margin-top:4px;color:#7f887f;font-size:10px}.switch-row input{position:absolute;opacity:0}.switch-row i{position:relative;width:37px;height:21px;border-radius:14px;background:#c5c7c1;transition:background .15s}.switch-row i:after{position:absolute;top:3px;left:3px;width:15px;height:15px;border-radius:50%;background:#fff;box-shadow:0 1px 4px rgb(0 0 0 / 20%);content:"";transition:transform .15s}.switch-row input:checked+i{background:#52705b}.switch-row input:checked+i:after{transform:translateX(16px)}.field-grid{display:grid;grid-template-columns:1fr 1fr;gap:14px 12px;margin-top:4px}.field-grid.disabled{opacity:.58}.field{display:grid;gap:7px}.field.full{grid-column:1/-1}.field small{color:#989187;font-size:10px;line-height:1.45}.api-key-field>div{display:flex}.api-key-field input{min-width:0;flex:1;border-radius:8px 0 0 8px!important}.api-key-field>div>button{padding:0 11px;border:1px solid #dcd7cd;border-left:0;border-radius:0 8px 8px 0;color:#69645b;background:#f3f0e9;cursor:pointer;font-size:11px}.clear-key{justify-self:start;padding:0;border:0;color:#a34f47;background:transparent;cursor:pointer;font-size:10px}.connection-test{display:flex;align-items:center;gap:11px;margin-top:7px}.connection-test button,.choose-directory{height:35px;padding:0 13px;border:1px solid #cfd8d0;border-radius:8px;color:#3e5b47;background:#eef3ee;cursor:pointer;font-size:11px;font-weight:650}.connection-test button:disabled,.choose-directory:disabled{opacity:.55;cursor:default}.connection-test span{min-width:0;overflow:hidden;color:#7e786f;font-size:10px;text-overflow:ellipsis;white-space:nowrap}.connection-test span.success{color:#3e7650}.connection-test span.error{color:#a34b43}
.storage-card{display:flex;align-items:center;gap:13px;padding:16px;border:1px solid #e2ded5;border-radius:11px;background:#fff}.folder-icon{display:grid;width:40px;height:40px;place-items:center;border-radius:10px;color:#5c735f;background:#e9efe9}.storage-card>div:last-child{display:grid;min-width:0;gap:5px}.storage-card strong,.storage-info strong{font-size:12px}.storage-card code{overflow:hidden;color:#777168;font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:10px;text-overflow:ellipsis;white-space:nowrap}.choose-directory{justify-self:start}.storage-info{margin-top:10px;padding:16px;border-radius:10px;color:#767067;background:#f2efe8}.storage-info p{margin-top:7px;font-size:11px;line-height:1.55}.about-settings{padding:14px 4px}.about-product{display:flex;align-items:center;gap:14px;text-align:left}.about-product img{width:58px;height:58px;border-radius:16px;box-shadow:0 10px 30px rgb(50 66 54 / 16%)}.about-product h3{margin:0;font-size:17px}.about-product p,.privacy-note{margin-top:5px;color:#7f796f;font-size:10px;line-height:1.65}.update-card{position:relative;display:grid;grid-template-columns:1fr auto;gap:5px 16px;margin-top:7px;padding:16px;border:1px solid #e3ded4;border-radius:12px;background:#fff;text-align:left}.update-card>div:first-child{min-width:0}.update-card strong{font-size:12px}.update-card p{margin:5px 0 0;color:#878076;font-size:10px;line-height:1.55}.update-card>button{height:33px;align-self:center;padding:0 12px;border:1px solid var(--accent-border);border-radius:8px;color:var(--accent-strong);background:var(--accent-softest);cursor:pointer;font-size:10px;font-weight:700}.update-card>button:disabled{opacity:.58;cursor:default}.update-card.available{border-color:var(--accent-border);background:#fffaf5}.update-progress{grid-column:1/-1;height:4px!important;margin-top:7px;overflow:hidden;border-radius:99px;background:#eee8df}.update-progress i{display:block;height:100%;border-radius:inherit;background:var(--accent-solid);transition:width .18s}.update-notes{grid-column:1/-1;max-height:64px;overflow:auto;white-space:pre-line}.privacy-note{max-width:500px;margin:3px 0 0!important}
.directory-message{margin:0;color:#54715c!important;font-size:10px!important}
.settings-content>footer{display:flex;min-height:63px;align-items:center;justify-content:space-between;padding:12px 25px;border-top:1px solid #e8e4db}.settings-content footer>span{color:#aaa399;font-size:10px}.settings-content footer div{display:flex;gap:8px}.settings-content footer button{height:34px;padding:0 14px;border-radius:8px;cursor:pointer;font-size:11px;font-weight:650}.settings-content footer .cancel{border:1px solid #ddd8cf;background:#fffefa}.settings-content footer .save{border:1px solid #4d6654;color:#fff;background:#4d6654}.settings-content footer .save:disabled{opacity:.55}
@media(max-width:760px){.settings-dialog{grid-template-columns:145px}.settings-nav{padding:18px 8px}.settings-brand{padding-left:5px}.settings-scroll{padding:20px}.field-grid{grid-template-columns:1fr}.field{grid-column:1/-1}}
.settings-nav nav button.active{color:var(--accent-strong);background:var(--accent-softest)}
.setting-row input[type=range],.field input[type=range]{accent-color:var(--accent)}
.switch-row{border-color:#ead8c8;background:#fffaf4}
.switch-row input:checked+i{background:var(--accent-solid)}
.connection-test button,.choose-directory{border-color:var(--accent-border);color:var(--accent-strong);background:var(--accent-softest)}
.folder-icon{color:var(--accent-strong);background:var(--accent-soft)}
.directory-message{color:var(--accent-strong)!important}
.settings-content footer .save{border-color:var(--accent-solid);background:var(--accent-solid)}
</style>
