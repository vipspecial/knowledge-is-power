#!/usr/bin/env node
// 兼容 GitHub Secret 中的原始 minisign 私钥文本与缺少补位的 Base64，且不输出密钥内容。
import { randomUUID } from "node:crypto";
import { appendFileSync } from "node:fs";

function decodedPrivateKey(encoded) {
  const compact = encoded.replace(/\s+/g, "").replace(/-/g, "+").replace(/_/g, "/");
  if (!compact || !/^[A-Za-z0-9+/]*={0,2}$/.test(compact)) {
    throw new Error("TAURI_SIGNING_PRIVATE_KEY 不是有效的私钥文本或 Base64");
  }
  const withoutPadding = compact.replace(/=+$/, "");
  const padded = `${withoutPadding}${"=".repeat((4 - (withoutPadding.length % 4)) % 4)}`;
  const buffer = Buffer.from(padded, "base64");
  if (buffer.toString("base64").replace(/=+$/, "") !== withoutPadding) {
    throw new Error("TAURI_SIGNING_PRIVATE_KEY 的 Base64 内容无效");
  }
  return { decoded: buffer.toString("utf8"), encoded: padded };
}

function normalizePrivateKey(input) {
  let expanded = input.trim().replace(/\\r\\n/g, "\n").replace(/\\n/g, "\n");
  const assignment = expanded.match(/^TAURI_SIGNING_PRIVATE_KEY\s*=\s*([\s\S]+)$/);
  if (assignment) expanded = assignment[1].trim();
  if (
    (expanded.startsWith('"') && expanded.endsWith('"'))
    || (expanded.startsWith("'") && expanded.endsWith("'"))
  ) {
    expanded = expanded.slice(1, -1).trim();
  }
  const normalized = expanded.startsWith("untrusted comment:")
    ? { decoded: expanded, encoded: Buffer.from(expanded, "utf8").toString("base64") }
    : decodedPrivateKey(expanded);
  const lines = normalized.decoded.split(/\r?\n/).filter(Boolean);
  if (!lines[0]?.startsWith("untrusted comment:") || !/secret key/i.test(lines[0]) || lines.length < 2) {
    throw new Error("TAURI_SIGNING_PRIVATE_KEY 解码后不是完整的 minisign 私钥");
  }
  return normalized.encoded;
}

const input = process.env.TAURI_SIGNING_PRIVATE_KEY_INPUT ?? "";
const githubEnv = process.env.GITHUB_ENV;
if (!input.trim()) throw new Error("缺少 GitHub Secret：TAURI_SIGNING_PRIVATE_KEY");
if (!githubEnv) throw new Error("仅可在 GitHub Actions 中准备签名密钥");

const normalizedKey = normalizePrivateKey(input);
const delimiter = `TAURI_SIGNING_KEY_${randomUUID()}`;
appendFileSync(
  githubEnv,
  `TAURI_SIGNING_PRIVATE_KEY<<${delimiter}\n${normalizedKey}\n${delimiter}\n`,
  { encoding: "utf8", mode: 0o600 },
);
console.log("Tauri 更新签名私钥格式检查通过");
