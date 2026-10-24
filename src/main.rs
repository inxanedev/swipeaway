#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod core;
mod preview;
mod safety;
mod theme;
mod ui;
mod util;

use std::path::PathBuf;

use app::SwipeAwayApp;

fn main() -> eframe::Result<()> {
    let initial = std::env::args_os().nth(1).map(PathBuf::from);

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("SwipeAway")
        .with_inner_size([1180.0, 820.0])
        .with_min_inner_size([960.0, 680.0]);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png")) {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "SwipeAway",
        options,
        Box::new(move |cc| Ok(Box::new(SwipeAwayApp::new(cc, initial)))),
    )
}
