import { Channel, invoke } from "@tauri-apps/api/core";
import type { AiOperation, AiRequest, AiStreamEvent, Note } from "./types";

export function createDocumentAiRequest(
  note: Note,
  operation: AiOperation,
  prompt = "",
  selection = "",
  model = "",
): AiRequest {
  if (!note.id) throw new Error("当前文档缺少标识，无法安全调用 AI");
  return {
    documentId: note.id,
    model,
    operation,
    prompt,
    selection,
    noteTitle: note.title,
    noteContent: note.content,
  };
}

export async function streamAi(
  request: AiRequest,
  onEvent: (event: AiStreamEvent) => void,
): Promise<void> {
  if (!("__TAURI_INTERNALS__" in window)) {
    throw new Error("请在桌面应用中使用 AI 功能");
  }
  const channel = new Channel<AiStreamEvent>();
  channel.onmessage = onEvent;
  await invoke("stream_ai", { request, onEvent: channel });
}
