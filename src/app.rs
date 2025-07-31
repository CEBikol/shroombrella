use crate::{
    settings::Settings,
    storage,
    ui::{
        password_manager_ui::PasswordManager, settings_ui::SettingsWindow,
        theme_creator_ui::ThemeCreator, vault_creator_ui::VaultCreator,
    },
};
use eframe::egui::{self, Widget};
use std::path::PathBuf;

pub struct PasswordApp {
    // Состояние приложения
    state: AppState,

    // Данные формы авторизации
    selected_vault_path: Option<PathBuf>,
    master_password: String,
    error_message: String,

    // Менеджер паролей
    password_manager: PasswordManager,

    // UI состояние
    available_vaults: Vec<PathBuf>,

    // Настройки
    settings_window: SettingsWindow,
    theme_creator: ThemeCreator,
    vault_creator: VaultCreator,
    settings_applied: bool,
}

#[derive(PartialEq)]
pub enum AppState {
    Locked,
    Unlocked,
}

impl Default for PasswordApp {
    fn default() -> Self {
        let settings = Settings::load();
        let available_vaults = storage::get_available_vaults();
        let selected_vault_path = if !available_vaults.is_empty() {
            Some(available_vaults[0].clone())
        } else {
            None
        };

        let password_manager = PasswordManager::new();
        let settings_window = SettingsWindow::new(settings);
        let theme_creator = ThemeCreator::new();
        let vault_creator = VaultCreator::new();
        let settings_applied = false;

        Self {
            state: AppState::Locked,
            selected_vault_path,
            master_password: String::new(),
            error_message: String::new(),
            password_manager,
            available_vaults,
            settings_window,
            theme_creator,
            vault_creator,
            settings_applied,
        }
    }
}

impl eframe::App for PasswordApp {
    // В src/app.rs, метод update
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Применяем настройки UI
        let current_settings = self.settings_window.get_current_settings();
        ctx.set_pixels_per_point(current_settings.ui_scale);
        if let Ok(theme) = current_settings.load_theme() {
            ctx.set_visuals(theme.to_egui_visuals());
        }

        egui::TopBottomPanel::top("custom_title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Создаем область перетаскивания, которая будет занимать ВСЁ пространство
                let drag_response = ui.allocate_response(ui.available_size(), egui::Sense::drag());

                // Добавляем обработчик для перемещения окна
                if drag_response.dragged() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                drag_response.on_hover_cursor(egui::CursorIcon::Grab);

                // Кнопка закрытия (будет поверх области перетаскивания)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let close_btn = egui::Button::new("X")
                        .frame(false)
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE);

                    if ui.add(close_btn).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        // Центральная панель
        egui::CentralPanel::default().show(ctx, |ui| match self.state {
            AppState::Locked => {
                self.show_login_form(ui);
            }
            AppState::Unlocked => {
                if !self.password_manager.ui(ui) {
                    self.logout();
                }
            }
        });

        //  Подвал с настройками и версией
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Кнопка настроек слева
                if ui.button("⚙️ Настройки").clicked() {
                    self.settings_window.show = true;
                }

                // Версия приложения справа
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                });
            });
        });

        // Показываем окно настроек если нужно и проверяем применение
        self.settings_applied = if self.settings_window.show {
            self.settings_window.show(ctx)
        } else {
            false
        };

        // Показываем диалог создания темы если нужно
        if self.theme_creator.show {
            self.theme_creator.show(ctx);
        }

        // Показываем диалог создания хранилища если нужно
        if self.vault_creator.show {
            self.vault_creator.show(ctx);
        }

        // Проверяем запрос на создание темы
        if self.settings_window.take_create_theme_request() {
            // Инициализируем диалог с текущей темой как шаблоном
            // Используем СОХРАНЕННУЮ тему для инициализации редактора
            if let Ok(current_theme) = self.settings_window.get_current_settings().load_theme() {
                self.theme_creator.init_with_template(current_theme);
            }
            self.theme_creator.show = true;
        }

        // Проверяем, была ли создана тема или список тем изменился
        let theme_created_or_list_changed = self.theme_creator.was_theme_created()
            || self.settings_window.take_theme_list_changed(); // <-- Новый флаг

        if theme_created_or_list_changed {
            // Обновляем список тем в настройках
            self.settings_window.refresh_themes();
            // Если тема была создана, закрываем диалог
            if self.theme_creator.was_theme_created() {
                // Закрываем диалог создания темы
                self.theme_creator.show = false;
                // Очищаем сообщения в диалоге создания темы
                self.theme_creator.clear_messages();
            }
        }

        // Проверяем, было ли создано хранилище
        self.check_vault_creation();

        // Если настройки были применены (нажата кнопка "Применить"),
        // обновляем тему и масштаб из СОХРАНЕННЫХ настроек
        if self.settings_applied {
            // Эти вызовы повторяют то, что уже сделано в начале функции,
            // но это гарантирует немедленное применение после нажатия "Применить"
            let applied_settings = self.settings_window.get_current_settings();
            ctx.set_pixels_per_point(applied_settings.ui_scale);
            if let Ok(theme) = applied_settings.load_theme() {
                ctx.set_visuals(theme.to_egui_visuals());
            }
        }
    }
}

impl PasswordApp {
    fn show_login_form(&mut self, ui: &mut egui::Ui) {
        //  Возвращаем выравнивание по левому краю
        ui.heading("Shroombrella");
        ui.separator();

        // Показываем ошибки
        if !self.error_message.is_empty() {
            ui.colored_label(egui::Color32::RED, &self.error_message);
            ui.separator();
        }

        // Выбор хранилища
        ui.label("Выберите хранилище:");
        egui::ComboBox::from_id_salt("vault_selector")
            .selected_text(if let Some(path) = &self.selected_vault_path {
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            } else {
                "Нет доступных хранилищ".to_string()
            })
            .show_ui(ui, |ui| {
                for vault_path in &self.available_vaults {
                    let filename = vault_path.file_name().unwrap_or_default().to_string_lossy();
                    if ui
                        .selectable_label(
                            self.selected_vault_path.as_ref() == Some(vault_path),
                            filename.to_string(),
                        )
                        .clicked()
                    {
                        self.selected_vault_path = Some(vault_path.clone());
                        self.error_message.clear();
                    }
                }
            });

        ui.add_space(10.0);

        // Кнопки для управления хранилищами
        ui.horizontal(|ui| {
            if ui.button("➕ Создать новое").clicked() {
                self.vault_creator.show = true;
                self.vault_creator.vault_name.clear();
                self.vault_creator.master_password.clear();
                self.vault_creator.confirm_password.clear();
                self.vault_creator.clear_messages();
            }

            if ui.button("🔄 Обновить").clicked() {
                self.refresh_vault_list();
            }
        });

        ui.add_space(10.0);

        // Поле ввода пароля
        ui.label("Введите мастер-пароль:");
        ui.horizontal(|ui| {
            ui.label("🔑");
            ui.add(egui::TextEdit::singleline(&mut self.master_password).password(true));
        });

        ui.add_space(10.0);

        // Кнопки
        ui.horizontal(|ui| {
            if ui.button("🔓 Войти").clicked() {
                self.attempt_login();
            }
        });
    }

    fn attempt_login(&mut self) {
        self.error_message.clear();

        if let Some(vault_path) = &self.selected_vault_path {
            match storage::load_vault_from_path(vault_path) {
                Ok(vault) => {
                    match vault.decrypt_entries(&self.master_password) {
                        Ok(entries) => {
                            // Инициализируем менеджер паролей
                            self.password_manager.set_vault(
                                vault,
                                entries,
                                self.master_password.clone(),
                                vault_path.clone(),
                            );
                            self.state = AppState::Unlocked;
                            self.master_password.clear();
                        }
                        Err(e) => {
                            self.error_message = e;
                        }
                    }
                }
                Err(e) => {
                    self.error_message = format!("Ошибка загрузки хранилища: {}", e);
                }
            }
        } else {
            self.error_message = "Выберите хранилище".to_string();
        }
    }

    fn logout(&mut self) {
        self.state = AppState::Locked;
        self.password_manager.clear();
        self.error_message.clear();
    }

    fn refresh_vault_list(&mut self) {
        self.available_vaults = storage::get_available_vaults();
        if !self.available_vaults.is_empty() && self.selected_vault_path.is_none() {
            self.selected_vault_path = Some(self.available_vaults[0].clone());
        }
        // Обновляем список тем в настройках
        self.settings_window.refresh_themes();
    }

    // Проверяем создание хранилища
    fn check_vault_creation(&mut self) {
        if self.vault_creator.was_vault_created() {
            // Обновляем список хранилищ
            self.refresh_vault_list();
            // Устанавливаем созданное хранилище как выбранное
            if let Some(vault_path) = self.vault_creator.take_created_vault_path() {
                self.selected_vault_path = Some(vault_path);
            }
            // Закрываем диалог
            self.vault_creator.show = false;
        }
    }
}
