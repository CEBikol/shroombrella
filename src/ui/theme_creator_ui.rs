use crate::theme::ThemeVisuals;
use dirs;
use eframe::egui;

// Вспомогательная функция для выбора цвета RGB (вне impl)
fn color_picker_rgb_ui(ui: &mut egui::Ui, color: &mut [u8; 3]) {
    ui.horizontal(|ui| {
        ui.add(
            egui::DragValue::new(&mut color[0])
                .range(0..=255)
                .prefix("R:"),
        );
        ui.add(
            egui::DragValue::new(&mut color[1])
                .range(0..=255)
                .prefix("G:"),
        );
        ui.add(
            egui::DragValue::new(&mut color[2])
                .range(0..=255)
                .prefix("B:"),
        );
    });
}

// Вспомогательная функция для выбора цвета с опциональным значением (вне impl)
fn color_picker_ui(ui: &mut egui::Ui, color: &mut Option<[u8; 3]>) {
    let mut use_color = color.is_some();
    ui.checkbox(&mut use_color, "Использовать");

    if use_color {
        let mut rgb = color.unwrap_or([255, 255, 255]);
        color_picker_rgb_ui(ui, &mut rgb);
        *color = Some(rgb);
    } else {
        *color = None;
    }
}

pub struct ThemeCreator {
    pub show: bool,
    pub theme_name: String,
    pub theme_data: ThemeVisuals,
    pub error_message: String,
    pub success_message: String,
}

impl ThemeCreator {
    pub fn new() -> Self {
        Self {
            show: false,
            theme_name: String::new(),
            theme_data: ThemeVisuals::default_dark(),
            error_message: String::new(),
            success_message: String::new(),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let mut show = self.show;
        egui::Window::new("➕ Создать новую тему")
            .open(&mut show)
            .resizable(true)
            .default_width(500.0)
            .default_height(600.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });
        self.show = show;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Создание новой темы");
        ui.separator();

        // Показываем сообщения
        if !self.error_message.is_empty() {
            ui.colored_label(egui::Color32::RED, &self.error_message);
            ui.separator();
        }

        if !self.success_message.is_empty() {
            ui.colored_label(egui::Color32::GREEN, &self.success_message);
            ui.separator();
        }

        // Название темы
        ui.label("Название темы:");
        ui.text_edit_singleline(&mut self.theme_name);

        ui.separator();

        // Основные настройки темы
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.theme_settings_ui(ui);
        });

        ui.separator();

        // Кнопки
        ui.horizontal(|ui| {
            if ui.button("✅ Создать").clicked() {
                self.create_theme();
            }

            if ui.button("❌ Отмена").clicked() {
                self.show = false;
                self.clear_messages();
            }
        });
    }

    fn theme_settings_ui(&mut self, ui: &mut egui::Ui) {
        // Dark mode
        ui.checkbox(&mut self.theme_data.dark_mode, "Темная тема");

        ui.separator();

        // Цвета
        ui.label("🎨 Цвета:");

        // Override text color
        ui.label("Цвет текста (RGB):");
        color_picker_ui(ui, &mut self.theme_data.override_text_color);

        // Hyperlink color
        ui.label("Цвет ссылок (RGB):");
        color_picker_rgb_ui(ui, &mut self.theme_data.hyperlink_color);

        // Faint background color
        ui.label("Слабый фон (RGB):");
        color_picker_rgb_ui(ui, &mut self.theme_data.faint_bg_color);

        // Extreme background color
        ui.label("Экстремальный фон (RGB):");
        color_picker_rgb_ui(ui, &mut self.theme_data.extreme_bg_color);

        // Code background color
        ui.label("Фон кода (RGB):");
        color_picker_rgb_ui(ui, &mut self.theme_data.code_bg_color);

        // Warning color
        ui.label("Цвет предупреждений (RGB):");
        color_picker_rgb_ui(ui, &mut self.theme_data.warn_fg_color);

        // Error color
        ui.label("Цвет ошибок (RGB):");
        color_picker_rgb_ui(ui, &mut self.theme_data.error_fg_color);

        // Window fill
        ui.label("Фон окон (RGB):");
        color_picker_rgb_ui(ui, &mut self.theme_data.window_fill);

        // Panel fill
        ui.label("Фон панелей (RGB):");
        color_picker_rgb_ui(ui, &mut self.theme_data.panel_fill);

        ui.separator();

        // Window stroke
        ui.label("🎨 Обводка окон:");
        ui.label("Цвет обводки (RGB):");
        color_picker_rgb_ui(ui, &mut self.theme_data.window_stroke_color);

        ui.label("Ширина обводки:");
        ui.add(egui::DragValue::new(&mut self.theme_data.window_stroke_width).speed(0.1));

        ui.separator();

        // Простые значения
        ui.label("⚙️ Настройки:");

        ui.label("Размер угла окна:");
        ui.add(egui::DragValue::new(&mut self.theme_data.resize_corner_size).speed(1.0));

        ui.checkbox(&mut self.theme_data.button_frame, "Рамка кнопок");
        ui.checkbox(
            &mut self.theme_data.collapsing_header_frame,
            "Рамка заголовков",
        );
        ui.checkbox(&mut self.theme_data.striped, "Полосатые таблицы");

        ui.label("Прозрачность отключенных элементов:");
        ui.add(egui::Slider::new(
            &mut self.theme_data.disabled_alpha,
            0.0..=1.0,
        ));
    }

    // Создаем тему
    fn create_theme(&mut self) {
        self.clear_messages();

        if self.theme_name.is_empty() {
            self.error_message = "Введите название темы".to_string();
            return;
        }

        // Устанавливаем имя темы
        self.theme_data.name = self.theme_name.clone();

        // Сохраняем тему
        if self.save_theme() {
            self.success_message = format!("Тема '{}' успешно создана!", self.theme_name);
        } else {
            self.error_message = "Ошибка при создании темы".to_string();
        }
    }

    // Сохраняем тему в файл
    fn save_theme(&self) -> bool {
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("shroombrella");
            let themes_dir = app_dir.join("themes");

            // Создаем директорию если её нет
            if let Err(_) = std::fs::create_dir_all(&themes_dir) {
                return false;
            }

            // Сохраняем тему в файл
            let theme_path = themes_dir.join(format!("{}.json", self.theme_name));
            match serde_json::to_string_pretty(&self.theme_data) {
                Ok(json) => std::fs::write(&theme_path, json).is_ok(),
                Err(_) => false,
            }
        } else {
            false
        }
    }

    pub fn clear_messages(&mut self) {
        self.error_message.clear();
        self.success_message.clear();
    }

    // Метод для инициализации с шаблоном темы
    pub fn init_with_template(&mut self, template: ThemeVisuals) {
        self.theme_data = template;
        self.clear_messages();
    }

    // Проверяем, была ли успешно создана тема
    pub fn was_theme_created(&self) -> bool {
        !self.success_message.is_empty() && self.success_message.contains("успешно создана")
    }
}
