#!/usr/bin/env node
// 从桌面构建产物生成 Tauri 自动更新清单 latest.json，供 GitHub Release 托管。
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const artifactsDir = process.argv[2] ?? "release-artifacts";
const version = JSON.parse(readFileSync("package.json", "utf8")).version;
const repo = process.env.GITHUB_REPOSITORY ?? "vipspecial/knowledge-is-power";
const releaseTag = process.env.RELEASE_TAG || `v${version}`;

const files = readdirSync(artifactsDir);

function signatureFor(baseName) {
  const sigName = files.find((name) => name === `${baseName}.sig`);
  if (!sigName) throw new Error(`缺少签名文件：${baseName}.sig`);
  return readFileSync(join(artifactsDir, sigName), "utf8").trim();
}

function platformEntry(fileName) {
  const url = `https://github.com/${repo}/releases/download/${encodeURIComponent(releaseTag)}/${encodeURIComponent(fileName)}`;
  return { signature: signatureFor(fileName), url };
}

const platforms = {};

const appTar = files.find((name) => name.endsWith(".app.tar.gz"));
if (!appTar) throw new Error("缺少 macOS 产物（*.app.tar.gz）");
// Universal 构建同时覆盖 Intel 与 Apple Silicon。
{
  const entry = platformEntry(appTar);
  platforms["darwin-aarch64"] = entry;
  platforms["darwin-x86_64"] = entry;
}

const setupExe = files.find((name) => name.endsWith("-setup.exe"));
if (!setupExe) throw new Error("缺少 Windows 产物（*-setup.exe）");
platforms["windows-x86_64"] = platformEntry(setupExe);

const appImage = files.find((name) => name.endsWith(".AppImage"));
if (!appImage) throw new Error("缺少 Linux 产物（*.AppImage）");
platforms["linux-x86_64"] = platformEntry(appImage);

if (Object.keys(platforms).length === 0) {
  throw new Error("release-artifacts 中未找到任何更新产物（.app.tar.gz / -setup.exe / .AppImage）");
}

const manifest = {
  version,
  notes: "",
  pub_date: new Date().toISOString(),
  platforms,
};

writeFileSync(join(artifactsDir, "latest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`latest.json 已生成（版本 ${version}）：${Object.keys(platforms).join(", ")}`);
