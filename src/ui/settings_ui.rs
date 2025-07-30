use crate::settings::Settings;
use dirs;
use eframe::egui;

pub struct SettingsWindow {
    pub show: bool,
    settings: Settings,        // Оригинальные настройки (для отката)
    buffer_settings: Settings, // Буфер для временных изменений
    available_themes: Vec<String>,

    // Флаг для запроса создания темы
    pub create_theme_requested: bool,

    // --- Поле для сигнализации об изменении темы ---
    // Указывает, что список тем или текущая тема могли измениться
    pub theme_list_changed: bool,
    // -----------------------------------------------
}

impl SettingsWindow {
    pub fn new(settings: Settings) -> Self {
        let available_themes = settings.get_available_themes();
        let buffer_settings = settings.clone(); // Копируем в буфер

        Self {
            show: false,
            settings,        // Оригинал
            buffer_settings, // Буфер для изменений
            available_themes,
            create_theme_requested: false,
            theme_list_changed: false, // Инициализация
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut show = self.show;
        let mut settings_applied = false;
        egui::Window::new("⚙️ Настройки")
            .open(&mut show)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                settings_applied = self.ui(ui);
            });
        self.show = show;
        settings_applied
    }

    fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut settings_applied = false; // Локальная переменная для отслеживания применения
        let mut local_theme_changed = false; // Локальный флаг для отслеживания изменений тем/списка

        ui.heading("Настройки приложения");
        ui.separator();

        // --- Основной UI настроек ---

        // Выбор темы
        ui.label("🎨 Тема интерфейса:");

        egui::ComboBox::from_label("Тема")
            .selected_text(&self.buffer_settings.current_theme)
            .show_ui(ui, |ui| {
                // Создаем копию вектора, чтобы можно было итерироваться по нему,
                // не беспокоясь о заимствованиях self.available_themes
                let themes_to_show: Vec<String> = self.available_themes.clone();

                for theme_name in &themes_to_show {
                    let is_builtin = theme_name == "Dark" || theme_name == "Light";

                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(
                                &self.buffer_settings.current_theme == theme_name,
                                theme_name,
                            )
                            .clicked()
                        {
                            self.buffer_settings.current_theme = theme_name.clone();
                        }

                        // Добавляем кнопку удаления только для пользовательских тем
                        if !is_builtin {
                            ui.add_space(5.0);

                            // --- Используем clone для имени темы перед замыканием ---
                            let theme_name_clone = theme_name.clone();
                            // --- Используем clone для текущей темы из буфера перед замыканием ---
                            let current_buffer_theme = self.buffer_settings.current_theme.clone();

                            if ui.button("🗑").on_hover_text("Удалить тему").clicked()
                            {
                                // Немедленно удаляем тему
                                Self::delete_theme(&theme_name_clone);
                                // Обновляем список тем
                                self.refresh_themes();
                                // Сообщаем, что список тем изменился
                                local_theme_changed = true;

                                // Если удаленная тема была выбрана в буфере, переключаем буфер
                                if current_buffer_theme == theme_name_clone {
                                    self.buffer_settings.current_theme = "Dark".to_string();
                                    // settings_applied останется false, так как это только предпросмотр
                                    // Фактическое применение будет по кнопке "Применить"
                                }
                            }
                        }
                    });
                }
            });
        ui.add_space(10.0);

        // Кнопка добавления новой темы
        if ui.button("➕ Добавить новую тему").clicked() {
            self.create_theme_requested = true;
        }

        ui.separator();

        // Масштаб интерфейса
        ui.label("🔍 Масштаб интерфейса:");
        ui.add(egui::Slider::new(&mut self.buffer_settings.ui_scale, 0.5..=3.0).text("Масштаб"));
        ui.label(format!(
            "Текущий масштаб: {:.1}x",
            self.buffer_settings.ui_scale
        ));

        ui.separator();
        ui.separator();

        // Кнопки
        ui.horizontal(|ui| {
            if ui.button("💾 Применить").clicked() {
                // Применяем изменения
                self.settings = self.buffer_settings.clone();
                if let Err(e) = self.settings.save() {
                    println!("Ошибка сохранения настроек: {}", e);
                } else {
                    println!("Настройки применены и сохранены!");
                    settings_applied = true; // Устанавливаем флаг применения
                    // Если тема или список тем изменились, сигнализируем об этом
                    if local_theme_changed {
                        self.theme_list_changed = true;
                    }
                }
            }

            if ui.button("↩️ Отмена").clicked() {
                // Откатываем изменения
                self.buffer_settings = self.settings.clone();
                // Сбрасываем флаг локальных изменений тем
                local_theme_changed = false;
            }
        });

        // Если тема была изменена локально (удалена), но не применена,
        // все равно сигнализируем основному приложению для обновления списка
        if local_theme_changed && !settings_applied {
            self.theme_list_changed = true;
        }

        settings_applied // Возвращаем флаг применения
    }

    // Метод для получения и сброса флага запроса создания темы
    pub fn take_create_theme_request(&mut self) -> bool {
        let requested = self.create_theme_requested;
        self.create_theme_requested = false;
        requested
    }

    // Метод для получения и сброса флага изменения тем
    pub fn take_theme_list_changed(&mut self) -> bool {
        let changed = self.theme_list_changed;
        self.theme_list_changed = false;
        changed
    }

    // Получаем текущие (буферные) настройки для предварительного просмотра
    // (например, масштаба в реальном времени)
    // pub fn get_buffer_settings(&self) -> &Settings {
    //     &self.buffer_settings
    // }

    // Получаем текущие (сохраненные) настройки
    pub fn get_current_settings(&self) -> &Settings {
        &self.settings
    }

    // Обновляем список доступных тем
    pub fn refresh_themes(&mut self) {
        self.available_themes = self.settings.get_available_themes();
        // Буфер не обновляем, так как пользователь может быть в процессе редактирования
        // self.buffer_settings = self.settings.clone(); // Убираем это
    }

    // --- Приватный метод для удаления темы из файловой системы ---
    // Вызывается напрямую
    fn delete_theme(theme_name: &str) {
        // Проверяем, что это не встроенная тема
        if theme_name == "Dark" || theme_name == "Light" {
            println!("Невозможно удалить встроенную тему: {}", theme_name);
            return;
        }

        // Получаем путь к файлу темы
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("shroombrella");
            let themes_dir = app_dir.join("themes");
            let theme_path = themes_dir.join(format!("{}.json", theme_name));

            // Пытаемся удалить файл
            match std::fs::remove_file(&theme_path) {
                Ok(_) => {
                    println!("Тема '{}' успешно удалена", theme_name);
                }
                Err(e) => {
                    eprintln!("Ошибка удаления темы '{}': {}", theme_name, e);
                }
            }
        }
    }
}
