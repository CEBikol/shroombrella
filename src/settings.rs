use crate::theme::ThemeVisuals;
use dirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub current_theme: String,
    pub ui_scale: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            current_theme: "Dark".to_string(),
            ui_scale: 1.0,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        if let Some(config_path) = Self::get_settings_path() {
            if config_path.exists() {
                if let Ok(contents) = std::fs::read_to_string(&config_path) {
                    if let Ok(settings) = serde_json::from_str(&contents) {
                        return settings;
                    }
                }
            } else {
                let default_settings = Self::default();
                if default_settings.save().is_ok() {
                    println!("Создан файл настроек: {:?}", config_path);
                }
                return default_settings;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path =
            Self::get_settings_path().ok_or("Не удалось определить путь к настройкам")?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|_| "Не удалось создать директорию настроек")?;
        }

        let contents =
            serde_json::to_string_pretty(self).map_err(|_| "Ошибка сериализации настроек")?;

        std::fs::write(&config_path, contents).map_err(|_| "Ошибка записи файла настроек")?;

        Ok(())
    }

    fn get_settings_path() -> Option<PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("shroombrella");
            Some(app_dir.join("settings.json"))
        } else {
            None
        }
    }

    // Получаем список доступных тем
    pub fn get_available_themes(&self) -> Vec<String> {
        let mut themes = vec!["Dark".to_string(), "Light".to_string()];

        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("shroombrella");
            let themes_dir = app_dir.join("themes");

            if themes_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&themes_dir) {
                    for entry in entries.filter_map(Result::ok) {
                        let path = entry.path();
                        if path.extension().map_or(false, |ext| ext == "json") {
                            if let Some(filename) = path.file_stem() {
                                let theme_name = filename.to_string_lossy().to_string();
                                if !themes.contains(&theme_name) {
                                    themes.push(theme_name);
                                }
                            }
                        }
                    }
                }
            }
        }

        themes
    }

    // Загружаем тему
    pub fn load_theme(&self) -> Result<ThemeVisuals, String> {
        match self.current_theme.as_str() {
            "Dark" => Ok(ThemeVisuals::default_dark()),
            "Light" => Ok(ThemeVisuals::default_light()),
            _ => {
                // Пытаемся загрузить кастомную тему
                self.load_custom_theme(&self.current_theme)
            }
        }
    }

    // Загружаем кастомную тему из файла
    fn load_custom_theme(&self, theme_name: &str) -> Result<ThemeVisuals, String> {
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("shroombrella");
            let themes_dir = app_dir.join("themes");
            let theme_path = themes_dir.join(format!("{}.json", theme_name));

            if theme_path.exists() {
                let contents = std::fs::read_to_string(&theme_path)
                    .map_err(|e| format!("Ошибка чтения файла темы: {}", e))?;
                let theme: ThemeVisuals = serde_json::from_str(&contents)
                    .map_err(|e| format!("Ошибка парсинга темы: {}", e))?;
                Ok(theme)
            } else {
                // Если кастомная тема не найдена, возвращаем темную по умолчанию
                Ok(ThemeVisuals::default_dark())
            }
        } else {
            Ok(ThemeVisuals::default_dark())
        }
    }
}
