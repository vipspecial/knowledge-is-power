use crate::{
    library::load_store_from_directory,
    models::{Note, NotesStore},
};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

const MCP_PROTOCOL_VERSION: &str = "2025-06-18";
const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_QUERY_CHARS: usize = 200;
const MAX_SEARCH_RESULTS: usize = 50;
const MAX_DOCUMENT_CHARS: usize = 100_000;

pub(crate) struct McpServer {
    root: PathBuf,
}

impl McpServer {
    fn new(root: impl AsRef<Path>) -> Result<Self, String> {
        let root = root.as_ref();
        load_store_from_directory(root)?;
        let root = root
            .canonicalize()
            .map_err(|error| format!("无法确认知识库目录：{error}"))?;
        Ok(Self { root })
    }

    fn handle_line(&self, line: &str) -> Option<String> {
        let request = match serde_json::from_str::<Value>(line) {
            Ok(value) => value,
            Err(error) => {
                return Some(json_rpc_error(Value::Null, -32700, &format!("JSON 解析失败：{error}")))
            }
        };
        self.handle_request(&request).map(|response| response.to_string())
    }

    fn handle_request(&self, request: &Value) -> Option<Value> {
        let id = request.get("id").cloned();
        let Some(method) = request.get("method").and_then(Value::as_str) else {
            return id.map(|id| json_rpc_error_value(id, -32600, "无效的 JSON-RPC 请求"));
        };
        let params = request.get("params").cloned().unwrap_or_else(|| json!({}));

        if method.starts_with("notifications/") {
            return None;
        }
        let id = id?;
        Some(match method {
            "initialize" => json_rpc_result(id, initialize_result(&params)),
            "ping" => json_rpc_result(id, json!({})),
            "tools/list" => json_rpc_result(id, json!({ "tools": tool_definitions() })),
            "tools/call" => json_rpc_result(id, self.call_tool(&params)),
            _ => json_rpc_error_value(id, -32601, "不支持的方法"),
        })
    }

    fn call_tool(&self, params: &Value) -> Value {
        let Some(name) = params.get("name").and_then(Value::as_str) else {
            return tool_error("缺少工具名称");
        };
        let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
        let store = match load_store_from_directory(&self.root) {
            Ok(store) => store,
            Err(error) => return tool_error(&format!("无法读取知识库：{error}")),
        };

        match name {
            "list_knowledge_bases" => list_knowledge_bases(&store),
            "search_documents" => search_documents(&store, &arguments),
            "read_document" => read_document(&store, &arguments),
            _ => tool_error("未知工具"),
        }
    }
}

pub(crate) fn run(args: &[String]) -> Result<(), String> {
    let root = parse_directory_argument(args)?;
    let server = McpServer::new(root)?;
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();

    for line in stdin.lock().lines() {
        let line = line.map_err(|error| format!("无法读取 MCP 请求：{error}"))?;
        let response = if line.len() > MAX_REQUEST_BYTES {
            Some(json_rpc_error(Value::Null, -32600, "MCP 请求过大"))
        } else if line.trim().is_empty() {
            None
        } else {
            server.handle_line(&line)
        };
        if let Some(response) = response {
            stdout
                .write_all(response.as_bytes())
                .and_then(|_| stdout.write_all(b"\n"))
                .and_then(|_| stdout.flush())
                .map_err(|error| format!("无法写入 MCP 响应：{error}"))?;
        }
    }
    Ok(())
}

fn parse_directory_argument(args: &[String]) -> Result<PathBuf, String> {
    let mut directory: Option<PathBuf> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--directory" => {
                let value = args.get(index + 1).ok_or("--directory 缺少目录路径")?;
                if directory.is_some() {
                    return Err("--directory 只能指定一次".to_string());
                }
                directory = Some(PathBuf::from(value));
                index += 2;
            }
            argument => return Err(format!("不支持的 MCP 参数：{argument}")),
        }
    }
    directory.ok_or_else(|| "MCP 需要通过 --directory 指定知识库目录".to_string())
}

fn initialize_result(params: &Value) -> Value {
    let requested = params.get("protocolVersion").and_then(Value::as_str);
    let protocol_version = match requested {
        Some("2024-11-05" | "2025-03-26" | "2025-06-18") => requested.unwrap_or(MCP_PROTOCOL_VERSION),
        _ => MCP_PROTOCOL_VERSION,
    };
    json!({
        "protocolVersion": protocol_version,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": {
            "name": "orange-run-knowledge",
            "title": "拿了桔子跑啊 · 本地知识库",
            "version": env!("CARGO_PKG_VERSION")
        },
        "instructions": "只读访问用户明确指定的拿了桔子跑啊知识库。搜索与读取不会返回回收站文档。"
    })
}

fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "list_knowledge_bases",
            "title": "列出知识库",
            "description": "列出知识库名称、ID 与未删除文档数量，不返回文档正文。",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
        }),
        json!({
            "name": "search_documents",
            "title": "搜索文档",
            "description": "在全部或指定知识库的标题、正文和标签中搜索，不包含回收站文档。",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "搜索关键词，最多 200 个字符" },
                    "knowledgeBaseId": { "type": "string", "description": "可选的知识库 ID" },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 50, "default": 20 }
                },
                "required": ["query"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "read_document",
            "title": "读取文档",
            "description": "按搜索结果中的稳定 ID 读取一篇未删除文档，正文最多返回 100000 个字符。",
            "inputSchema": {
                "type": "object",
                "properties": { "documentId": { "type": "string" } },
                "required": ["documentId"],
                "additionalProperties": false
            }
        }),
    ]
}

fn list_knowledge_bases(store: &NotesStore) -> Value {
    let counts = store
        .notes
        .iter()
        .filter(|note| note.deleted_at.is_none())
        .fold(HashMap::<&str, usize>::new(), |mut counts, note| {
            *counts.entry(note.knowledge_base_id.as_str()).or_default() += 1;
            counts
        });
    let knowledge_bases: Vec<Value> = store
        .knowledge_bases
        .iter()
        .map(|base| json!({
            "id": base.id,
            "name": base.name,
            "documentCount": counts.get(base.id.as_str()).copied().unwrap_or(0)
        }))
        .collect();
    tool_success(json!({ "knowledgeBases": knowledge_bases }))
}

fn search_documents(store: &NotesStore, arguments: &Value) -> Value {
    let Some(query) = arguments.get("query").and_then(Value::as_str).map(str::trim) else {
        return tool_error("query 必须是非空字符串");
    };
    if query.is_empty() {
        return tool_error("query 不能为空");
    }
    if query.chars().count() > MAX_QUERY_CHARS {
        return tool_error("query 不能超过 200 个字符");
    }
    let limit = arguments
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(20)
        .clamp(1, MAX_SEARCH_RESULTS as u64) as usize;
    let knowledge_base_id = arguments
        .get("knowledgeBaseId")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if let Some(id) = knowledge_base_id {
        if !store.knowledge_bases.iter().any(|base| base.id == id) {
            return tool_error("指定的知识库不存在");
        }
    }

    let query_lower = query.to_lowercase();
    let base_names: HashMap<&str, &str> = store
        .knowledge_bases
        .iter()
        .map(|base| (base.id.as_str(), base.name.as_str()))
        .collect();
    let mut matches: Vec<(&Note, u8)> = store
        .notes
        .iter()
        .filter(|note| note.deleted_at.is_none())
        .filter(|note| knowledge_base_id.map_or(true, |id| note.knowledge_base_id == id))
        .filter_map(|note| match_rank(note, &query_lower).map(|rank| (note, rank)))
        .collect();
    matches.sort_by(|(left, left_rank), (right, right_rank)| {
        left_rank.cmp(right_rank).then_with(|| right.updated_at.cmp(&left.updated_at))
    });

    let total = matches.len();
    let documents: Vec<Value> = matches
        .into_iter()
        .take(limit)
        .map(|(note, _)| json!({
            "id": note.id,
            "title": display_title(note),
            "knowledgeBaseId": note.knowledge_base_id,
            "knowledgeBaseName": base_names.get(note.knowledge_base_id.as_str()).copied().unwrap_or("未知知识库"),
            "tags": note.tags,
            "updatedAt": note.updated_at,
            "snippet": content_snippet(&note.content, &query_lower, 240)
        }))
        .collect();
    tool_success(json!({ "query": query, "total": total, "documents": documents }))
}

fn read_document(store: &NotesStore, arguments: &Value) -> Value {
    let Some(document_id) = arguments
        .get("documentId")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return tool_error("documentId 必须是非空字符串");
    };
    let Some(note) = store
        .notes
        .iter()
        .find(|note| note.id == document_id && note.deleted_at.is_none())
    else {
        return tool_error("文档不存在或已在回收站");
    };
    let Some(base) = store
        .knowledge_bases
        .iter()
        .find(|base| base.id == note.knowledge_base_id)
    else {
        return tool_error("文档所属知识库不存在");
    };

    let character_count = note.content.chars().count();
    let content = take_characters(&note.content, MAX_DOCUMENT_CHARS);
    tool_success(json!({
        "document": {
            "id": note.id,
            "title": display_title(note),
            "knowledgeBaseId": base.id,
            "knowledgeBaseName": base.name,
            "parentId": note.parent_id,
            "tags": note.tags,
            "createdAt": note.created_at,
            "updatedAt": note.updated_at,
            "content": content,
            "characterCount": character_count,
            "truncated": character_count > MAX_DOCUMENT_CHARS
        }
    }))
}

fn match_rank(note: &Note, query_lower: &str) -> Option<u8> {
    let title = note.title.to_lowercase();
    let tags = note.tags.join(" ").to_lowercase();
    let content = note.content.to_lowercase();
    if title == query_lower {
        Some(0)
    } else if title.starts_with(query_lower) {
        Some(1)
    } else if title.contains(query_lower) {
        Some(2)
    } else if tags.contains(query_lower) {
        Some(3)
    } else if content.contains(query_lower) {
        Some(4)
    } else {
        None
    }
}

fn display_title(note: &Note) -> &str {
    let title = note.title.trim();
    if title.is_empty() { "无标题文档" } else { title }
}

fn plain_text(content: &str) -> String {
    content
        .chars()
        .map(|character| {
            if matches!(character, '#' | '*' | '_' | '`' | '>' | '[' | ']' | '~') {
                ' '
            } else {
                character
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn content_snippet(content: &str, query_lower: &str, limit: usize) -> String {
    let plain = plain_text(content);
    if plain.is_empty() {
        return "暂无正文".to_string();
    }
    let characters: Vec<char> = plain.chars().collect();
    if characters.len() <= limit {
        return plain;
    }
    let lower: Vec<char> = plain.to_lowercase().chars().collect();
    let needle: Vec<char> = query_lower.chars().collect();
    let position = if needle.is_empty() || needle.len() > lower.len() {
        None
    } else {
        lower.windows(needle.len()).position(|window| window == needle)
    };
    let start = position.map_or(0, |position| position.saturating_sub(limit / 3));
    let end = (start + limit).min(characters.len());
    format!(
        "{}{}{}",
        if start > 0 { "…" } else { "" },
        characters[start..end].iter().collect::<String>(),
        if end < characters.len() { "…" } else { "" }
    )
}

fn take_characters(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

fn tool_success(data: Value) -> Value {
    let text = serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string());
    json!({
        "content": [{ "type": "text", "text": text }],
        "structuredContent": data,
        "isError": false
    })
}

fn tool_error(message: &str) -> Value {
    json!({ "content": [{ "type": "text", "text": message }], "isError": true })
}

fn json_rpc_result(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn json_rpc_error(id: Value, code: i64, message: &str) -> String {
    json_rpc_error_value(id, code, message).to_string()
}

fn json_rpc_error_value(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::write_store_to_directory;
    use crate::models::KnowledgeBase;
    use std::fs;

    fn sample_store() -> NotesStore {
        let knowledge_base = KnowledgeBase {
            id: "base-main".to_string(),
            name: "产品知识".to_string(),
            created_at: "2026-08-24T00:00:00Z".to_string(),
        };
        let knowledge_base_id = knowledge_base.id.clone();
        let note = |id: &str, title: &str, content: &str, deleted_at: Option<&str>| Note {
            id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            knowledge_base_id: knowledge_base_id.clone(),
            parent_id: None,
            deleted_at: deleted_at.map(str::to_string),
            pinned: false,
            tags: vec!["AI".to_string()],
            created_at: "2026-08-24T00:00:00Z".to_string(),
            updated_at: "2026-08-24T01:00:00Z".to_string(),
        };
        NotesStore {
            knowledge_bases: vec![knowledge_base],
            notes: vec![
                note("note-active", "AI 产品笔记", "安全的本地知识正文", None),
                note("note-second", "搜索说明", "使用 AI 搜索知识", None),
                note("note-deleted", "回收站秘密", "不应被搜索到", Some("2026-08-24T02:00:00Z")),
            ],
        }
    }

    fn server_fixture() -> (tempfile::TempDir, McpServer) {
        let directory = tempfile::tempdir().expect("temporary directory should exist");
        write_store_to_directory(directory.path(), &sample_store()).expect("store should be written");
        let server = McpServer::new(directory.path()).expect("server should open the library");
        (directory, server)
    }

    fn request(server: &McpServer, method: &str, params: Value) -> Value {
        server
            .handle_request(&json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }))
            .expect("request should receive a response")
    }

    fn call_tool(server: &McpServer, name: &str, arguments: Value) -> Value {
        request(server, "tools/call", json!({ "name": name, "arguments": arguments }))["result"].clone()
    }

    #[test]
    fn initializes_and_lists_read_only_tools() {
        let (_directory, server) = server_fixture();
        let initialized = request(&server, "initialize", json!({ "protocolVersion": "2025-06-18" }));
        assert_eq!(initialized["result"]["serverInfo"]["name"], "orange-run-knowledge");

        let tools = request(&server, "tools/list", json!({}));
        let names: Vec<&str> = tools["result"]["tools"]
            .as_array()
            .expect("tools should be an array")
            .iter()
            .filter_map(|tool| tool["name"].as_str())
            .collect();
        assert_eq!(names, vec!["list_knowledge_bases", "search_documents", "read_document"]);
    }

    #[test]
    fn lists_bases_and_limits_search_results() {
        let (_directory, server) = server_fixture();
        let bases = call_tool(&server, "list_knowledge_bases", json!({}));
        assert_eq!(bases["structuredContent"]["knowledgeBases"][0]["documentCount"], 2);
        assert!(!bases.to_string().contains("安全的本地知识正文"));

        let results = call_tool(&server, "search_documents", json!({ "query": "AI", "limit": 1 }));
        assert_eq!(results["structuredContent"]["total"], 2);
        assert_eq!(results["structuredContent"]["documents"].as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn reads_active_document_but_rejects_deleted_and_unknown_ids() {
        let (_directory, server) = server_fixture();
        let active = call_tool(&server, "read_document", json!({ "documentId": "note-active" }));
        assert_eq!(active["structuredContent"]["document"]["content"], "安全的本地知识正文");
        assert!(!active.to_string().contains("使用 AI 搜索知识"));

        let deleted = call_tool(&server, "read_document", json!({ "documentId": "note-deleted" }));
        assert_eq!(deleted["isError"], true);
        let unknown = call_tool(&server, "read_document", json!({ "documentId": "missing" }));
        assert_eq!(unknown["isError"], true);

        let hidden = call_tool(&server, "search_documents", json!({ "query": "秘密" }));
        assert_eq!(hidden["structuredContent"]["total"], 0);
    }

    #[test]
    fn rejects_manifest_paths_outside_the_library() {
        let directory = tempfile::tempdir().expect("temporary directory should exist");
        let root = directory.path().join("library");
        fs::create_dir(&root).expect("library directory should exist");
        fs::write(directory.path().join("outside.md"), "private").expect("outside file should exist");
        fs::write(
            root.join(".mojian-library.json"),
            r#"{"version":1,"knowledgeBases":[],"files":[{"noteId":"outside","relativePath":"../outside.md"}]}"#,
        )
        .expect("manifest should be written");
        let error = McpServer::new(&root).err().expect("unsafe manifest should fail");
        assert!(error.contains("无效路径"));
    }

    #[test]
    fn notifications_do_not_write_a_response() {
        let (_directory, server) = server_fixture();
        assert!(server
            .handle_request(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }))
            .is_none());
    }
}
