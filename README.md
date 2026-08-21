# 拿了桔子跑啊

[![Desktop Build](https://github.com/vipspecial/knowledge-is-power/actions/workflows/build-desktop.yml/badge.svg)](https://github.com/vipspecial/knowledge-is-power/actions/workflows/build-desktop.yml)
[![Release](https://img.shields.io/github/v/release/vipspecial/knowledge-is-power?display_name=tag)](https://github.com/vipspecial/knowledge-is-power/releases)
[![Stars](https://img.shields.io/github/stars/vipspecial/knowledge-is-power?style=flat)](https://github.com/vipspecial/knowledge-is-power/stargazers)
[![Downloads](https://img.shields.io/github/downloads/vipspecial/knowledge-is-power/total)](https://github.com/vipspecial/knowledge-is-power/releases)
[![License](https://img.shields.io/github/license/vipspecial/knowledge-is-power)](./LICENSE)

[下载应用](https://github.com/vipspecial/knowledge-is-power/releases) · [版本标签](https://github.com/vipspecial/knowledge-is-power/tags) · [宣传页](https://vipspecial.github.io/knowledge-is-power/)

一款以 AI 辅助创作为核心、本地优先的知识库应用。支持 macOS、Windows 和 Linux。

名字源自范伟电影台词中对 **Knowledge is power** 的趣味谐音：“拿了桔子跑啊”。我们借用这份幽默，也希望知识不只被收藏，更能随手带走、马上用起来。

## 核心能力

### AI 写作

- 生成文章、标题、大纲和标签
- 续写、改写、校对和总结全文
- 润色、精简、扩写、翻译和解释选区
- 针对当前文档问答并分析内容缺口
- AI 结果统一进入当前文档的右侧会话，流式输出并可直接应用

### 知识管理

- 多知识库与父子文档
- 搜索、标签、置顶和文档副本
- 卡片与紧凑列表模式
- 回收站与文档树恢复

### 所见即所得编辑

- 直接编辑最终排版，不显示 Markdown 源码
- 标题、粗斜体、下划线、删除线与高亮
- 列表、任务、引用、代码、链接和表格
- 兼容富文本粘贴及 Markdown 导入、导出

## 适用场景

| 场景 | 用法 |
| --- | --- |
| 长文创作 | 用 AI 生成大纲、续写、校对和优化表达 |
| 工作记录 | 整理会议纪要、周报、方案和行动项 |
| 学习研究 | 管理读书笔记，并针对当前内容提问 |
| 项目知识库 | 用知识库和父子文档划分不同项目 |

## 快速开始

1. 在左侧创建或选择知识库。
2. 新建文档，像普通文档一样直接编辑排版。
3. 在设置中配置 AI 服务。
4. 从标题、选区、正文或右侧助手调用 AI。

## AI 使用

AI 功能分布在对应的写作位置：

| 目标 | 入口 |
| --- | --- |
| 生成标题或标签 | 标题和标签右侧 |
| 优化一段文字 | 选中文字后出现 AI 浮层，结果进入右侧会话 |
| 续写或校对全文 | 正文格式栏，结果进入右侧会话 |
| 从零起草文章 | AI 写作工作台 |
| 总结或询问文章 | 右侧 AI 助手 |

首次使用时，前往“设置 → AI 助手”填写 API 地址、模型和 API Key。支持 Chat Completions、Responses API 及兼容服务；Ollama 等本地服务可不填写 Key。

## 文档与数据

- 文档以真实 `.md` 文件保存，可自行备份或同步。
- 默认目录为系统“文档/拿了桔子跑啊”，可在设置中修改。
- 删除的文档先进入回收站，可恢复或永久删除。
- AI 默认关闭，仅在主动调用时发送当前文档或选区。
- 不同文档的 AI 上下文相互隔离。
- API Key 保存在应用配置目录，不使用系统钥匙串。

## 界面操作

- 知识库栏、文档栏和 AI 栏均可折叠。
- 文档栏和 AI 栏可调整宽度，正文自动占用剩余空间。
- 右键知识库、文档或回收站条目可快速操作。
- `Cmd/Ctrl + N`：新建文档
- `Cmd/Ctrl + F`：搜索
- `Cmd/Ctrl + S`：保存
- `Cmd/Ctrl + J`：AI 写作
- `Cmd/Ctrl + ,`：设置
- “设置 → 关于”：从 GitHub Releases 检查并安装正式更新

## 本地启动

需要 Node.js 22+、Rust stable，以及对应系统的 [Tauri 依赖](https://v2.tauri.app/start/prerequisites/)。

```bash
npm install
npm run tauri dev
```

只调试宣传页或 Vue 界面时可运行 `npm run dev`。提交前建议执行：

```bash
npm run check:version
npm run build
npm run test:rust
```

## 打包与部署

本地安装包：

```bash
npm run build:mac
npm run build:windows
npm run build:linux
```

在对应系统执行相应命令；普通本地打包不需要更新签名密钥。

- 宣传页：修改 `website/` 后推送到 `main`，GitHub Actions 自动部署 Pages。
- 桌面版本：更新版本号与 `CHANGELOG.md`，推送同名标签，例如 `vX.Y.Z`。
- 桌面安装包：GitHub Actions 自动构建并上传到 Releases，当前暂不启用应用内自动更新。

## Star 趋势

[![Star History Chart](https://api.star-history.com/svg?repos=vipspecial/knowledge-is-power&type=Date)](https://www.star-history.com/#vipspecial/knowledge-is-power&Date)

## 开源协议

本项目采用 [MIT License](./LICENSE)，允许个人和商业项目自由使用、修改与分发，但需保留原版权和许可声明。
