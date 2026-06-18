use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Identifies which action is being rebound in the shortcuts config panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutTarget {
    NewFile,
    OpenFile,
    SaveFile,
    SaveAs,
    CloseFile,
    Find,
    SelectAll,
    FormatCode,
    GotoLine,
    ToggleSidebar,
    Quit,
    Undo,
    Redo,
}

/// A single keyboard binding stored as display-friendly strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    /// Single character ("n", "s") or named key ("Return", "F1").
    pub key: String,
}

impl ShortcutConfig {
    fn ctrl(key: &str) -> Self {
        Self { ctrl: true, shift: false, alt: false, key: key.to_string() }
    }
    fn ctrl_shift(key: &str) -> Self {
        Self { ctrl: true, shift: true, alt: false, key: key.to_string() }
    }

    pub fn display(&self) -> String {
        let mut s = String::new();
        if self.ctrl { s.push_str("Ctrl+"); }
        if self.shift { s.push_str("Shift+"); }
        if self.alt { s.push_str("Alt+"); }
        if self.key.len() == 1 {
            s.push_str(&self.key.to_uppercase());
        } else {
            s.push_str(&self.key);
        }
        s
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shortcuts {
    pub new_file: ShortcutConfig,
    pub open_file: ShortcutConfig,
    pub save_file: ShortcutConfig,
    pub save_as: ShortcutConfig,
    pub close_file: ShortcutConfig,
    pub find: ShortcutConfig,
    pub select_all: ShortcutConfig,
    pub format_code: ShortcutConfig,
    pub goto_line: ShortcutConfig,
    pub toggle_sidebar: ShortcutConfig,
    pub quit: ShortcutConfig,
    pub undo: ShortcutConfig,
    pub redo: ShortcutConfig,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            new_file: ShortcutConfig::ctrl("n"),
            open_file: ShortcutConfig::ctrl("o"),
            save_file: ShortcutConfig::ctrl("s"),
            save_as: ShortcutConfig::ctrl_shift("s"),
            close_file: ShortcutConfig::ctrl("w"),
            find: ShortcutConfig::ctrl("f"),
            select_all: ShortcutConfig::ctrl("a"),
            format_code: ShortcutConfig::ctrl_shift("f"),
            goto_line: ShortcutConfig::ctrl("g"),
            toggle_sidebar: ShortcutConfig::ctrl("b"),
            quit: ShortcutConfig::ctrl("q"),
            undo: ShortcutConfig::ctrl("z"),
            redo: ShortcutConfig::ctrl("y"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub dark_mode: bool,
    pub font_size: f32,
    pub font_family: String,
    pub tab_width: usize,
    pub use_spaces: bool,
    pub show_line_numbers: bool,
    pub word_wrap: bool,
    pub auto_indent: bool,
    pub autocomplete_brackets: bool,
    pub autocomplete_quotes: bool,
    pub show_page_guide: bool,
    pub page_guide_column: usize,
    pub highlight_current_line: bool,
    pub locale: String,
    #[serde(default)]
    pub shortcuts: Shortcuts,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dark_mode: false,
            font_size: 14.0,
            font_family: "Monospace".to_string(),
            tab_width: 4,
            use_spaces: true,
            show_line_numbers: true,
            word_wrap: true,
            auto_indent: true,
            autocomplete_brackets: true,
            autocomplete_quotes: true,
            show_page_guide: false,
            page_guide_column: 80,
            highlight_current_line: true,
            locale: "en".to_string(),
            shortcuts: Shortcuts::default(),
        }
    }
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("tincta").join("config.json"))
    }

    pub fn load() -> Self {
        Self::config_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(path, json);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = Config::default();
        assert_eq!(config.tab_width, 4);
        assert!(config.use_spaces);
        assert_eq!(config.font_size, 14.0);
        assert_eq!(config.locale, "en");
    }

    #[test]
    fn config_serialization_roundtrip() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let restored: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config.tab_width, restored.tab_width);
        assert_eq!(config.dark_mode, restored.dark_mode);
        assert_eq!(config.font_size, restored.font_size);
    }

    #[test]
    fn shortcut_display() {
        let sc = ShortcutConfig::ctrl("n");
        assert_eq!(sc.display(), "Ctrl+N");
        let sc2 = ShortcutConfig::ctrl_shift("s");
        assert_eq!(sc2.display(), "Ctrl+Shift+S");
    }
}
