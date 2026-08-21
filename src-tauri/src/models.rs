use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Note {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) content: String,
    #[serde(default)]
    pub(crate) knowledge_base_id: String,
    #[serde(default)]
    pub(crate) parent_id: Option<String>,
    #[serde(default)]
    pub(crate) deleted_at: Option<String>,
    #[serde(default)]
    pub(crate) pinned: bool,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KnowledgeBase {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) created_at: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NotesStore {
    #[serde(default)]
    pub(crate) knowledge_bases: Vec<KnowledgeBase>,
    #[serde(default)]
    pub(crate) notes: Vec<Note>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct GeneralSettings {
    pub(crate) auto_save_delay_ms: u64,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            auto_save_delay_ms: 450,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct AiSettings {
    pub(crate) enabled: bool,
    #[serde(default = "missing_ai_provider")]
    pub(crate) provider: String,
    pub(crate) base_url: String,
    pub(crate) protocol: String,
    pub(crate) model: String,
    #[serde(default)]
    pub(crate) models: Vec<String>,
    pub(crate) temperature: f32,
    pub(crate) max_context_chars: usize,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "openai".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            protocol: "chatCompletions".to_string(),
            model: "gpt-5.6".to_string(),
            models: vec!["gpt-5.6".to_string()],
            temperature: 0.3,
            max_context_chars: 30_000,
        }
    }
}

fn missing_ai_provider() -> String {
    String::new()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSettings {
    #[serde(default)]
    pub(crate) general: GeneralSettings,
    #[serde(default)]
    pub(crate) ai: AiSettings,
    #[serde(default)]
    pub(crate) document_directory: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SettingsView {
    pub(crate) settings: AppSettings,
    pub(crate) has_api_key: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportedMarkdown {
    pub(crate) title: String,
    pub(crate) content: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AiRequest {
    pub(crate) document_id: String,
    #[serde(default)]
    pub(crate) model: String,
    pub(crate) operation: String,
    #[serde(default)]
    pub(crate) prompt: String,
    #[serde(default)]
    pub(crate) selection: String,
    #[serde(default)]
    pub(crate) note_title: String,
    #[serde(default)]
    pub(crate) note_content: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub(crate) enum AiStreamEvent {
    Started,
    Delta { content: String },
    Done,
    Error { message: String },
}
