/**
 * 流式输出节奏器：部分 AI 服务不支持流式或经代理缓冲，内容会一次性到达。
 * 该队列把一次性文本按目标时长平滑放出，保证界面始终逐步显示。
 */
const TICK_MS = 16;
const TARGET_TICKS = 150;

export interface StreamPacer {
  push(text: string): void;
  flush(): Promise<void>;
  reset(): void;
}

export function createStreamPacer(emit: (chunk: string) => void): StreamPacer {
  let queue: string[] = [];
  let timer: number | undefined;
  let waiters: Array<() => void> = [];

  function settle(): void {
    timer = undefined;
    const pending = waiters;
    waiters = [];
    for (const resolve of pending) resolve();
  }

  function pump(): void {
    if (queue.length === 0) {
      settle();
      return;
    }
    const count = Math.min(
      queue.length,
      Math.max(1, Math.ceil(queue.length / TARGET_TICKS)),
    );
    emit(queue.splice(0, count).join(""));
    timer = window.setTimeout(pump, TICK_MS);
  }

  return {
    push(text: string): void {
      // 按码点入队，避免代理对字符被拆开渲染。
      queue.push(...Array.from(text));
      if (timer === undefined) timer = window.setTimeout(pump, 0);
    },
    flush(): Promise<void> {
      if (queue.length === 0 && timer === undefined) return Promise.resolve();
      return new Promise((resolve) => waiters.push(resolve));
    },
    reset(): void {
      if (timer !== undefined) window.clearTimeout(timer);
      queue = [];
      settle();
    },
  };
}
