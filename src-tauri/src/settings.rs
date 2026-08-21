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
        if !matches!(settings.protocol.as_str(), "chatCompletions" | "responses") {
            return Err("不支持的 AI 接口协议".to_string());
        }
    }
    if !(0.0..=2.0).contains(&settings.temperature) {
        return Err("温度参数必须在 0 到 2 之间".to_string());
    }
    if !(2_000..=200_000).contains(&settings.max_context_chars) {
        return Err("上下文长度必须在 2,000 到 200,000 字符之间".to_string());
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
