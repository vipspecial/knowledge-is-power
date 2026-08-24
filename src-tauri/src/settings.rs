use crate::models::{AiSettings, AppSettings, SettingsView};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::Manager;

const APP_IDENTIFIER: &str = "com.peter.orange-run-notes";
const LEGACY_APP_IDENTIFIER: &str = "com.peter.mojian";
const CREDENTIAL_ACCOUNT: &str = "ai-api-key";
const LEGACY_API_KEY_FILE: &str = "ai-api-key";
const LEGACY_API_KEY_TEMP_FILE: &str = "ai-api-key.tmp";
const LEGACY_MCP_ACCESS_FILE: &str = "mcp-access.json";

trait CredentialStore {
    fn get(&self) -> Result<Option<String>, String>;
    fn set(&self, value: &str) -> Result<(), String>;
    fn delete(&self) -> Result<(), String>;
}

struct SystemCredentialStore;

impl SystemCredentialStore {
    fn entry(&self) -> Result<keyring::Entry, String> {
        keyring::Entry::new(APP_IDENTIFIER, CREDENTIAL_ACCOUNT)
            .map_err(|error| format!("无法连接系统安全凭据存储：{error}"))
    }
}

impl CredentialStore for SystemCredentialStore {
    fn get(&self) -> Result<Option<String>, String> {
        match self.entry()?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(format!("无法读取系统安全凭据：{error}")),
        }
    }

    fn set(&self, value: &str) -> Result<(), String> {
        self.entry()?
            .set_password(value)
            .map_err(|error| format!("无法保存到系统安全凭据：{error}"))
    }

    fn delete(&self) -> Result<(), String> {
        match self.entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(format!("无法从系统安全凭据中移除密钥：{error}")),
        }
    }
}

fn app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| format!("无法获取应用数据目录：{error}"))
}

fn legacy_app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let current = app_data_dir(app)?;
    let parent = current.parent().ok_or("应用数据目录无效")?;
    Ok(parent.join(LEGACY_APP_IDENTIFIER))
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("settings.json"))
}

fn default_document_directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if let Ok(documents) = app.path().document_dir() {
        return Ok(documents.join("拿了桔子跑啊"));
    }
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法获取默认文档目录：{error}"))?;
    Ok(data_dir.join("library"))
}

fn infer_ai_provider(base_url: &str, protocol: &str) -> String {
    let base_url = base_url.to_ascii_lowercase();
    for (host, provider) in [
        ("api.deepseek.com", "deepseek"),
        ("dashscope.aliyuncs.com", "dashscope"),
        ("open.bigmodel.cn", "zhipu"),
        ("api.moonshot.cn", "moonshot"),
        ("ark.cn-beijing.volces.com", "volcengine"),
        ("api.siliconflow.cn", "siliconflow"),
        ("api.openai.com", "openai"),
        ("api.anthropic.com", "anthropic"),
        ("generativelanguage.googleapis.com", "gemini"),
        ("api.x.ai", "xai"),
        ("api.mistral.ai", "mistral"),
        ("openrouter.ai", "openrouter"),
    ] {
        if base_url.contains(host) {
            return provider.to_string();
        }
    }
    if protocol == "anthropic" && base_url.contains("anthropic") {
        return "anthropic".to_string();
    }
    "custom".to_string()
}

fn normalize_ai_models(settings: &mut AiSettings) {
    let mut models = Vec::new();
    for model in settings
        .models
        .iter()
        .map(|model| model.trim())
        .chain(std::iter::once(settings.model.trim()))
    {
        if !model.is_empty() && !models.iter().any(|item| item == model) {
            models.push(model.to_string());
        }
    }
    settings.models = models;
}

fn legacy_api_key_paths(app: &tauri::AppHandle) -> Result<Vec<PathBuf>, String> {
    let current = app_data_dir(app)?;
    let legacy = legacy_app_data_dir(app)?;
    Ok(vec![
        current.join(LEGACY_API_KEY_FILE),
        current.join(LEGACY_API_KEY_TEMP_FILE),
        legacy.join(LEGACY_API_KEY_FILE),
        legacy.join(LEGACY_API_KEY_TEMP_FILE),
    ])
}

fn remove_plaintext_api_keys(paths: &[PathBuf]) -> Result<(), String> {
    for path in paths {
        if path.exists() {
            fs::remove_file(path)
                .map_err(|error| format!("安全存储已启用，但无法移除旧明文密钥：{error}"))?;
        }
    }
    Ok(())
}

fn remove_legacy_app_dir_if_empty(app: &tauri::AppHandle) -> Result<(), String> {
    let path = legacy_app_data_dir(app)?;
    match fs::remove_dir(path) {
        Ok(()) => Ok(()),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
            ) =>
        {
            Ok(())
        }
        Err(error) => Err(format!("无法清理旧应用数据目录：{error}")),
    }
}

fn save_secure_api_key(store: &impl CredentialStore, value: &str) -> Result<(), String> {
    store.set(value)?;
    if store.get()?.as_deref() != Some(value) {
        return Err("系统安全凭据校验失败，API Key 未保存".to_string());
    }
    Ok(())
}

fn copy_known_file(source: &Path, destination: &Path) -> Result<(), String> {
    if !source.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(source)
        .map_err(|error| format!("无法读取旧应用数据：{error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("旧应用数据路径不是普通文件，已停止自动迁移".to_string());
    }
    if destination.exists() {
        let destination_metadata = fs::symlink_metadata(destination)
            .map_err(|error| format!("无法读取新应用数据：{error}"))?;
        if destination_metadata.file_type().is_symlink() || !destination_metadata.is_file() {
            return Err("新应用数据路径不是普通文件，已停止自动迁移".to_string());
        }
        let source_data = fs::read(source)
            .map_err(|error| format!("无法校验旧应用数据：{error}"))?;
        let destination_data = fs::read(destination)
            .map_err(|error| format!("无法校验新应用数据：{error}"))?;
        if source_data == destination_data {
            fs::remove_file(source)
                .map_err(|error| format!("无法清理已迁移的旧应用数据：{error}"))?;
        }
        return Ok(());
    }

    let parent = destination.parent().ok_or("新应用数据路径无效")?;
    fs::create_dir_all(parent).map_err(|error| format!("无法创建新应用数据目录：{error}"))?;
    let temporary = parent.join(format!(
        "{}.migration.tmp",
        destination
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("data")
    ));
    fs::copy(source, &temporary).map_err(|error| format!("无法复制旧应用数据：{error}"))?;
    let source_data = fs::read(source).map_err(|error| format!("无法校验旧应用数据：{error}"))?;
    let temporary_data = fs::read(&temporary)
        .map_err(|error| format!("无法校验迁移后的应用数据：{error}"))?;
    if source_data != temporary_data {
        let _ = fs::remove_file(&temporary);
        return Err("旧应用数据迁移校验失败，原文件已保留".to_string());
    }
    fs::rename(&temporary, destination)
        .map_err(|error| format!("无法启用迁移后的应用数据：{error}"))?;
    fs::remove_file(source).map_err(|error| format!("无法清理已迁移的旧应用数据：{error}"))
}

pub(crate) fn migrate_legacy_app_data(app: &tauri::AppHandle) -> Result<(), String> {
    let current = app_data_dir(app)?;
    let legacy = legacy_app_data_dir(app)?;
    if current != legacy && legacy.exists() {
        copy_known_file(&legacy.join("settings.json"), &current.join("settings.json"))?;
        copy_known_file(&legacy.join("notes.json"), &current.join("notes.json"))?;

        // 旧 MCP 客户端持有旧授权路径；删除授权文件可立即撤销其目录访问。
        let legacy_mcp = legacy.join(LEGACY_MCP_ACCESS_FILE);
        if legacy_mcp.exists() {
            fs::remove_file(&legacy_mcp)
                .map_err(|error| format!("无法停用旧 MCP 授权：{error}"))?;
        }
    }
    // 安全策略明确禁止继续读取旧明文 Key；升级后由用户重新填写。
    remove_plaintext_api_keys(&legacy_api_key_paths(app)?)?;
    remove_legacy_app_dir_if_empty(app)
}

pub(crate) fn get_api_key(app: &tauri::AppHandle) -> Result<Option<String>, String> {
    migrate_legacy_app_data(app)?;
    SystemCredentialStore.get()
}

pub(crate) fn validate_ai_settings(settings: &AiSettings) -> Result<(), String> {
    if settings.enabled {
        let url = reqwest::Url::parse(settings.base_url.trim())
            .map_err(|_| "AI API 地址格式无效".to_string())?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err("AI API 地址必须使用 http 或 https".to_string());
        }
        if settings.model.trim().is_empty() {
            return Err("AI 模型名称不能为空".to_string());
        }
        let provider = settings.provider.trim();
        if provider.is_empty()
            || provider.len() > 64
            || !provider
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
        {
            return Err("AI 服务商标识无效".to_string());
        }
        if !matches!(
            settings.protocol.as_str(),
            "chatCompletions" | "responses" | "anthropic"
        ) {
            return Err("不支持的 AI 接口协议".to_string());
        }
    }
    if !(0.0..=2.0).contains(&settings.temperature) {
        return Err("温度参数必须在 0 到 2 之间".to_string());
    }
    if !(2_000..=200_000).contains(&settings.max_context_chars) {
        return Err("上下文长度必须在 2,000 到 200,000 字符之间".to_string());
    }
    if settings.models.is_empty() || settings.models.len() > 20 {
        return Err("AI 模型列表必须包含 1 到 20 个模型".to_string());
    }
    if settings
        .models
        .iter()
        .any(|model| model.trim().is_empty() || model.chars().count() > 200)
    {
        return Err("AI 模型名称不能为空且不能超过 200 个字符".to_string());
    }
    if !settings.models.iter().any(|model| model == &settings.model) {
        return Err("当前 AI 模型不在已配置模型列表中".to_string());
    }
    Ok(())
}

pub(crate) fn load_app_settings(app: &tauri::AppHandle) -> Result<AppSettings, String> {
    migrate_legacy_app_data(app)?;
    let path = settings_path(app)?;
    let mut settings = if path.exists() {
        let data = fs::read(&path).map_err(|error| format!("无法读取设置：{error}"))?;
        serde_json::from_slice::<AppSettings>(&data)
            .map_err(|error| format!("设置文件格式无效：{error}"))?
    } else {
        AppSettings::default()
    };

    if settings.document_directory.trim().is_empty() {
        settings.document_directory = default_document_directory(app)?
            .to_string_lossy()
            .into_owned();
    }
    if settings.ai.provider.trim().is_empty() {
        settings.ai.provider = infer_ai_provider(&settings.ai.base_url, &settings.ai.protocol);
    }
    normalize_ai_models(&mut settings.ai);
    Ok(settings)
}

pub(crate) fn save_app_settings(
    app: &tauri::AppHandle,
    settings: &AppSettings,
) -> Result<(), String> {
    validate_ai_settings(&settings.ai)?;
    if !(300..=10_000).contains(&settings.general.auto_save_delay_ms) {
        return Err("自动保存等待时间必须在 300 到 10,000 毫秒之间".to_string());
    }
    if settings.document_directory.trim().is_empty() {
        return Err("文档目录不能为空".to_string());
    }
    let path = settings_path(app)?;
    let parent = path.parent().ok_or("设置保存路径无效")?;
    fs::create_dir_all(parent).map_err(|error| format!("无法创建设置目录：{error}"))?;
    let data = serde_json::to_vec_pretty(settings)
        .map_err(|error| format!("无法整理设置数据：{error}"))?;
    let temporary = parent.join("settings.json.tmp");
    fs::write(&temporary, data).map_err(|error| format!("无法写入设置：{error}"))?;
    if path.exists() {
        fs::remove_file(&path).map_err(|error| format!("无法更新设置：{error}"))?;
    }
    fs::rename(temporary, path).map_err(|error| format!("无法完成设置保存：{error}"))?;
    crate::mcp::update_access_directory(app, &settings.document_directory)
}

#[tauri::command]
pub(crate) fn load_settings(app: tauri::AppHandle) -> Result<SettingsView, String> {
    let settings = load_app_settings(&app)?;
    let credential = get_api_key(&app);
    Ok(SettingsView {
        settings,
        has_api_key: credential.as_ref().is_ok_and(Option::is_some),
        credential_error: credential.err(),
    })
}

#[tauri::command]
pub(crate) fn save_settings(
    app: tauri::AppHandle,
    settings: AppSettings,
    api_key: Option<String>,
) -> Result<SettingsView, String> {
    save_app_settings(&app, &settings)?;
    if let Some(api_key) = api_key {
        let api_key = api_key.trim();
        if !api_key.is_empty() {
            save_secure_api_key(&SystemCredentialStore, api_key)?;
            remove_plaintext_api_keys(&legacy_api_key_paths(&app)?)?;
        }
    }
    let credential = get_api_key(&app);
    Ok(SettingsView {
        settings,
        has_api_key: credential.as_ref().is_ok_and(Option::is_some),
        credential_error: credential.err(),
    })
}

#[tauri::command]
pub(crate) fn clear_ai_api_key(app: tauri::AppHandle) -> Result<(), String> {
    SystemCredentialStore.delete()?;
    remove_plaintext_api_keys(&legacy_api_key_paths(&app)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    struct FakeCredentialStore {
        value: RefCell<Option<String>>,
        reject_writes: bool,
    }

    impl CredentialStore for FakeCredentialStore {
        fn get(&self) -> Result<Option<String>, String> {
            Ok(self.value.borrow().clone())
        }

        fn set(&self, value: &str) -> Result<(), String> {
            if self.reject_writes {
                return Err("模拟安全存储失败".to_string());
            }
            self.value.replace(Some(value.to_string()));
            Ok(())
        }

        fn delete(&self) -> Result<(), String> {
            self.value.replace(None);
            Ok(())
        }
    }

    #[test]
    fn infers_known_and_custom_ai_providers() {
        assert_eq!(
            infer_ai_provider("https://api.deepseek.com/v1", "chatCompletions"),
            "deepseek"
        );
        assert_eq!(
            infer_ai_provider("https://api.anthropic.com/v1", "anthropic"),
            "anthropic"
        );
        assert_eq!(
            infer_ai_provider("http://localhost:11434/v1", "chatCompletions"),
            "custom"
        );
    }

    #[test]
    fn accepts_anthropic_and_migrates_legacy_provider_metadata() {
        let legacy: AiSettings = serde_json::from_value(serde_json::json!({
            "enabled": true,
            "baseUrl": "https://api.anthropic.com/v1",
            "protocol": "anthropic",
            "model": "claude-test",
            "temperature": 0.3,
            "maxContextChars": 30000
        }))
        .expect("deserialize legacy settings");
        assert!(legacy.provider.is_empty());
        assert!(legacy.models.is_empty());

        let mut migrated = legacy;
        migrated.provider = infer_ai_provider(&migrated.base_url, &migrated.protocol);
        normalize_ai_models(&mut migrated);
        assert_eq!(migrated.provider, "anthropic");
        assert_eq!(migrated.models, vec!["claude-test"]);
        validate_ai_settings(&migrated).expect("validate Anthropic settings");
    }

    #[test]
    fn saves_api_key_directly_in_secure_storage() {
        let store = FakeCredentialStore::default();

        save_secure_api_key(&store, "test-secret").expect("save API key");

        assert_eq!(store.get().unwrap().as_deref(), Some("test-secret"));
    }

    #[test]
    fn removes_legacy_plaintext_api_key_without_reading_it() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("ai-api-key");
        fs::write(&path, b"legacy-value").expect("write legacy API key");

        remove_plaintext_api_keys(std::slice::from_ref(&path)).expect("remove legacy API key");

        assert!(!path.exists());
    }

    #[test]
    fn reports_secure_storage_write_failure() {
        let store = FakeCredentialStore {
            reject_writes: true,
            ..Default::default()
        };

        let result = save_secure_api_key(&store, "test-secret");

        assert!(result.is_err());
        assert!(store.get().unwrap().is_none());
    }

    #[test]
    fn copies_known_app_data_before_removing_the_old_file() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let source = directory.path().join("old/settings.json");
        let destination = directory.path().join("new/settings.json");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, br#"{"version":1}"#).unwrap();

        copy_known_file(&source, &destination).expect("migrate settings");

        assert!(!source.exists());
        assert_eq!(fs::read(destination).unwrap(), br#"{"version":1}"#);
    }
}
