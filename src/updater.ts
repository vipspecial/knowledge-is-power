import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

export type { Update };

export type UpdaterState =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "installing"
  | "restart"
  | "upToDate"
  | "error";

export interface UpdaterStatus {
  state: UpdaterState;
  message: string;
  version: string;
  notes: string;
  progress: number;
}

export const initialUpdaterStatus: UpdaterStatus = {
  state: "idle",
  message: "",
  version: "",
  notes: "",
  progress: 0,
};

function isDesktop(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

/** 检查更新；返回 null 表示当前已是最新版本。 */
export async function checkForUpdates(): Promise<Update | null> {
  if (!isDesktop()) throw new Error("自动更新仅在桌面应用内可用");
  return check();
}

/** 下载并安装更新，进度回调返回 0-100。完成后需要 relaunch 才会生效。 */
export async function downloadAndInstallUpdate(
  update: Update,
  onProgress?: (percent: number) => void,
): Promise<void> {
  let total = 0;
  let downloaded = 0;
  await update.downloadAndInstall((event) => {
    switch (event.event) {
      case "Started":
        total = event.data.contentLength ?? 0;
        break;
      case "Progress":
        downloaded += event.data.chunkLength;
        if (onProgress && total > 0) {
          onProgress(Math.min(100, Math.round((downloaded / total) * 100)));
        }
        break;
      case "Finished":
        if (onProgress) onProgress(100);
        break;
    }
  });
}

/** 安装完成后重启应用以应用新版本。 */
export async function relaunchApp(): Promise<void> {
  await relaunch();
}
