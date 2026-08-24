# MCP 使用说明

“拿了桔子跑啊”内置只读 stdio MCP，让其他 AI 工具安全检索你明确指定的知识库目录。

## 可用工具

| 工具 | 作用 |
| --- | --- |
| `list_knowledge_bases` | 列出知识库及文档数 |
| `search_documents` | 搜索标题、正文和标签 |
| `read_document` | 按文档 ID 读取一篇文档 |

MCP 不返回回收站文档，也不能创建、修改或删除文件。它不会读取 AI 设置和 API Key。

## 准备路径

先在应用的“设置 → 文档存储”确认知识库目录，再找到应用可执行文件：

- macOS 安装版：`/Applications/拿了桔子跑啊.app/Contents/MacOS/orange-run-notes`
- Windows / Linux：使用安装目录中的 `orange-run-notes` 可执行文件
- 源码构建：运行 `cargo build --release --manifest-path src-tauri/Cargo.toml`，可执行文件位于 `src-tauri/target/release/`

以下示例中的两个路径都要替换为本机绝对路径，不要填写 API Key。

## Claude Desktop / Cursor

将下面配置加入 Claude Desktop 的 `claude_desktop_config.json`，或 Cursor 项目的 `.cursor/mcp.json`：

```json
{
  "mcpServers": {
    "orange-run-knowledge": {
      "command": "/absolute/path/to/orange-run-notes",
      "args": [
        "--mcp",
        "--directory",
        "/absolute/path/to/your/knowledge-library"
      ]
    }
  }
}
```

## Codex

在 Codex 的 `config.toml` 中加入：

```toml
[mcp_servers.orange_run_knowledge]
command = "/absolute/path/to/orange-run-notes"
args = ["--mcp", "--directory", "/absolute/path/to/your/knowledge-library"]
```

保存配置并重启对应 AI 工具。连接成功后，应能看到三个只读工具。

## 常见问题

- 提示“指定目录不是拿了桔子跑啊知识库”：先在应用中选定该目录并保存一次文档。
- 搜索不到文档：回收站内容会被主动排除；可先用 `list_knowledge_bases` 确认知识库 ID。
- 移动知识库目录后：同步更新 MCP 配置中的 `--directory` 路径。
