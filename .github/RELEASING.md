# 发布与应用更新

应用通过 GitHub Releases 检查、下载并安装签名更新。以下配置只由仓库发布者执行一次，应用用户和普通开发者都不需要密钥。

## 首次配置

本地 `updater.key` 是更新签名私钥，已被 `.gitignore` 排除，不能提交或丢失。把它的完整内容保存为仓库 Secret：

```bash
gh secret set TAURI_SIGNING_PRIVATE_KEY < updater.key
```

公钥保存在 `.github/updater.pub`，并已写入 `src-tauri/tauri.conf.json`。如果更换密钥，旧版本将无法验证新更新。

## 发布版本

1. 同步修改应用版本并执行 `npm run check:version`。
2. 更新 `CHANGELOG.md`。
3. 推送与版本一致的标签，例如 `vX.Y.Z`。
4. GitHub Actions 自动构建各平台安装包、签名更新包和 `latest.json`。

发布成功后，已安装应用会从仓库的最新 Release 获取更新。
