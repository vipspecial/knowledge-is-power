use crate::{
    models::{ImportedMarkdown, KnowledgeBase, Note, NotesStore},
    settings::{load_app_settings, save_app_settings},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Component, Path, PathBuf},
};
use tauri::Manager;

const MAX_NOTE_COUNT: usize = 10_000;
const MAX_KNOWLEDGE_BASE_COUNT: usize = 100;
const MAX_STORE_BYTES: usize = 20 * 1024 * 1024;
const MANIFEST_NAME: &str = ".orange-run-notes-library.json";
const LEGACY_MANIFEST_NAME: &str = ".mojian-library.json";
const METADATA_PREFIX: &str = "<!-- orange-run-notes-meta:";
const LEGACY_METADATA_PREFIX: &str = "<!-- mojian-meta:";

/// The manifest owns library-level structure and maps stable note IDs to files.
/// User-authored content stays in ordinary Markdown files rather than a database.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LibraryManifest {
    version: u32,
    #[serde(default)]
    knowledge_bases: Vec<KnowledgeBase>,
    #[serde(default)]
    files: Vec<ManifestFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestFile {
    note_id: String,
    relative_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NoteMetadata {
    id: String,
    knowledge_base_id: String,
    #[serde(default)]
    parent_id: Option<String>,
    // Soft-delete state is embedded in the Markdown metadata so the trash can
    // survive restarts and document-directory moves without a second database.
    #[serde(default)]
    deleted_at: Option<String>,
    pinned: bool,
    tags: Vec<String>,
    created_at: String,
    updated_at: String,
}

fn legacy_notes_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法获取应用数据目录：{error}"))?;
    Ok(data_dir.join("notes.json"))
}

fn library_root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(PathBuf::from(load_app_settings(app)?.document_directory))
}

fn parse_legacy_store(data: &[u8]) -> Result<NotesStore, String> {
    if let Ok(store) = serde_json::from_slice::<NotesStore>(data) {
        return Ok(store);
    }
    let notes = serde_json::from_slice::<Vec<Note>>(data)
        .map_err(|error| format!("笔记文件格式无效：{error}"))?;
    Ok(NotesStore {
        knowledge_bases: Vec::new(),
        notes,
    })
}

fn safe_component(value: &str, fallback: &str) -> String {
    let mut result = String::new();
    let mut previous_separator = false;
    for character in value.chars().take(80) {
        if character.is_alphanumeric() || matches!(character, '-' | '_') {
            result.push(character);
            previous_separator = false;
        } else if character.is_whitespace() && !previous_separator {
            result.push('_');
            previous_separator = true;
        }
    }
    let result = result.trim_matches(['_', '-']);
    if result.is_empty() {
        fallback.to_string()
    } else {
        result.to_string()
    }
}

fn short_id(id: &str) -> String {
    let value: String = id.chars().filter(|value| value.is_ascii_alphanumeric()).take(8).collect();
    if value.is_empty() {
        "document".to_string()
    } else {
        value
    }
}

fn relative_note_path(note: &Note, knowledge_base: &KnowledgeBase) -> String {
    let folder = format!(
        "{}--{}",
        safe_component(&knowledge_base.name, "知识库"),
        short_id(&knowledge_base.id)
    );
    let file = format!(
        "{}--{}.md",
        safe_component(&note.title, "无标题笔记"),
        short_id(&note.id)
    );
    format!("{folder}/{file}")
}

fn safe_relative_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("文档清单包含无效路径".to_string());
    }
    Ok(root.join(path))
}

fn render_note(note: &Note) -> Result<String, String> {
    let metadata = NoteMetadata {
        id: note.id.clone(),
        knowledge_base_id: note.knowledge_base_id.clone(),
        parent_id: note.parent_id.clone(),
        deleted_at: note.deleted_at.clone(),
        pinned: note.pinned,
        tags: note.tags.clone(),
        created_at: note.created_at.clone(),
        updated_at: note.updated_at.clone(),
    };
    let metadata = serde_json::to_vec(&metadata)
        .map_err(|error| format!("无法整理文档元数据：{error}"))?;
    let metadata = URL_SAFE_NO_PAD.encode(metadata);
    let title = note.title.replace(['\r', '\n'], " ");
    Ok(format!(
        "{METADATA_PREFIX}{metadata} -->\n\n# {}\n\n{}\n",
        title.trim(),
        note.content
    ))
}

fn parse_note(data: &str) -> Result<Note, String> {
    let first_line = data.lines().next().ok_or("Markdown 文档为空")?;
    let encoded = [METADATA_PREFIX, LEGACY_METADATA_PREFIX]
        .iter()
        .find_map(|prefix| first_line.strip_prefix(prefix))
        .and_then(|line| line.strip_suffix(" -->"))
        .ok_or("Markdown 文档缺少拿了桔子跑啊元数据")?;
    let metadata = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|error| format!("文档元数据编码无效：{error}"))?;
    let metadata: NoteMetadata = serde_json::from_slice(&metadata)
        .map_err(|error| format!("文档元数据无效：{error}"))?;

    let mut body = data[first_line.len()..].trim_start_matches(['\r', '\n']);
    let title = if let Some(line_end) = body.find('\n') {
        let first = body[..line_end].trim_end_matches('\r');
        body = body[line_end + 1..].trim_start_matches(['\r', '\n']);
        first.strip_prefix('#').unwrap_or(first).trim().to_string()
    } else {
        body.strip_prefix('#').unwrap_or(body).trim().to_string()
    };

    Ok(Note {
        id: metadata.id,
        title,
        content: body.trim_end_matches(['\r', '\n']).to_string(),
        knowledge_base_id: metadata.knowledge_base_id,
        parent_id: metadata.parent_id,
        deleted_at: metadata.deleted_at,
        pinned: metadata.pinned,
        tags: metadata.tags,
        created_at: metadata.created_at,
        updated_at: metadata.updated_at,
    })
}

fn read_manifest(root: &Path) -> Result<Option<LibraryManifest>, String> {
    let current_path = root.join(MANIFEST_NAME);
    let legacy_path = root.join(LEGACY_MANIFEST_NAME);
    let path = if current_path.exists() {
        current_path
    } else {
        legacy_path
    };
    if !path.exists() {
        return Ok(None);
    }
    let file_size = fs::metadata(&path)
        .map_err(|error| format!("无法读取知识库清单信息：{error}"))?
        .len();
    if file_size > MAX_STORE_BYTES as u64 {
        return Err("知识库清单不能超过 20 MB".to_string());
    }
    let data = fs::read(path).map_err(|error| format!("无法读取知识库清单：{error}"))?;
    serde_json::from_slice(&data)
        .map(Some)
        .map_err(|error| format!("知识库清单格式无效：{error}"))
}

fn write_if_changed(path: &Path, data: &[u8]) -> Result<(), String> {
    if path.exists() {
        if let Ok(current) = fs::read(path) {
            if current == data {
                return Ok(());
            }
        }
    }
    let parent = path.parent().ok_or("文档路径无效")?;
    fs::create_dir_all(parent).map_err(|error| format!("无法创建文档目录：{error}"))?;
    let temporary = path.with_extension("orange-run-notes.tmp");
    fs::write(&temporary, data).map_err(|error| format!("无法写入文档：{error}"))?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| format!("无法更新文档：{error}"))?;
    }
    fs::rename(temporary, path).map_err(|error| format!("无法完成文档保存：{error}"))
}

pub(crate) fn write_store_to_directory(root: &Path, store: &NotesStore) -> Result<(), String> {
    if store.notes.len() > MAX_NOTE_COUNT {
        return Err(format!("笔记数量不能超过 {MAX_NOTE_COUNT} 篇"));
    }
    if store.knowledge_bases.len() > MAX_KNOWLEDGE_BASE_COUNT {
        return Err(format!("知识库数量不能超过 {MAX_KNOWLEDGE_BASE_COUNT} 个"));
    }
    fs::create_dir_all(root).map_err(|error| format!("无法创建文档目录：{error}"))?;
    let old_manifest = read_manifest(root)?.unwrap_or_default();
    let knowledge_bases: HashMap<&str, &KnowledgeBase> = store
        .knowledge_bases
        .iter()
        .map(|base| (base.id.as_str(), base))
        .collect();

    let mut files = Vec::with_capacity(store.notes.len());
    for note in &store.notes {
        let knowledge_base = knowledge_bases
            .get(note.knowledge_base_id.as_str())
            .ok_or_else(|| format!("笔记“{}”没有有效的知识库", note.title))?;
        let relative_path = relative_note_path(note, knowledge_base);
        let path = safe_relative_path(root, &relative_path)?;
        let markdown = render_note(note)?;
        write_if_changed(&path, markdown.as_bytes())?;
        files.push(ManifestFile {
            note_id: note.id.clone(),
            relative_path,
        });
    }

    let new_paths: HashSet<&str> = files.iter().map(|file| file.relative_path.as_str()).collect();
    for old_file in &old_manifest.files {
        if new_paths.contains(old_file.relative_path.as_str()) {
            continue;
        }
        let old_path = safe_relative_path(root, &old_file.relative_path)?;
        if old_path.extension().and_then(|value| value.to_str()) == Some("md") && old_path.exists() {
            fs::remove_file(&old_path).map_err(|error| format!("无法移除旧文档：{error}"))?;
            if let Some(parent) = old_path.parent() {
                let _ = fs::remove_dir(parent);
            }
        }
    }

    let manifest = LibraryManifest {
        version: 1,
        knowledge_bases: store.knowledge_bases.clone(),
        files,
    };
    let data = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| format!("无法整理知识库清单：{error}"))?;
    if data.len() > MAX_STORE_BYTES {
        return Err("知识库清单不能超过 20 MB".to_string());
    }
    write_if_changed(&root.join(MANIFEST_NAME), &data)?;
    let legacy_manifest = root.join(LEGACY_MANIFEST_NAME);
    if legacy_manifest.exists() {
        fs::remove_file(legacy_manifest)
            .map_err(|error| format!("无法移除旧知识库清单：{error}"))?;
    }
    Ok(())
}

fn read_store_from_directory(root: &Path, manifest: LibraryManifest) -> Result<NotesStore, String> {
    if manifest.files.len() > MAX_NOTE_COUNT {
        return Err(format!("笔记数量不能超过 {MAX_NOTE_COUNT} 篇"));
    }
    if manifest.knowledge_bases.len() > MAX_KNOWLEDGE_BASE_COUNT {
        return Err(format!("知识库数量不能超过 {MAX_KNOWLEDGE_BASE_COUNT} 个"));
    }
    let canonical_root = fs::canonicalize(root)
        .map_err(|error| format!("无法确认文档目录：{error}"))?;
    let mut notes = Vec::with_capacity(manifest.files.len());
    let mut note_ids = HashSet::with_capacity(manifest.files.len());
    for file in manifest.files {
        let path = safe_relative_path(root, &file.relative_path)?;
        if !path.exists() {
            continue;
        }
        let canonical_path = fs::canonicalize(&path)
            .map_err(|error| format!("无法确认文档路径“{}”：{error}", file.relative_path))?;
        if !canonical_path.starts_with(&canonical_root) {
            return Err(format!("文档路径越过知识库目录：{}", file.relative_path));
        }
        let file_size = fs::metadata(&canonical_path)
            .map_err(|error| format!("无法读取文档信息“{}”：{error}", file.relative_path))?
            .len();
        if file_size > MAX_STORE_BYTES as u64 {
            return Err(format!("文档“{}”不能超过 20 MB", file.relative_path));
        }
        let data = fs::read_to_string(&canonical_path)
            .map_err(|error| format!("无法读取文档“{}”：{error}", file.relative_path))?;
        let note = parse_note(&data)?;
        if note.id != file.note_id {
            return Err(format!("文档“{}”的 ID 与清单不一致", file.relative_path));
        }
        if !note_ids.insert(note.id.clone()) {
            return Err(format!("知识库清单包含重复文档 ID：{}", note.id));
        }
        notes.push(note);
    }
    Ok(NotesStore {
        knowledge_bases: manifest.knowledge_bases,
        notes,
    })
}

/// Load a library from an explicitly selected root without reading app settings.
/// The MCP server uses this boundary so it cannot discover unrelated local data.
pub(crate) fn load_store_from_directory(root: &Path) -> Result<NotesStore, String> {
    if !root.is_dir() {
        return Err("指定的知识库目录不存在或不是文件夹".to_string());
    }
    let manifest = read_manifest(root)?.ok_or("指定目录不是拿了桔子跑啊知识库")?;
    read_store_from_directory(root, manifest)
}

#[tauri::command]
pub(crate) fn load_store(app: tauri::AppHandle) -> Result<NotesStore, String> {
    let root = library_root(&app)?;
    if let Some(manifest) = read_manifest(&root)? {
        return read_store_from_directory(&root, manifest);
    }

    let legacy_path = legacy_notes_path(&app)?;
    if legacy_path.exists() {
        let data = fs::read(&legacy_path).map_err(|error| format!("无法读取旧笔记：{error}"))?;
        return parse_legacy_store(&data);
    }
    Ok(NotesStore::default())
}

#[tauri::command]
pub(crate) fn save_store(app: tauri::AppHandle, store: NotesStore) -> Result<(), String> {
    let root = library_root(&app)?;
    write_store_to_directory(&root, &store)
}

#[tauri::command]
pub(crate) async fn choose_document_directory(
    app: tauri::AppHandle,
    store: NotesStore,
) -> Result<Option<String>, String> {
    let current = library_root(&app)?;
    let dialog = rfd::AsyncFileDialog::new().set_title("选择拿了桔子跑啊文档目录");
    let dialog = if current.exists() {
        dialog.set_directory(&current)
    } else {
        dialog
    };
    let Some(folder) = dialog.pick_folder().await else {
        return Ok(None);
    };
    let new_root = folder.path().to_path_buf();
    write_store_to_directory(&new_root, &store)?;

    let mut settings = load_app_settings(&app)?;
    settings.document_directory = new_root.to_string_lossy().into_owned();
    save_app_settings(&app, &settings)?;
    Ok(Some(settings.document_directory))
}

fn safe_export_title(title: &str) -> String {
    let value: String = title
        .chars()
        .filter(|character| {
            !matches!(
                character,
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
            )
        })
        .take(80)
        .collect();
    let value = value.trim();
    if value.is_empty() {
        "无标题笔记".to_string()
    } else {
        value.to_string()
    }
}

#[tauri::command]
pub(crate) async fn import_markdown() -> Result<Option<ImportedMarkdown>, String> {
    let Some(file) = rfd::AsyncFileDialog::new()
        .add_filter("Markdown", &["md", "markdown", "txt"])
        .pick_file()
        .await
    else {
        return Ok(None);
    };

    let content = fs::read_to_string(file.path())
        .map_err(|error| format!("无法读取 Markdown 文件：{error}"))?;
    if content.len() > MAX_STORE_BYTES {
        return Err("导入的文件不能超过 20 MB".to_string());
    }
    let title = file
        .path()
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("导入的笔记")
        .to_string();
    Ok(Some(ImportedMarkdown { title, content }))
}

#[tauri::command]
pub(crate) async fn export_markdown(note: Note) -> Result<Option<String>, String> {
    let file_name = format!("{}.md", safe_export_title(&note.title));
    let Some(file) = rfd::AsyncFileDialog::new()
        .add_filter("Markdown", &["md"])
        .set_file_name(&file_name)
        .save_file()
        .await
    else {
        return Ok(None);
    };

    let markdown = if note.title.trim().is_empty() {
        note.content
    } else {
        format!("# {}\n\n{}", note.title.trim(), note.content)
    };
    fs::write(file.path(), markdown)
        .map_err(|error| format!("无法导出 Markdown 文件：{error}"))?;
    Ok(Some(file.path().to_string_lossy().into_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_note() -> Note {
        Note {
            id: "note-12345678".to_string(),
            title: "测试笔记".to_string(),
            content: "## 内容\n\n正文".to_string(),
            knowledge_base_id: "base-12345678".to_string(),
            parent_id: None,
            deleted_at: None,
            pinned: true,
            tags: vec!["测试".to_string()],
            created_at: "2026-08-18T00:00:00Z".to_string(),
            updated_at: "2026-08-18T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn note_markdown_round_trip_preserves_data() {
        let mut note = sample_note();
        note.parent_id = Some("parent-note-87654321".to_string());
        note.deleted_at = Some("2026-08-21T00:00:00Z".to_string());
        let markdown = render_note(&note).expect("note should render");
        let parsed = parse_note(&markdown).expect("note should parse");
        assert_eq!(parsed.id, note.id);
        assert_eq!(parsed.title, note.title);
        assert_eq!(parsed.content, note.content);
        assert_eq!(parsed.parent_id, note.parent_id);
        assert_eq!(parsed.deleted_at, note.deleted_at);
        assert_eq!(parsed.tags, note.tags);
        assert!(parsed.pinned);
        assert!(markdown.starts_with(METADATA_PREFIX));
    }

    #[test]
    fn reads_legacy_markdown_metadata() {
        let note = sample_note();
        let markdown = render_note(&note)
            .expect("note should render")
            .replacen(METADATA_PREFIX, LEGACY_METADATA_PREFIX, 1);

        let parsed = parse_note(&markdown).expect("legacy metadata should remain readable");

        assert_eq!(parsed.id, note.id);
        assert_eq!(parsed.content, note.content);
    }

    #[test]
    fn writes_real_markdown_files_and_loads_them_back() {
        let directory = tempfile::tempdir().expect("temporary directory should be created");
        let knowledge_base = KnowledgeBase {
            id: "base-12345678".to_string(),
            name: "产品知识".to_string(),
            created_at: "2026-08-18T00:00:00Z".to_string(),
        };
        let store = NotesStore {
            knowledge_bases: vec![knowledge_base],
            notes: vec![sample_note()],
        };
        write_store_to_directory(directory.path(), &store).expect("library should be written");
        assert!(directory.path().join(MANIFEST_NAME).exists());
        let manifest = read_manifest(directory.path())
            .expect("manifest should be readable")
            .expect("manifest should exist");
        assert_eq!(manifest.files.len(), 1);
        let markdown_path = safe_relative_path(
            directory.path(),
            &manifest.files[0].relative_path,
        )
        .expect("relative path should be safe");
        assert_eq!(markdown_path.extension().and_then(|value| value.to_str()), Some("md"));
        assert!(markdown_path.exists());

        let loaded = read_store_from_directory(directory.path(), manifest)
            .expect("library should load from markdown");
        assert_eq!(loaded.notes.len(), 1);
        assert_eq!(loaded.notes[0].content, store.notes[0].content);
    }

    #[test]
    fn rewrites_legacy_manifest_with_the_current_name() {
        let directory = tempfile::tempdir().expect("temporary directory should be created");
        let knowledge_base = KnowledgeBase {
            id: "base-12345678".to_string(),
            name: "产品知识".to_string(),
            created_at: "2026-08-18T00:00:00Z".to_string(),
        };
        let store = NotesStore {
            knowledge_bases: vec![knowledge_base],
            notes: vec![sample_note()],
        };
        write_store_to_directory(directory.path(), &store).expect("write current library");
        fs::rename(
            directory.path().join(MANIFEST_NAME),
            directory.path().join(LEGACY_MANIFEST_NAME),
        )
        .expect("prepare legacy manifest");

        write_store_to_directory(directory.path(), &store).expect("rewrite legacy library");

        assert!(directory.path().join(MANIFEST_NAME).exists());
        assert!(!directory.path().join(LEGACY_MANIFEST_NAME).exists());
    }

    #[test]
    fn loads_notes_saved_by_the_first_version() {
        let json = r#"[{
            "id":"1",
            "title":"旧笔记",
            "content":"内容",
            "createdAt":"2026-08-18T00:00:00Z",
            "updatedAt":"2026-08-18T00:00:00Z"
        }]"#;
        let store = parse_legacy_store(json.as_bytes()).expect("old note array should still load");
        assert_eq!(store.notes.len(), 1);
        assert!(store.notes[0].knowledge_base_id.is_empty());
    }

    #[test]
    fn removes_invalid_file_name_characters() {
        assert_eq!(safe_export_title("周报: 8/18?"), "周报 818");
        assert_eq!(safe_export_title("<>|"), "无标题笔记");
    }
}
