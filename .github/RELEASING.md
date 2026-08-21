# 发布说明

应用内自动更新暂时停用。GitHub Actions 不读取签名私钥，只构建普通安装包并上传到 Releases。

## 发布版本

1. 同步修改应用版本并执行 `npm run check:version`。
2. 更新 `CHANGELOG.md`。
3. 推送与版本一致的标签，例如 `vX.Y.Z`。
4. GitHub Actions 自动构建并发布安装包。

当前构建架构：macOS 使用 Universal 包，同时支持 Apple 芯片与 Intel；Windows 和 Linux 提供主流的 x64 版本。

本地 `updater.key` 与 `.github/updater.pub` 保留给以后重新启用签名更新使用，私钥仍禁止提交。
