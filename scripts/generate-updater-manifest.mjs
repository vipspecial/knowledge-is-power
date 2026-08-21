import { existsSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";

const artifactDirectory = resolve(process.argv[2] ?? "release-artifacts");
const packageInfo = JSON.parse(readFileSync("package.json", "utf8"));
const repository = process.env.GITHUB_REPOSITORY ?? "vipspecial/knowledge-is-power";
const tag = process.env.GITHUB_REF_NAME ?? `v${packageInfo.version}`;

function walk(directory) {
  return readdirSync(directory).flatMap((entry) => {
    const path = join(directory, entry);
    return statSync(path).isDirectory() ? walk(path) : [path];
  });
}

function releaseAssetUrl(path) {
  return `https://github.com/${repository}/releases/download/${tag}/${encodeURIComponent(basename(path))}`;
}

function signedPlatform(files, suffix) {
  const archive = files.find((path) => path.endsWith(suffix) && !path.endsWith(".sig"));
  if (!archive || !existsSync(`${archive}.sig`)) {
    throw new Error(`缺少更新包或签名：*${suffix}`);
  }
  return {
    signature: readFileSync(`${archive}.sig`, "utf8").trim(),
    url: releaseAssetUrl(archive),
  };
}

const files = walk(artifactDirectory);
const macOS = signedPlatform(files, ".app.tar.gz");
const windows = signedPlatform(files, ".nsis.zip");
const linux = signedPlatform(files, ".AppImage.tar.gz");
const manifest = {
  version: packageInfo.version,
  notes: `查看 ${tag} 的 GitHub Release 获取完整更新说明。`,
  pub_date: new Date().toISOString(),
  platforms: {
    "darwin-aarch64": macOS,
    "darwin-x86_64": macOS,
    "windows-x86_64": windows,
    "linux-x86_64": linux,
  },
};

const output = join(artifactDirectory, "latest.json");
writeFileSync(output, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`已生成 ${output}`);
