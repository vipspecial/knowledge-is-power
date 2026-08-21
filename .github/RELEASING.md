# 发布与应用更新

应用通过 GitHub Releases 检查、下载并安装签名更新。以下配置只由仓库发布者执行一次，应用用户和普通开发者都不需要密钥。

## 首次配置

本地 `updater.key` 是更新签名私钥，已被 `.gitignore` 排除，不能提交或丢失。当前密钥使用空密码，由 GitHub Secret 保护。把文件的完整原始内容直接保存为仓库 Secret，不要手动 Base64 解码、不要添加引号：

```bash
gh secret set TAURI_SIGNING_PRIVATE_KEY < updater.key
```

如果通过网页配置，进入 `Settings → Secrets and variables → Actions`：

- 删除旧的 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。
- 重新创建或更新 `TAURI_SIGNING_PRIVATE_KEY`，值为本地 `updater.key` 的完整内容。

工作流会显式使用空密码，并在耗时的多平台编译前验证密钥；配置错误会立即失败。

公钥保存在 `.github/updater.pub`，并已写入 `src-tauri/tauri.conf.json`。如果更换密钥，旧版本将无法验证新更新。

## 发布版本

1. 同步修改应用版本并执行 `npm run check:version`。
2. 更新 `CHANGELOG.md`。
3. 推送与版本一致的标签，例如 `vX.Y.Z`。
4. GitHub Actions 自动构建各平台安装包、签名更新包和 `latest.json`。

发布成功后，已安装应用会从仓库的最新 Release 获取更新。

当前构建架构：macOS 使用 Universal 包，同时支持 Apple 芯片与 Intel；Windows 和 Linux 提供主流的 x64 版本。x86 指 32 位旧系统，不是 x64 的另一种必需安装包。

## 排查更新检查失败

打开最新 Release 的附件列表，确认同时存在 `latest.json`、各平台更新压缩包及对应 `.sig` 文件。若最新 Release 是旧流程生成且没有 `latest.json`，发布一个与当前代码版本一致的新标签即可；不要只上传 DMG 或 EXE 安装包。
