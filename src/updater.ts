import { getVersion } from "@tauri-apps/api/app";
import {
  check,
  type DownloadEvent,
  type Update,
} from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import packageInfo from "../package.json";

export interface AppUpdateInfo {
  currentVersion: string;
  version: string;
  date?: string;
  notes?: string;
}

let pendingUpdate: Update | null = null;
let checkInFlight: Promise<AppUpdateInfo | null> | null = null;

export function updaterAvailable(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

export async function getCurrentAppVersion(): Promise<string> {
  return updaterAvailable() ? getVersion() : packageInfo.version;
}

/** Keep one checked update alive so the About page can install it later. */
export async function checkForAppUpdate(): Promise<AppUpdateInfo | null> {
  if (!updaterAvailable()) return null;
  if (checkInFlight) return checkInFlight;

  checkInFlight = (async () => {
    const update = await check({ timeout: 15_000 });
    if (pendingUpdate && pendingUpdate !== update) await pendingUpdate.close();
    pendingUpdate = update;
    if (!update) return null;
    return {
      currentVersion: update.currentVersion,
      version: update.version,
      date: update.date,
      notes: update.body,
    };
  })();

  try {
    return await checkInFlight;
  } finally {
    checkInFlight = null;
  }
}

export async function installCheckedUpdate(
  onProgress: (percent: number | null) => void,
): Promise<void> {
  if (!pendingUpdate) throw new Error("请先检查更新");

  let downloaded = 0;
  let total: number | undefined;
  const handleProgress = (event: DownloadEvent): void => {
    if (event.event === "Started") {
      total = event.data.contentLength;
      downloaded = 0;
      onProgress(total ? 0 : null);
    } else if (event.event === "Progress") {
      downloaded += event.data.chunkLength;
      onProgress(total ? Math.min(99, Math.round((downloaded / total) * 100)) : null);
    } else {
      onProgress(100);
    }
  };

  await pendingUpdate.downloadAndInstall(handleProgress, { timeout: 120_000 });
  await relaunch();
}
