use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub sitting_duration: u32,
    pub standing_duration: u32,
    pub transition_duration: u32,
    pub work_schedules: Vec<String>,
    pub idle_threshold: u32,
    pub overlay_alpha: f32,
    pub strict_mode: bool,
    pub auto_start: bool,
    pub sit_msg: String,
    pub stand_msg: String,
    pub sound_path: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            sitting_duration: 25,
            standing_duration: 5,
            transition_duration: 30,
            work_schedules: vec!["08:00-12:00".to_string(), "13:00-17:00".to_string()],
            idle_threshold: 5,
            overlay_alpha: 0.8,
            strict_mode: true,
            auto_start: false,
            sit_msg: "Prepare to Sit Down".to_string(),
            stand_msg: "Prepare to Stand Up".to_string(),
            sound_path: "".to_string(),
        }
    }
}

impl AppSettings {
    fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
        path.push("breaker-pro");
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        path.push("settings.json");
        path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                    // Cập nhật đường dẫn âm thanh nếu cần, hoặc validate
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }
}
