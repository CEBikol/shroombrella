#![windows_subsystem = "windows"]
mod app;
mod crypto;
mod settings;
mod storage;
mod theme;
mod ui;
mod vault;

use eframe::egui::ViewportBuilder;
use image::ImageReader;

fn load_icon() -> eframe::egui::IconData {
    let icon_bytes = include_bytes!("../assets/shroombrella.png");
    let cursor = std::io::Cursor::new(icon_bytes);
    let image = ImageReader::new(cursor)
        .with_guessed_format()
        .expect("Failed to guess image format")
        .decode()
        .expect("Failed to decode image");

    let image_buffer = image.into_rgba8();
    let (width, height) = image_buffer.dimensions();

    eframe::egui::IconData {
        rgba: image_buffer.into_raw(),
        width,
        height,
    }
}

fn main() -> Result<(), eframe::Error> {
    let viewport: ViewportBuilder = ViewportBuilder::default()
        .with_icon(load_icon())
        .with_decorations(false);

    let mut options = eframe::NativeOptions::default();
    options.viewport = viewport;

    eframe::run_native(
        "ShroomBrella",
        options,
        Box::new(|cc| {
            // Загружаем тему из настроек
            let settings = crate::settings::Settings::load();
            if let Ok(theme) = settings.load_theme() {
                cc.egui_ctx.set_visuals(theme.to_egui_visuals());
            }
            Ok(Box::new(app::PasswordApp::default()))
        }),
    )
}
