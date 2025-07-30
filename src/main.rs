mod app;
mod crypto;
mod settings;
mod storage;
mod theme;
mod ui;
mod vault;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        ..Default::default()
    };

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
