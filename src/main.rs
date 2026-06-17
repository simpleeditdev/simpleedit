#[macro_use]
extern crate rust_i18n;

use iced::{Application, Settings, Size, window};

fn load_icon() -> Option<window::Icon> {
    let bytes = include_bytes!("assets/icon.png");
    let img = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    window::icon::from_rgba(img.into_raw(), w, h).ok()
}

mod app;
mod config;
mod editor;
mod i18n;
mod menu_bar;
mod preferences;
mod search;
mod sidebar;
mod theme;
mod formatter;

i18n!("src/i18n");

fn main() -> iced::Result {
    let locale = std::env::var("LANG")
        .unwrap_or_default()
        .starts_with("fr")
        .then_some("fr")
        .unwrap_or("en");
    rust_i18n::set_locale(locale);

    app::TinctaApp::run(Settings {
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
