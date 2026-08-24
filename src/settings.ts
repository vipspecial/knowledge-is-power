import { invoke } from "@tauri-apps/api/core";
import { inferAiProvider } from "./aiProviders";
import { browserStorageKeys, readBrowserStorage, writeBrowserStorage } from "./browserStorage";
import type { AiProtocol, AiSettings, AppSettings, McpSetupInfo, NotesStore, SettingsView } from "./types";

export const defaultSettings: AppSettings = {
  general: {
    autoSaveDelayMs: 450,
  },
  ai: {
    enabled: false,
    provider: "openai",
    baseUrl: "https://api.openai.com/v1",
    protocol: "chatCompletions",
    model: "gpt-5.6",
    models: ["gpt-5.6"],
    temperature: 0.3,
    maxContextChars: 30000,
  },
  documentDirectory: "",
};

export function cloneAppSettings(settings: AppSettings): AppSettings {
  const models = Array.isArray(settings.ai.models) && settings.ai.models.length
    ? settings.ai.models
    : [settings.ai.model];
  return {
    general: { ...settings.general },
    ai: { ...settings.ai, models: [...models] },
    documentDirectory: settings.documentDirectory,
  };
}

function isRunningInTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

function normalizeSettings(settings: Partial<AppSettings>): AppSettings {
  const storedAi = settings.ai as Partial<AiSettings> | undefined;
  const ai = { ...defaultSettings.ai, ...storedAi };
  if (!storedAi?.provider) {
    ai.provider = inferAiProvider(ai.baseUrl, ai.protocol as AiProtocol);
  }
  ai.models = [...new Set([...(storedAi?.models ?? []), ai.model].map((model) => model.trim()).filter(Boolean))];
  return {
    general: { ...defaultSettings.general, ...settings.general },
    ai,
    documentDirectory: settings.documentDirectory ?? "",
  };
}

export async function loadAppSettings(): Promise<SettingsView> {
  if (isRunningInTauri()) return invoke<SettingsView>("load_settings");
  const stored = readBrowserStorage(browserStorageKeys.settings);
  return {
    settings: normalizeSettings(stored ? JSON.parse(stored) : {}),
    hasApiKey: false,
    credentialError: null,
  };
}

export async function saveAppSettings(
  settings: AppSettings,
  apiKey?: string,
): Promise<SettingsView> {
  if (isRunningInTauri()) {
    return invoke<SettingsView>("save_settings", {
      settings,
      apiKey: apiKey?.trim() || null,
    });
  }
  writeBrowserStorage(browserStorageKeys.settings, JSON.stringify(settings));
  return { settings, hasApiKey: false, credentialError: null };
}

export async function clearApiKey(): Promise<void> {
  if (isRunningInTauri()) await invoke("clear_ai_api_key");
}

export async function getMcpSetupInfo(): Promise<McpSetupInfo> {
  if (isRunningInTauri()) return invoke<McpSetupInfo>("get_mcp_setup_info");
  return {
    enabled: readBrowserStorage(browserStorageKeys.mcpEnabled) === "true",
    serviceName: "orange-run-notes",
    executablePath: "",
    accessFilePath: "",
  };
}

export async function setMcpEnabled(enabled: boolean): Promise<McpSetupInfo> {
  if (isRunningInTauri()) return invoke<McpSetupInfo>("set_mcp_enabled", { enabled });
  writeBrowserStorage(browserStorageKeys.mcpEnabled, String(enabled));
  return { enabled, serviceName: "orange-run-notes", executablePath: "", accessFilePath: "" };
}

export async function chooseDocumentDirectory(store: NotesStore): Promise<string | null> {
  if (!isRunningInTauri()) return null;
  return invoke<string | null>("choose_document_directory", { store });
}

export async function testAiConnection(
  settings: AiSettings,
  apiKey?: string,
): Promise<string> {
  if (!isRunningInTauri()) throw new Error("请在桌面应用中测试 AI 连接");
  return invoke<string>("test_ai_connection", {
    settings,
    apiKey: apiKey?.trim() || null,
  });
}

export async function listAiModels(
  settings: AiSettings,
  apiKey?: string,
): Promise<string[]> {
  if (!isRunningInTauri()) throw new Error("请在桌面应用中获取模型列表");
  return invoke<string[]>("list_ai_models", {
    settings,
    apiKey: apiKey?.trim() || null,
  });
}
