use iced::{
    executor,
    widget::{button, column, container, row, text, text_editor},
    Application, Command, Element, Length, Theme,
};
use std::path::PathBuf;

use crate::{
    config::Config,
    editor::EditorState,
    preferences::{PreferencesMessage, PreferencesState},
    search::{SearchMessage, SearchState},
    sidebar::{SidebarAction, SidebarMessage, SidebarState},
    theme as tincta_theme,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopMenu {
    File,
    Edit,
    View,
    Help,
}

#[derive(Debug, Clone)]
pub enum Message {
    EditorAction(text_editor::Action),
    Sidebar(SidebarMessage),
    Search(SearchMessage),
    Preferences(PreferencesMessage),
    OpenFile,
    FileOpened(Result<(PathBuf, String), String>),
    SaveFile,
    SaveFileAs,
    FileSaved(Result<PathBuf, String>),
    NewFile,
    CloseFile,
    ToggleSearch,
    ToggleSidebar,
    TogglePreferences,
    ThemeChanged(bool), // false = light, true = dark
    InsertTab,
    ToggleMenu(TopMenu),
    SelectAll,
    Quit,
    About,
    ContextCopy,
    ContextCut,
    ContextPaste,
    ContextDelete,
    ClipboardContent(Option<String>),
}

pub struct TinctaApp {
    config: Config,
    editor: EditorState,
    sidebar: SidebarState,
    search: SearchState,
    preferences: PreferencesState,
    show_search: bool,
    show_sidebar: bool,
    show_preferences: bool,
    open_menu: Option<TopMenu>,
    current_file: Option<PathBuf>,
    is_dirty: bool,
    status_message: String,
}

impl Application for TinctaApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let config = Config::load();
        let editor = EditorState::new();

        (
            Self {
                editor,
                sidebar: SidebarState::new(),
                search: SearchState::new(),
                preferences: PreferencesState::from_config(&config),
                show_search: false,
                show_sidebar: true,
                show_preferences: false,
                open_menu: None,
                current_file: None,
                is_dirty: false,
                status_message: t!("status.ready").to_string(),
                config,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        let untitled = t!("app.untitled").to_string();
        let file_name = self
            .current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or(untitled.as_str());
        let dirty = if self.is_dirty { " •" } else { "" };
        format!("Tincta — {}{}", file_name, dirty)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        // Any interaction except toggling a menu closes the open dropdown.
        if !matches!(&message, Message::ToggleMenu(_)) {
            self.open_menu = None;
        }

        match message {
            Message::EditorAction(action) => {
                let is_edit = action.is_edit();
                self.editor.content.perform(action);
                if is_edit {
                    self.is_dirty = true;
                }
                Command::none()
            }
            Message::InsertTab => {
                if !self.show_search && !self.show_preferences {
                    let text = if self.config.use_spaces {
                        " ".repeat(self.config.tab_width)
                    } else {
                        "\t".to_string()
                    };
                    self.editor
                        .content
                        .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                            std::sync::Arc::new(text),
                        )));
                    self.is_dirty = true;
                }
                Command::none()
            }
            Message::SelectAll => {
                self.editor
                    .content
                    .perform(text_editor::Action::Move(text_editor::Motion::DocumentStart));
                self.editor
                    .content
                    .perform(text_editor::Action::Select(text_editor::Motion::DocumentEnd));
                Command::none()
            }
            Message::ToggleMenu(menu) => {
                self.open_menu = if self.open_menu == Some(menu) {
                    None
                } else {
                    Some(menu)
                };
                Command::none()
            }
            Message::NewFile => {
                self.editor = EditorState::new();
                self.current_file = None;
                self.is_dirty = false;
                self.status_message = t!("status.new_file").to_string();
                Command::none()
            }
            Message::OpenFile => Command::perform(open_file(), Message::FileOpened),
            Message::FileOpened(result) => {
                match result {
                    Ok((path, content)) => {
                        self.editor = EditorState::from_content(&content);
                        let ext = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_string();
                        self.editor.set_language_by_extension(&ext);
                        self.sidebar.add_file(path.clone());
                        self.current_file = Some(path);
                        self.is_dirty = false;
                        self.status_message = t!("status.file_opened").to_string();
                    }
                    Err(e) => {
                        self.status_message = format!("{}: {}", t!("status.error"), e);
                    }
                }
                Command::none()
            }
            Message::SaveFile => {
                if let Some(ref path) = self.current_file.clone() {
                    let content = self.editor.content.text().to_string();
                    let path = path.clone();
                    Command::perform(save_file(path, content), Message::FileSaved)
                } else {
                    Command::perform(
                        save_file_as(self.editor.content.text().to_string()),
                        Message::FileSaved,
                    )
                }
            }
            Message::SaveFileAs => Command::perform(
                save_file_as(self.editor.content.text().to_string()),
                Message::FileSaved,
            ),
            Message::FileSaved(result) => {
                match result {
                    Ok(path) => {
                        self.sidebar.add_file(path.clone());
                        let ext = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_string();
                        self.editor.set_language_by_extension(&ext);
                        self.current_file = Some(path);
                        self.is_dirty = false;
                        self.status_message = t!("status.file_saved").to_string();
                    }
                    Err(e) => {
                        self.status_message = format!("{}: {}", t!("status.error"), e);
                    }
                }
                Command::none()
            }
            Message::CloseFile => {
                self.editor = EditorState::new();
                self.current_file = None;
                self.is_dirty = false;
                self.status_message = t!("status.ready").to_string();
                Command::none()
            }
            Message::ToggleSearch => {
                self.show_search = !self.show_search;
                Command::none()
            }
            Message::ToggleSidebar => {
                self.show_sidebar = !self.show_sidebar;
                Command::none()
            }
            Message::TogglePreferences => {
                self.show_preferences = !self.show_preferences;
                Command::none()
            }
            Message::ThemeChanged(dark) => {
                self.config.dark_mode = dark;
                self.config.save();
                Command::none()
            }
            Message::Sidebar(msg) => match self.sidebar.update(msg) {
                SidebarAction::OpenFile(path) => {
                    Command::perform(read_file(path), Message::FileOpened)
                }
                SidebarAction::CloseFile(path) => {
                    if self.current_file.as_ref() == Some(&path) {
                        self.editor = EditorState::new();
                        self.current_file = None;
                        self.is_dirty = false;
                    }
                    Command::none()
                }
                SidebarAction::SaveFile(path) => {
                    let content = self.editor.content.text().to_string();
                    Command::perform(save_file(path, content), Message::FileSaved)
                }
                SidebarAction::SaveFileAs => Command::perform(
                    save_file_as(self.editor.content.text().to_string()),
                    Message::FileSaved,
                ),
                SidebarAction::None => Command::none(),
            },
            Message::Search(msg) => {
                self.search.update(msg, &mut self.editor.content);
                Command::none()
            }
            Message::Preferences(msg) => {
                self.preferences.update(msg, &mut self.config);
                self.config.save();
                Command::none()
            }
            Message::About => {
                self.status_message = format!("Tincta v{}", env!("CARGO_PKG_VERSION"));
                Command::none()
            }
            Message::Quit => std::process::exit(0),
            Message::ContextCopy => {
                if let Some(text) = self.editor.content.selection() {
                    return iced::clipboard::write(text);
                }
                Command::none()
            }
            Message::ContextCut => {
                if let Some(text) = self.editor.content.selection() {
                    self.editor
                        .content
                        .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                    self.is_dirty = true;
                    return iced::clipboard::write(text);
                }
                Command::none()
            }
            Message::ContextPaste => {
                iced::clipboard::read(Message::ClipboardContent)
            }
            Message::ClipboardContent(Some(text)) => {
                self.editor
                    .content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Paste(
                        std::sync::Arc::new(text),
                    )));
                self.is_dirty = true;
                Command::none()
            }
            Message::ClipboardContent(None) => Command::none(),
            Message::ContextDelete => {
                self.editor
                    .content
                    .perform(text_editor::Action::Edit(text_editor::Edit::Delete));
                self.is_dirty = true;
                Command::none()
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::keyboard::on_key_press(|key, modifiers| {
            use iced::keyboard::{key::Named, Key, Modifiers};

            // Tab: insert tab/spaces (no modifier needed)
            if key == Key::Named(Named::Tab) {
                return Some(Message::InsertTab);
            }

            let cmd = modifiers.command(); // Ctrl on Linux/Windows, Cmd on macOS
            if !cmd {
                return None;
            }

            match key.as_ref() {
                Key::Character("a") => Some(Message::SelectAll),
                Key::Character("s") if modifiers.shift() => Some(Message::SaveFileAs),
                Key::Character("s") => Some(Message::SaveFile),
                Key::Character("n") => Some(Message::NewFile),
                Key::Character("o") => Some(Message::OpenFile),
                Key::Character("w") => Some(Message::CloseFile),
                Key::Character("f") => Some(Message::ToggleSearch),
                // Ctrl+C/X/V are handled internally by text_editor when focused;
                // use the right-click context menu when the editor is not focused.
                _ => None,
            }
        })
    }

    fn view(&self) -> Element<Message> {
        let dark = self.config.dark_mode;

        // Menu bar (Fichier / Édition / Affichage / Aide)
        let menu_bar = crate::menu_bar::view(&self.config, self.open_menu);

        // Editor with right-click context menu
        let editor_widget = self.editor.view(&self.config);
        // Pre-compute labels as owned Strings so they can be moved into the 'static closure.
        let lbl_select_all = t!("ctx.select_all").to_string();
        let lbl_cut = t!("ctx.cut").to_string();
        let lbl_copy = t!("ctx.copy").to_string();
        let lbl_paste = t!("ctx.paste").to_string();
        let lbl_delete = t!("ctx.delete").to_string();
        let editor_with_context = iced_aw::ContextMenu::new(editor_widget, move || {
            let item = |label: String, msg: Message| -> Element<'static, Message> {
                button(text(label).size(13))
                    .padding([6, 10])
                    .width(Length::Fixed(180.0))
                    .on_press(msg)
                    .style(iced::theme::Button::custom(crate::theme::GhostButton {
                        dark,
                        active: false,
                    }))
                    .into()
            };
            container(
                column![
                    item(lbl_select_all.clone(), Message::SelectAll),
                    item(lbl_cut.clone(), Message::ContextCut),
                    item(lbl_copy.clone(), Message::ContextCopy),
                    item(lbl_paste.clone(), Message::ContextPaste),
                    item(lbl_delete.clone(), Message::ContextDelete),
                ]
                .spacing(2)
                .padding(6),
            )
            .style(crate::theme::card(dark))
            .into()
        });

        // Search panel (conditionally shown)
        let search_panel = if self.show_search {
            Some(self.search.view(dark))
        } else {
            None
        };

        // Main content area
        let editor_area: Element<Message> = if let Some(search) = search_panel {
            column![search, editor_with_context].into()
        } else {
            editor_with_context.into()
        };

        // Sidebar (conditionally shown)
        let mut main_row = row![];
        if self.show_sidebar {
            main_row = main_row.push(self.sidebar.view(dark, &self.current_file));
        }
        main_row = main_row.push(editor_area);
        if self.show_preferences {
            main_row = main_row.push(self.preferences.view(dark));
        }

        // Status bar
        let status_bar = crate::editor::statusbar::view(
            &self.status_message,
            &self.current_file,
            self.is_dirty,
            dark,
        );

        let content = column![menu_bar, main_row, status_bar,]
            .width(Length::Fill)
            .height(Length::Fill);

        let base: Element<Message> = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        // Float the dropdown overlay on top — doesn't shift the layout.
        if let Some(menu) = self.open_menu {
            let dropdown = crate::menu_bar::dropdown_view(menu, &self.config);
            let x = crate::menu_bar::dropdown_x_offset(menu);
            iced_aw::floating_element::FloatingElement::new(base, dropdown)
                .anchor(iced_aw::floating_element::Anchor::NorthWest)
                .offset(iced_aw::floating_element::Offset { x, y: crate::menu_bar::BAR_HEIGHT })
                .into()
        } else {
            base
        }
    }

    fn theme(&self) -> Theme {
        if self.config.dark_mode {
            tincta_theme::ink_dark()
        } else {
            tincta_theme::ink_light()
        }
    }
}

async fn open_file() -> Result<(PathBuf, String), String> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Open File")
        .add_filter(
            "All supported files",
            &[
                "txt", "md", "markdown", "rst",
                "rs", "toml", "lock",
                "py", "pyw",
                "js", "jsx", "mjs", "cjs",
                "ts", "tsx",
                "html", "htm", "xhtml",
                "css", "scss", "sass", "less",
                "json", "json5", "jsonc",
                "yaml", "yml",
                "xml", "svg", "plist",
                "sh", "bash", "zsh", "fish", "ps1",
                "c", "h", "cpp", "cc", "cxx", "hpp", "hh",
                "java", "go", "rb", "php", "swift", "kt", "kts",
                "sql", "lua", "r", "m", "vb", "cs",
                "makefile", "dockerfile", "gitignore", "env",
                "conf", "cfg", "ini", "properties",
                "log",
            ],
        )
        .add_filter("Text files", &["txt", "md", "rst", "log"])
        .add_filter("Source code", &[
            "rs", "py", "js", "ts", "jsx", "tsx", "c", "h", "cpp",
            "java", "go", "rb", "php", "swift", "kt", "lua", "sql",
        ])
        .add_filter("Web files", &["html", "htm", "css", "scss", "json", "xml", "svg"])
        .add_filter("Config files", &["toml", "yaml", "yml", "ini", "cfg", "conf", "env"])
        .add_filter("All files", &["*"])
        .pick_file()
        .await
        .ok_or_else(|| "cancelled".to_string())?;

    let path = handle.path().to_path_buf();
    let content =
        std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    Ok((path, content))
}

async fn read_file(path: PathBuf) -> Result<(PathBuf, String), String> {
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    Ok((path, content))
}

async fn save_file(path: PathBuf, content: String) -> Result<PathBuf, String> {
    std::fs::write(&path, &content).map_err(|e| e.to_string())?;
    Ok(path)
}

async fn save_file_as(content: String) -> Result<PathBuf, String> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Save File As")
        .save_file()
        .await
        .ok_or_else(|| "cancelled".to_string())?;

    let path = handle.path().to_path_buf();
    std::fs::write(&path, &content).map_err(|e| e.to_string())?;
    Ok(path)
}
