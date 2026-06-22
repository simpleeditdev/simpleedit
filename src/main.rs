#[macro_use]
extern crate rust_i18n;

use iced::{window, Application, Settings, Size};

fn load_icon() -> Option<window::Icon> {
    let bytes = include_bytes!("assets/icon.png");
    let img = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    window::icon::from_rgba(img.into_raw(), w, h).ok()
}

mod app;
mod config;
mod editor;
mod formatter;
mod i18n;
mod menu_bar;
mod preferences;
mod search;
mod session;
mod sidebar;
mod theme;

i18n!("src/i18n");

fn main() -> iced::Result {
    let locale = if std::env::var("LANG").unwrap_or_default().starts_with("fr") {
        "fr"
    } else {
        "en"
    };
    rust_i18n::set_locale(locale);

    // Optional file path passed as first CLI argument: `tincta path/to/file`
    let file_arg = std::env::args()
        .nth(1)
        .filter(|a| !a.starts_with('-'))
        .map(std::path::PathBuf::from);

    app::TinctaApp::run(Settings {
        flags: file_arg,
        window: window::Settings {
            size: Size::new(1200.0, 800.0),
            min_size: Some(Size::new(600.0, 400.0)),
            resizable: true,
            icon: load_icon(),
            ..Default::default()
        },
        ..Default::default()
    })
}
