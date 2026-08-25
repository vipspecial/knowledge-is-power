#!/usr/bin/env node
// 发布打标防呆脚本：从 package.json 读取版本生成 vX.Y.Z 标签并推送，
// 保证标签名与版本一致、标签落在已推送的 HEAD 上，避免 CI 校验失败。
import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { readFileSync } from "node:fs";

function git(...args) {
  return execFileSync("git", args, { encoding: "utf8" }).trim();
}

function fail(message) {
  console.error(`✗ ${message}`);
  process.exit(1);
}

const version = JSON.parse(readFileSync("package.json", "utf8")).version;
const tag = `v${version}`;

// 1. 工作区必须干净，避免把未提交内容与发布标签混淆。
const status = git("status", "--porcelain");
if (status) fail("工作区有未提交改动，请先提交或暂存后再打发布标签");

// 2. HEAD 必须已推送到远端，否则 CI 构建的提交与本地不一致。
const head = git("rev-parse", "HEAD");
const remoteHead = git("rev-parse", "origin/main");
if (head !== remoteHead) fail("HEAD 未与 origin/main 同步，请先 push 再打发布标签");

// 3. 标签唯一：本地与远端都不能已存在。
if (existsSync(`.git/refs/tags/${tag}`)) fail(`本地标签 ${tag} 已存在`);
const remoteTags = git("ls-remote", "--tags", "origin", tag);
if (remoteTags) fail(`远端标签 ${tag} 已存在；如属误打，请先删除再重试`);

git("tag", tag);
try {
  git("push", "origin", tag);
} catch (error) {
  git("tag", "-d", tag);
  fail(`推送标签失败，已回滚本地标签：${error.message}`);
}

console.log(`✓ 已创建并推送发布标签 ${tag}（${head.slice(0, 7)}），CI 将开始构建`);
