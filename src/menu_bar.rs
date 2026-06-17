use iced::{
    widget::{button, column, container, row, text, Space},
    Element, Length,
};

use crate::app::{Message, TopMenu};
use crate::config::Config;
use crate::preferences::PreferencesMessage;
use crate::theme;

/// Just the menu bar row — no dropdown, no layout side effects.
pub fn view(config: &Config, open: Option<TopMenu>) -> Element<'static, Message> {
    let dark = config.dark_mode;

    let top_button = |label: String, menu: TopMenu| {
        button(text(label).size(13))
            .padding([6, 12])
            .on_press(Message::ToggleMenu(menu))
            .style(iced::theme::Button::custom(theme::GhostButton {
                dark,
                active: open == Some(menu),
            }))
    };

    let bar = row![
        top_button(t!("menu.file").to_string(), TopMenu::File),
        top_button(t!("menu.edit").to_string(), TopMenu::Edit),
        top_button(t!("menu.view").to_string(), TopMenu::View),
        top_button(t!("menu.help").to_string(), TopMenu::Help),
        Space::with_width(Length::Fill),
        text(t!("app.title").to_string())
            .size(12)
            .style(theme::muted_text(dark)),
    ]
    .spacing(2)
    .padding([4, 10]);

    container(bar).width(Length::Fill).style(theme::bar(dark)).into()
}

/// The floating dropdown panel for the active menu. Call only when a menu is open.
pub fn dropdown_view(menu: TopMenu, config: &Config, has_formatter: bool) -> Element<'static, Message> {
    let items = menu_items(menu, config, has_formatter);
    container(column(items).spacing(2).padding(6).width(220))
        .style(theme::card(config.dark_mode))
        .into()
}

/// Approximate x position (px from window left) of each menu button.
/// Used to anchor the floating dropdown below the right button.
/// Bar left padding = 10. Button padding = [6, 12]. Size-13 font.
pub fn dropdown_x_offset(menu: TopMenu) -> f32 {
    match menu {
        TopMenu::File => 10.0,
        TopMenu::Edit => 90.0,
        TopMenu::View => 170.0,
        TopMenu::Help => 264.0,
    }
}

/// Approximate height of the menu bar row (px). Used as the y offset for the dropdown.
pub const BAR_HEIGHT: f32 = 38.0;

fn item(label: String, dark: bool, message: Message) -> Element<'static, Message> {
    button(text(label).size(13))
        .padding([6, 10])
        .width(Length::Fill)
        .on_press(message)
        .style(iced::theme::Button::custom(theme::GhostButton {
            dark,
            active: false,
        }))
        .into()
}

fn disabled_item(label: String, dark: bool) -> Element<'static, Message> {
    container(
        text(label).size(13).style(crate::theme::muted_text(dark))
    )
    .padding([6, 10])
    .width(Length::Fill)
    .into()
}

fn menu_items(menu: TopMenu, config: &Config, has_formatter: bool) -> Vec<Element<'static, Message>> {
    let dark = config.dark_mode;
    match menu {
        TopMenu::File => vec![
            item(t!("menu.file_new").to_string(), dark, Message::NewFile),
            item(t!("menu.file_open").to_string(), dark, Message::OpenFile),
            item(t!("menu.file_save").to_string(), dark, Message::SaveFile),
            item(
                t!("menu.file_save_as").to_string(),
                dark,
                Message::SaveFileAs,
            ),
            item(
                t!("menu.file_close").to_string(),
                dark,
                Message::CloseFile,
            ),
            item(t!("menu.file_quit").to_string(), dark, Message::Quit),
        ],
        TopMenu::Edit => {
            let format_item: Element<'static, Message> = if has_formatter {
                item(t!("menu.edit_format").to_string(), dark, Message::FormatFile)
            } else {
                disabled_item(t!("menu.edit_format").to_string(), dark)
            };
            vec![
                item(
                    t!("menu.edit_select_all").to_string(),
                    dark,
                    Message::SelectAll,
                ),
                item(
                    t!("menu.edit_find").to_string(),
                    dark,
                    Message::ToggleSearch,
                ),
                format_item,
                item(
                    t!("menu.edit_preferences").to_string(),
                    dark,
                    Message::TogglePreferences,
                ),
            ]
        }
        TopMenu::View => vec![
            item(
                t!("menu.view_sidebar").to_string(),
                dark,
                Message::ToggleSidebar,
            ),
            item(
                if dark {
                    t!("toolbar.light").to_string()
                } else {
                    t!("toolbar.dark").to_string()
                },
                dark,
                Message::ThemeChanged(!dark),
            ),
            item(
                if config.show_line_numbers {
                    t!("menu.view_line_numbers_hide").to_string()
                } else {
                    t!("menu.view_line_numbers_show").to_string()
                },
                dark,
                Message::Preferences(PreferencesMessage::ShowLineNumbersToggled(!config.show_line_numbers)),
            ),
        ],
        TopMenu::Help => vec![item(
            t!("menu.help_about").to_string(),
            dark,
            Message::About,
        )],
    }
}
