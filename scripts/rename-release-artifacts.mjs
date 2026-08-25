#!/usr/bin/env node
// 发布产物文件名含中文品牌名时存在两个风险：
// 1. GitHub Release 上传资产会剥离非 ASCII 字符，导致 latest.json 下载链接 404；
// 2. download-artifact 解压含中文文件名的 zip 时文件名可能被破坏，导致资产丢失。
// 因此在构建机本地（上传 artifact 之前）递归重命名为 ASCII 前缀。
import { readdirSync, renameSync, statSync } from "node:fs";
import { join } from "node:path";

const asciiPrefix = "orange-run-notes";
const isAscii = (name) => /^[\x20-\x7E]+$/.test(name);

const renamed = [];

function normalizeDir(dir) {
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) {
      normalizeDir(path);
      continue;
    }
    if (isAscii(name)) continue;
    const newName = name.replace(/[^\x20-\x7E]+/g, asciiPrefix);
    renameSync(path, join(dir, newName));
    renamed.push(`${name} -> ${newName}`);
  }
}

for (const target of process.argv.slice(2)) {
  normalizeDir(target);
}

if (renamed.length === 0) {
  console.log("产物文件名均为 ASCII，无需重命名");
} else {
  console.log(`已重命名 ${renamed.length} 个产物：`);
  for (const line of renamed) console.log(`  ${line}`);
}
