import { getVersion } from "@tauri-apps/api/app";
import {
  check,
  type DownloadEvent,
  type Update,
} from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import packageInfo from "../package.json";

const latestReleaseApi =
  "https://api.github.com/repos/vipspecial/knowledge-is-power/releases/latest";

export interface AppUpdateInfo {
  currentVersion: string;
  version: string;
  date?: string;
  notes?: string;
}

let pendingUpdate: Update | null = null;
let checkInFlight: Promise<AppUpdateInfo | null> | null = null;

interface GitHubRelease {
  tag_name?: string;
  html_url?: string;
}

export function updaterAvailable(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

export async function getCurrentAppVersion(): Promise<string> {
  return updaterAvailable() ? getVersion() : packageInfo.version;
}

function versionParts(version: string): number[] {
  return version
    .trim()
    .replace(/^v/i, "")
    .split(".")
    .map((part) => Number.parseInt(part, 10) || 0);
}

function compareVersions(left: string, right: string): number {
  const leftParts = versionParts(left);
  const rightParts = versionParts(right);
  const length = Math.max(leftParts.length, rightParts.length);
  for (let index = 0; index < length; index += 1) {
    const difference = (leftParts[index] ?? 0) - (rightParts[index] ?? 0);
    if (difference !== 0) return difference;
  }
  return 0;
}

/**
 * Older releases may not contain Tauri's latest.json. In that case, use the
 * public GitHub API to distinguish "no newer release" from a broken package.
 */
async function checkReleaseFallback(): Promise<AppUpdateInfo | null> {
  let response: Response;
  try {
    response = await fetch(latestReleaseApi, {
      headers: { Accept: "application/vnd.github+json" },
    });
  } catch {
    throw new Error("无法连接 GitHub Releases，请检查网络后重试");
  }
  if (response.status === 404) return null;
  if (!response.ok) {
    throw new Error(`GitHub Releases 暂时不可用（${response.status}）`);
  }

  const release = (await response.json()) as GitHubRelease;
  const latestVersion = release.tag_name?.replace(/^v/i, "").trim();
  if (!latestVersion) {
    throw new Error("GitHub Release 缺少有效版本号");
  }
  const currentVersion = await getCurrentAppVersion();
  if (compareVersions(latestVersion, currentVersion) <= 0) return null;
  throw new Error(
    `发现 ${latestVersion}，但该 Release 缺少自动更新文件，请前往 GitHub Releases 手动下载`,
  );
}

/** Keep one checked update alive so the About page can install it later. */
export async function checkForAppUpdate(): Promise<AppUpdateInfo | null> {
  if (!updaterAvailable()) return null;
  if (checkInFlight) return checkInFlight;

  checkInFlight = (async () => {
    let update: Update | null;
    try {
      update = await check({ timeout: 15_000 });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      if (/valid release JSON|latest\.json|status code 404/i.test(message)) {
        return checkReleaseFallback();
      }
      throw error;
    }
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
