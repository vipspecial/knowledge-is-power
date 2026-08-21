import type { AiProtocol } from "./types";

export type AiProviderRegion = "china" | "global" | "custom";

export interface AiProviderPreset {
  id: string;
  name: string;
  region: AiProviderRegion;
  description: string;
  baseUrl: string;
  model: string;
  protocols: readonly AiProtocol[];
}

export const aiProviderPresets: readonly AiProviderPreset[] = [
  {
    id: "deepseek",
    name: "DeepSeek",
    region: "china",
    description: "深度求索官方 API，兼容 Chat Completions。",
    baseUrl: "https://api.deepseek.com/v1",
    model: "deepseek-chat",
    protocols: ["chatCompletions"],
  },
  {
    id: "dashscope",
    name: "阿里云百炼",
    region: "china",
    description: "通义千问 OpenAI 兼容接口。",
    baseUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    model: "qwen-plus",
    protocols: ["chatCompletions"],
  },
  {
    id: "zhipu",
    name: "智谱 AI",
    region: "china",
    description: "智谱开放平台 GLM 模型接口。",
    baseUrl: "https://open.bigmodel.cn/api/paas/v4",
    model: "glm-4.5",
    protocols: ["chatCompletions"],
  },
  {
    id: "moonshot",
    name: "月之暗面 Kimi",
    region: "china",
    description: "Moonshot 官方 OpenAI 兼容接口。",
    baseUrl: "https://api.moonshot.cn/v1",
    model: "moonshot-v1-32k",
    protocols: ["chatCompletions"],
  },
  {
    id: "volcengine",
    name: "火山方舟 / 豆包",
    region: "china",
    description: "火山方舟模型服务，模型名可替换为控制台中的接入点 ID。",
    baseUrl: "https://ark.cn-beijing.volces.com/api/v3",
    model: "doubao-seed-1-6-250615",
    protocols: ["chatCompletions"],
  },
  {
    id: "siliconflow",
    name: "硅基流动",
    region: "china",
    description: "聚合多种开源模型的 OpenAI 兼容服务。",
    baseUrl: "https://api.siliconflow.cn/v1",
    model: "deepseek-ai/DeepSeek-V3.1",
    protocols: ["chatCompletions"],
  },
  {
    id: "openai",
    name: "OpenAI",
    region: "global",
    description: "支持 Responses API 与 Chat Completions。",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-5.6",
    protocols: ["responses", "chatCompletions"],
  },
  {
    id: "anthropic",
    name: "Anthropic Claude",
    region: "global",
    description: "使用 Anthropic 原生 Messages API，支持流式输出。",
    baseUrl: "https://api.anthropic.com/v1",
    model: "claude-sonnet-4-6",
    protocols: ["anthropic"],
  },
  {
    id: "gemini",
    name: "Google Gemini",
    region: "global",
    description: "使用 Gemini 官方 OpenAI 兼容接口。",
    baseUrl: "https://generativelanguage.googleapis.com/v1beta/openai",
    model: "gemini-2.5-pro",
    protocols: ["chatCompletions"],
  },
  {
    id: "xai",
    name: "xAI Grok",
    region: "global",
    description: "xAI 官方 OpenAI 兼容接口。",
    baseUrl: "https://api.x.ai/v1",
    model: "grok-4",
    protocols: ["chatCompletions"],
  },
  {
    id: "mistral",
    name: "Mistral AI",
    region: "global",
    description: "Mistral 官方 Chat Completions 接口。",
    baseUrl: "https://api.mistral.ai/v1",
    model: "mistral-large-latest",
    protocols: ["chatCompletions"],
  },
  {
    id: "openrouter",
    name: "OpenRouter",
    region: "global",
    description: "统一接入多家模型，可继续修改具体模型名称。",
    baseUrl: "https://openrouter.ai/api/v1",
    model: "openrouter/auto",
    protocols: ["chatCompletions"],
  },
  {
    id: "custom",
    name: "自定义 API / 本地模型",
    region: "custom",
    description: "地址、协议和模型均可修改，支持 Ollama 等无 Key 本地服务。",
    baseUrl: "http://localhost:11434/v1",
    model: "qwen3:8b",
    protocols: ["chatCompletions", "responses", "anthropic"],
  },
];

export function findAiProvider(id: string): AiProviderPreset {
  return aiProviderPresets.find((provider) => provider.id === id)
    ?? aiProviderPresets[aiProviderPresets.length - 1];
}

export function inferAiProvider(baseUrl: string, protocol: AiProtocol): string {
  const normalized = baseUrl.toLowerCase();
  const match = aiProviderPresets.find((provider) =>
    provider.id !== "custom"
    && provider.baseUrl.length > 0
    && normalized.includes(new URL(provider.baseUrl).hostname.toLowerCase()),
  );
  if (match) return match.id;
  if (protocol === "anthropic" && normalized.includes("anthropic.com")) return "anthropic";
  return "custom";
}
