import { readFileSync } from "node:fs";

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function requireMatch(value, pattern, label) {
  const match = value.match(pattern);
  if (!match) throw new Error(`无法读取 ${label} 版本`);
  return match[1];
}

const packageJson = readJson("package.json");
const packageLock = readJson("package-lock.json");
const tauriConfig = readJson("src-tauri/tauri.conf.json");
const cargoToml = readFileSync("src-tauri/Cargo.toml", "utf8");
const cargoLock = readFileSync("src-tauri/Cargo.lock", "utf8");

const versions = {
  package: packageJson.version,
  packageLock: packageLock.version,
  packageLockRoot: packageLock.packages?.[""]?.version,
  tauri: tauriConfig.version,
  cargo: requireMatch(cargoToml, /\[package\][\s\S]*?^version = "([^"]+)"/m, "Cargo"),
  cargoLock: requireMatch(
    cargoLock,
    /\[\[package\]\]\s+name = "orange-run-notes"\s+version = "([^"]+)"/m,
    "Cargo.lock",
  ),
};

const expected = versions.package;
const mismatches = Object.entries(versions).filter(([, version]) => version !== expected);
if (mismatches.length > 0) {
  throw new Error(`版本不一致：${JSON.stringify(versions)}`);
}

if (process.env.GITHUB_REF_TYPE === "tag") {
  const expectedTag = `v${expected}`;
  if (process.env.GITHUB_REF_NAME !== expectedTag) {
    throw new Error(`发布标签应为 ${expectedTag}，当前为 ${process.env.GITHUB_REF_NAME}`);
  }
}

console.log(`版本 ${expected} 已同步`);
