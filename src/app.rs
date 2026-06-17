use iced::{
    executor,
    widget::{button, column, container, row, text, text_editor},
    Application, Command, Element, Length, Theme,
};
use std::collections::HashMap;
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
    FormatFile,
    FormatSelection,
    FileFormatted(Result<String, String>),
    SelectionFormatted(Result<String, String>),
    CloseErrorPanel,
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
    is_formatting: bool,
    status_message: String,
    status_is_error: bool,
    format_error: Option<String>,
    show_error_panel: bool,
    untitled_counter: u32,
    // In-memory cache of unsaved file states: path → (content, language, is_dirty)
    file_cache: HashMap<PathBuf, (String, Option<String>, bool)>,
}

impl Application for TinctaApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let config = Config::load();
        let editor = EditorState::new();
        // Start with one untitled file so the editor always has an identity
        let initial_path = untitled_path(1);
        let mut sidebar = SidebarState::new();
        sidebar.add_file(initial_path.clone());

        (
            Self {
                editor,
                sidebar,
                search: SearchState::new(),
                preferences: PreferencesState::from_config(&config),
                show_search: false,
                show_sidebar: true,
                show_preferences: false,
                open_menu: None,
                current_file: Some(initial_path),
                is_dirty: false,
                is_formatting: false,
                status_message: t!("status.ready").to_string(),
                status_is_error: false,
                format_error: None,
                show_error_panel: false,
                untitled_counter: 1,
                file_cache: HashMap::new(),
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
                // Save current state before switching
                if let Some(current) = self.current_file.clone() {
                    self.file_cache.insert(
                        current,
                        (self.editor.content.text().to_string(), self.editor.language.clone(), self.is_dirty),
                    );
                }
                self.untitled_counter += 1;
                let path = untitled_path(self.untitled_counter);
                self.editor = EditorState::new();
                self.current_file = Some(path.clone());
                self.sidebar.add_file(path);
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
                let content = self.editor.content.text().to_string();
                match self.current_file.clone() {
                    Some(path) if !is_untitled(&path) => {
                        Command::perform(save_file(path, content), Message::FileSaved)
                    }
                    _ => Command::perform(save_file_as(content), Message::FileSaved),
                }
            }
            Message::SaveFileAs => Command::perform(
                save_file_as(self.editor.content.text().to_string()),
                Message::FileSaved,
            ),
            Message::FileSaved(result) => {
                match result {
                    Ok(path) => {
                        self.file_cache.remove(&path);
                        // If saving an untitled file for the first time, replace its slot
                        if let Some(old) = &self.current_file {
                            if is_untitled(old) {
                                self.file_cache.remove(old);
                                self.sidebar.rename_file(old, path.clone());
                            }
                        }
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
                    if self.current_file.as_ref() == Some(&path) {
                        return Command::none();
                    }
                    // Save current editor state before switching
                    if let Some(current) = self.current_file.clone() {
                        self.file_cache.insert(
                            current,
                            (
                                self.editor.content.text().to_string(),
                                self.editor.language.clone(),
                                self.is_dirty,
                            ),
                        );
                    }
                    // Restore from cache or load from disk
                    if let Some((content, language, dirty)) = self.file_cache.get(&path).cloned() {
                        self.editor = EditorState::from_content(&content);
                        self.editor.language = language;
                        self.current_file = Some(path);
                        self.is_dirty = dirty;
                        self.status_message = t!("status.file_opened").to_string();
                        Command::none()
                    } else {
                        Command::perform(read_file(path), Message::FileOpened)
                    }
                }
                SidebarAction::CloseFile(path) => {
                    if self.current_file.as_ref() == Some(&path) {
                        self.file_cache.remove(&path);
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
            Message::FormatFile => {
                if let Some(ext) = self.editor.language.clone() {
                    self.is_formatting = true;
                    self.status_message = t!("status.formatting").to_string();
                    let content = self.editor.content.text().to_string();
                    Command::perform(crate::formatter::format(content, ext), Message::FileFormatted)
                } else {
                    self.status_message = t!("status.no_language").to_string();
                    Command::none()
                }
            }
            Message::FormatSelection => {
                if let (Some(ext), Some(selected)) = (
                    self.editor.language.clone(),
                    self.editor.content.selection(),
                ) {
                    self.is_formatting = true;
                    self.status_message = t!("status.formatting").to_string();
                    Command::perform(
                        crate::formatter::format(selected, ext),
                        Message::SelectionFormatted,
                    )
                } else {
                    Command::none()
                }
            }
            Message::FileFormatted(result) => {
                self.is_formatting = false;
                match result {
                    Ok(formatted) => {
                        self.editor.content = text_editor::Content::with_text(&formatted);
                        self.is_dirty = true;
                        self.status_is_error = false;
                        self.format_error = None;
                        self.status_message = t!("status.formatted").to_string();
                    }
                    Err(e) => {
                        self.status_is_error = true;
                        self.format_error = Some(e.clone());
                        self.show_error_panel = true;
                        self.status_message = t!("status.format_error").to_string();
                    }
                }
                Command::none()
            }
            Message::SelectionFormatted(result) => {
                self.is_formatting = false;
                match result {
                    Ok(formatted) => {
                        self.editor.content.perform(text_editor::Action::Edit(
                            text_editor::Edit::Paste(std::sync::Arc::new(formatted)),
                        ));
                        self.is_dirty = true;
                        self.status_is_error = false;
                        self.format_error = None;
                        self.status_message = t!("status.formatted").to_string();
                    }
                    Err(e) => {
                        self.status_is_error = true;
                        self.format_error = Some(e.clone());
                        self.show_error_panel = true;
                        self.status_message = t!("status.format_error").to_string();
                    }
                }
                Command::none()
            }
            Message::CloseErrorPanel => {
                self.show_error_panel = false;
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
        let lbl_format_sel = t!("ctx.format_selection").to_string();
        let lbl_format_all = t!("ctx.format_all").to_string();
        let has_fmt_ctx = self.editor.language.as_deref()
            .map(|ext| crate::formatter::has_formatter(ext))
            .unwrap_or(false);
        let has_sel_ctx = self.editor.content.selection().is_some();
        let editor_with_context = iced_aw::ContextMenu::new(editor_widget, move || {
            let item = |label: String, msg: Message| -> Element<'static, Message> {
                button(text(label).size(13))
                    .padding([6, 10])
                    .width(Length::Fixed(200.0))
                    .on_press(msg)
                    .style(iced::theme::Button::custom(crate::theme::GhostButton {
                        dark,
                        active: false,
                    }))
                    .into()
            };
            let disabled = |label: String| -> Element<'static, Message> {
                container(text(label).size(13).style(crate::theme::muted_text(dark)))
                    .padding([6, 10])
                    .width(Length::Fixed(200.0))
                    .into()
            };
            let fmt_sel_el: Element<'static, Message> = if has_fmt_ctx && has_sel_ctx {
                item(lbl_format_sel.clone(), Message::FormatSelection)
            } else {
                disabled(lbl_format_sel.clone())
            };
            let fmt_all_el: Element<'static, Message> = if has_fmt_ctx {
                item(lbl_format_all.clone(), Message::FormatFile)
            } else {
                disabled(lbl_format_all.clone())
            };
            let ctx_items: Vec<Element<'static, Message>> = vec![
                item(lbl_select_all.clone(), Message::SelectAll),
                item(lbl_cut.clone(), Message::ContextCut),
                item(lbl_copy.clone(), Message::ContextCopy),
                item(lbl_paste.clone(), Message::ContextPaste),
                item(lbl_delete.clone(), Message::ContextDelete),
                fmt_sel_el,
                fmt_all_el,
            ];
            container(
                column(ctx_items).spacing(2).padding(6),
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
        let cursor = self.editor.content.cursor_position();
        let status_bar = crate::editor::statusbar::view(
            &self.status_message,
            &self.current_file,
            self.is_dirty,
            dark,
            cursor,
            self.status_is_error,
        );

        // Error panel (shown when formatter reports an error)
        let error_panel: Option<Element<Message>> = if self.show_error_panel {
            if let Some(err) = &self.format_error {
                let header = row![
                    text(t!("panel.errors").to_string())
                        .size(11)
                        .style(tincta_theme::muted_text(dark)),
                    iced::widget::Space::with_width(Length::Fill),
                    button(text("✕").size(11).style(tincta_theme::muted_text(dark)))
                        .padding([2, 6])
                        .on_press(Message::CloseErrorPanel)
                        .style(iced::theme::Button::custom(tincta_theme::GhostButton {
                            dark,
                            active: false,
                        })),
                ]
                .padding([4, 10])
                .align_items(iced::Alignment::Center);

                let error_color = iced::Color::from_rgb(0.88, 0.27, 0.18);
                let body = iced::widget::scrollable(
                    container(
                        text(err.clone())
                            .size(12)
                            .style(error_color)
                            .font(iced::Font::MONOSPACE),
                    )
                    .padding([4, 12, 8, 12])
                    .width(Length::Fill),
                )
                .height(Length::Fixed(100.0));

                Some(
                    container(column![header, body])
                        .width(Length::Fill)
                        .style(tincta_theme::error_panel(dark))
                        .into(),
                )
            } else {
                None
            }
        } else {
            None
        };

        // Formatting banner shown above the editor while an async format is running
        let mut col = if self.is_formatting {
            let banner = container(
                text(t!("status.formatting").to_string())
                    .size(12)
                    .style(tincta_theme::accent_color()),
            )
            .width(Length::Fill)
            .padding([4, 14])
            .style(tincta_theme::accent_banner(dark));
            column![menu_bar, banner, main_row]
        } else {
            column![menu_bar, main_row]
        };
        if let Some(panel) = error_panel {
            col = col.push(panel);
        }
        let content = col
            .push(status_bar)
            .width(Length::Fill)
            .height(Length::Fill);

        let base: Element<Message> = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        // Float the dropdown overlay on top — doesn't shift the layout.
        if let Some(menu) = self.open_menu {
            let has_fmt = self.editor.language.as_deref()
                .map(|ext| crate::formatter::has_formatter(ext))
                .unwrap_or(false);
            let dropdown = crate::menu_bar::dropdown_view(menu, &self.config, has_fmt);
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

pub fn untitled_path(n: u32) -> PathBuf {
    PathBuf::from(format!("untitled://{}", n))
}

pub fn is_untitled(path: &PathBuf) -> bool {
    path.to_str().map(|s| s.starts_with("untitled://")).unwrap_or(false)
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
