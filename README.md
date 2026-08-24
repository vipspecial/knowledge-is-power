<p align="center">
  <img src="./public/logo.svg" width="112" alt="拿了桔子跑啊 Logo">
</p>

<h1 align="center">拿了桔子跑啊</h1>

<p align="center"><strong>Knowledge is power</strong></p>
<p align="center">AI 驱动、本地优先的知识创作桌面应用</p>

<p align="center">
  <a href="https://github.com/vipspecial/knowledge-is-power/actions/workflows/build-desktop.yml"><img src="https://github.com/vipspecial/knowledge-is-power/actions/workflows/build-desktop.yml/badge.svg" alt="Desktop Build"></a>
  <a href="https://github.com/vipspecial/knowledge-is-power/releases"><img src="https://img.shields.io/github/v/release/vipspecial/knowledge-is-power?display_name=tag" alt="Release"></a>
  <a href="https://github.com/vipspecial/knowledge-is-power/stargazers"><img src="https://img.shields.io/github/stars/vipspecial/knowledge-is-power?style=flat" alt="Stars"></a>
  <a href="https://github.com/vipspecial/knowledge-is-power/releases"><img src="https://img.shields.io/github/downloads/vipspecial/knowledge-is-power/total" alt="Downloads"></a>
  <a href="./LICENSE"><img src="https://img.shields.io/github/license/vipspecial/knowledge-is-power" alt="License"></a>
</p>

<p align="center">
  <a href="https://github.com/vipspecial/knowledge-is-power/releases">下载应用</a> ·
  <a href="https://vipspecial.github.io/knowledge-is-power/">宣传页</a> ·
  <a href="https://github.com/vipspecial/knowledge-is-power/issues">问题反馈</a>
</p>

名字源自电影《爱情呼叫转移 2》中范伟对 **Knowledge is power（知识就是力量）** 的趣味谐音：“拿了桔子跑啊”。知识不只被收藏，更要随手带走、马上用起来。

适合长文创作、工作记录、学习研究与项目知识沉淀。

## 下载

前往 [GitHub Releases](https://github.com/vipspecial/knowledge-is-power/releases) 获取最新版：

| 系统 | 安装包 | 支持架构 |
| --- | --- | --- |
| macOS | `.dmg` | Apple 芯片与 Intel（Universal） |
| Windows | `.exe` | x64 |
| Linux | `.deb` / `.AppImage` | x64 |

应用可在“设置 → 关于”中检查、下载并安装 GitHub Releases 的签名更新。

## 核心能力

### AI 融入写作过程

- 从零生成文章、标题、大纲与标签
- 对全文续写、改写、总结、校对和查漏补缺
- 选中文字即可润色、精简、扩写、翻译或解释
- AI 结果流式进入当前文档会话，可直接替换、追加或复制
- 每篇文档拥有独立上下文，避免不同文章相互干扰

### 所见即所得编辑

- 直接编辑最终排版，无需面对 Markdown 源码
- 支持标题、列表、任务、引用、代码、链接、表格与高亮
- 兼容富文本粘贴及 Markdown 文件导入、导出

### 本地知识管理

- 多知识库、父子文档、标签与跨知识库全文搜索
- 卡片和紧凑列表两种文档视图
- 文档保存为本地 `.md` 文件，目录可自行指定
- 回收站支持恢复或永久删除

### 连接其他 AI 工具

应用内置只读 MCP，可让 Claude Desktop、Cursor、Codex 等工具检索本地知识库。它只开放列出知识库、搜索文档和读取指定文档，不读取回收站，也不允许外部工具修改文件。

配置与安装方法见 [MCP 使用说明](./docs/MCP.md)。

## 开始使用

1. 创建知识库，并在设置中选择文档保存目录。
2. 新建文档，像普通文档一样直接编辑内容与排版。
3. 在“设置 → AI 助手”配置模型服务。
4. 从标题、选区、正文工具栏或右侧助手调用 AI。

AI 配置内置国内、国外主流服务预设，也支持 OpenAI-compatible、Responses、Anthropic Messages 及自定义 API。可从服务端获取模型列表或手动配置多个模型，并在每篇文档的 AI 对话区独立切换。使用 Ollama 等本地服务时可以不填写 API Key。

## 数据与隐私

- 文档保存在用户指定目录，可自行备份或同步。
- AI 默认关闭，仅在用户主动调用时发送当前文档或选区。
- 不同文档的 AI 会话和上下文相互隔离。
- API Key 保存在应用配置目录，不写入项目代码或文档目录。

## 常用操作

| 快捷键 | 功能 |
| --- | --- |
| `Cmd/Ctrl + N` | 新建文档 |
| `Cmd/Ctrl + K` | 全局搜索 |
| `Cmd/Ctrl + F` | 搜索当前知识库 |
| `Cmd/Ctrl + S` | 保存 |
| `Cmd/Ctrl + J` | AI 写作 |
| `Cmd/Ctrl + ,` | 设置 |

知识库栏、文档栏和 AI 栏均可折叠；文档栏与 AI 栏支持拖动调整宽度。

## 本地运行

需要 Node.js 22+、Rust stable，以及当前系统的 [Tauri 依赖](https://v2.tauri.app/start/prerequisites/)。

```bash
npm install
npm run tauri dev
```

仅调试 Vue 界面或宣传页时运行：

```bash
npm run dev
```

提交前检查：

```bash
npm run check:version
npm run build
npm run test:rust
```

## 打包与发布

在对应操作系统上执行本地打包：

```bash
npm run build:mac
npm run build:windows
npm run build:linux
```

- 宣传页：修改 `website/` 并推送到 `main`，GitHub Actions 自动部署至 Pages。
- 安装包：同步版本号与 `CHANGELOG.md`，推送 `vX.Y.Z` 标签后自动构建并发布至 Releases。
- 本地普通安装包使用 `tauri.local.conf.json`，不需要自动更新签名密钥。
- GitHub Release 需要配置 `TAURI_SIGNING_PRIVATE_KEY` 和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`；私钥可填写 `tauri signer generate` 生成的 Base64、完整的 minisign 私钥文本，或带变量名的输出行，工作流会安全规范化格式且不会输出内容。

## Star 趋势

[![Star History Chart](https://api.star-history.com/svg?repos=vipspecial/knowledge-is-power&type=Date)](https://www.star-history.com/#vipspecial/knowledge-is-power&Date)

## 开源协议

本项目采用 [MIT License](./LICENSE)，允许个人和商业项目自由使用、修改与分发，但需保留原版权和许可声明。
