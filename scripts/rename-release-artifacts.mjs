#!/usr/bin/env node
// 发布产物文件名含中文品牌名时，GitHub 上传资产会剥离非 ASCII 字符，
// 导致 latest.json 中的下载链接 404。上传前统一替换为 ASCII 前缀。
import { readdirSync, renameSync } from "node:fs";
import { join } from "node:path";

const artifactsDir = process.argv[2] ?? "release-artifacts";
const asciiPrefix = "orange-run-notes";

const renamed = [];
for (const name of readdirSync(artifactsDir)) {
  if (/^[\x20-\x7E]+$/.test(name)) continue;
  const newName = name.replace(/[^\x20-\x7E]+/g, asciiPrefix);
  renameSync(join(artifactsDir, name), join(artifactsDir, newName));
  renamed.push(`${name} -> ${newName}`);
}

if (renamed.length === 0) {
  console.log("产物文件名均为 ASCII，无需重命名");
} else {
  console.log(`已重命名 ${renamed.length} 个产物：`);
  for (const line of renamed) console.log(`  ${line}`);
}
