# 版本、验证与发布

## 每次修改都递增版本

普通修复、文档或维护变更递增补丁版本；新增完整能力或不兼容调整按语义化版本选择更高层级。必须同步以下位置：

1. `package.json`
2. `package-lock.json` 的根版本和根 package 版本
3. `src-tauri/Cargo.toml`
4. `src-tauri/Cargo.lock` 中 `orange-run-notes` 包版本
5. `src-tauri/tauri.conf.json`
6. `CHANGELOG.md` 顶部新增同版本中文记录和当天日期

运行 `npm run check:version` 确认一致。不要只修改安装包版本或只写更新记录。

## 验证

默认检查：

```bash
npm run check:version
npm run build
npm run test:rust
git diff --check
```

- 纯文档或 Skill 修改也要运行版本一致性与 `git diff --check`；条件允许时仍执行完整检查。
- 编辑器或界面修改需在 Tauri 窗口验证长文滚动、任务列表/表格、选区浮层、折叠恢复、拖动宽度、窄窗口和 macOS 标题栏。
- AI 修改需测试流式与非流式响应、无 Key 本地服务、错误响应和文档隔离，并补充 Rust 单元测试。
- 存储修改需测试 Markdown 往返、旧数据读取、目录切换、路径安全、回收站和失败恢复。
- 宣传页修改需检查 GitHub Pages 子路径下的资源与链接，避免使用会指向域名根目录的错误绝对路径。
- 运行时或 UI 改动在 macOS 上完成后执行 `npm run build:mac`，向用户提供实际 DMG 路径。纯文档、Skill 或工作流修改无需重复生成安装包，除非用户明确要求。

构建出现 Vite 大包警告时可以如实报告，但不要把警告当成失败；只有在任务涉及性能或包体时才扩展范围处理。

## Git 与提交

- 每个完成的修改创建一次中文 Conventional Commit，例如 `feat: 增加……`、`fix: 修复……`、`docs: 优化……`、`ci: 调整……`。
- 提交前再次检查 `git status` 和差异，只暂存本次文件；不夹带用户无关改动或本地密钥。
- 未经明确要求不要 amend 旧提交、强制推送、创建标签、推送标签或发布 Release。
- 标签不会自动生成。发布时标签必须与版本完全一致，格式为 `vX.Y.Z`。

## 当前 CI 与发布状态

- `.github/workflows/build-desktop.yml` 在推送 `v*` 标签时检查并构建：macOS Universal DMG、Windows x64 NSIS EXE、Linux x64 DEB 与 AppImage，然后发布 GitHub Release。
- 手动触发桌面工作流只构建附件，不会在无标签时发布 Release。
- `.github/workflows/deploy-pages.yml` 在 `website/**` 或工作流自身变更推送到 `main` 后部署宣传页。
- 应用内自动更新已启用：`createUpdaterArtifacts` 为 `true`，CI 从 GitHub Secrets 读取 `TAURI_SIGNING_PRIVATE_KEY` 与 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` 生成各平台 `.sig` 签名，Release 任务用 `scripts/generate-update-manifest.mjs` 汇总生成 `latest.json` 并随安装包一起发布。
- 更新公钥内嵌在 `src-tauri/tauri.conf.json` 的 `plugins.updater.pubkey`；更换密钥对、更新端点或调整签名方式必须作为独立任务处理并同步端到端验证升级链路，不能只为消除一次 CI 错误而临时改动。
- Secrets 缺失时打标签构建仍会执行，但产物无签名，`latest.json` 会因缺 `.sig` 而不完整；发布前确认 Secrets 已配置。
