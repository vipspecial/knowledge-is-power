use crate::models::{AiSettings, AppSettings, SettingsView};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tauri::Manager;

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法获取应用数据目录：{error}"))?;
    Ok(data_dir.join("settings.json"))
}

fn api_key_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("无法获取应用数据目录：{error}"))?;
    Ok(data_dir.join("ai-api-key"))
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

fn read_api_key_file(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
}

fn write_api_key_file(path: &Path, api_key: &str) -> Result<(), String> {
    let parent = path.parent().ok_or("API Key 保存路径无效")?;
    fs::create_dir_all(parent).map_err(|error| format!("无法创建应用配置目录：{error}"))?;
    let temporary = parent.join("ai-api-key.tmp");
    let mut options = fs::OpenOptions::new();
    options.create(true).write(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .map_err(|error| format!("无法创建 API Key 配置：{error}"))?;
    file.write_all(api_key.trim().as_bytes())
        .map_err(|error| format!("无法保存 API Key：{error}"))?;
    file.sync_all()
        .map_err(|error| format!("无法完成 API Key 保存：{error}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))
            .map_err(|error| format!("无法限制 API Key 文件权限：{error}"))?;
    }
    if path.exists() {
        fs::remove_file(path).map_err(|error| format!("无法更新 API Key：{error}"))?;
    }
    fs::rename(&temporary, path).map_err(|error| format!("无法完成 API Key 保存：{error}"))
}

fn delete_api_key_file(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_file(path).map_err(|error| format!("无法移除 API Key：{error}"))?;
    }
    Ok(())
}

pub(crate) fn get_api_key(app: &tauri::AppHandle) -> Option<String> {
    api_key_path(app)
        .ok()
        .and_then(|path| read_api_key_file(&path))
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
    fs::rename(temporary, path).map_err(|error| format!("无法完成设置保存：{error}"))
}

#[tauri::command]
pub(crate) fn load_settings(app: tauri::AppHandle) -> Result<SettingsView, String> {
    Ok(SettingsView {
        settings: load_app_settings(&app)?,
        has_api_key: get_api_key(&app).is_some(),
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
            write_api_key_file(&api_key_path(&app)?, api_key)?;
        }
    }
    Ok(SettingsView {
        settings,
        has_api_key: get_api_key(&app).is_some(),
    })
}

#[tauri::command]
pub(crate) fn clear_ai_api_key(app: tauri::AppHandle) -> Result<(), String> {
    delete_api_key_file(&api_key_path(&app)?)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn stores_api_key_in_a_private_local_file() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("ai-api-key");
        write_api_key_file(&path, "  test-secret  ").expect("save API key");
        assert_eq!(read_api_key_file(&path).as_deref(), Some("test-secret"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        }
        delete_api_key_file(&path).expect("remove API key");
        assert!(!path.exists());
    }
}
