use crate::storage;
use crate::vault::{Entry, Vault};
use eframe::egui;
use std::path::PathBuf;
use zeroize::Zeroize;

// Структура для редактирования записи
#[derive(Clone)]
pub struct EditEntry {
    pub index: usize,
    pub service: String,
    pub login: String,
    pub password: String,
}

impl zeroize::Zeroize for EditEntry {
    fn zeroize(&mut self) {
        self.service.zeroize();
        self.login.zeroize();
        self.password.zeroize();
    }
}

pub struct PasswordManager {
    pub current_vault: Option<Vault>,
    pub decrypted_entries: Vec<Entry>,
    pub master_password: String,
    pub vault_path: Option<PathBuf>,

    // Для добавления новых записей
    pub new_service: String,
    pub new_login: String,
    pub new_password: String,

    // Для редактирования
    pub edit_entry: Option<EditEntry>,
    pub show_edit_dialog: bool,

    // UI состояние
    pub hovered_password_index: Option<usize>,
    pub error_message: String,

    app_state: bool,
}

impl zeroize::Zeroize for PasswordManager {
    fn zeroize(&mut self) {
        self.decrypted_entries.zeroize();
        self.master_password.zeroize();
        self.edit_entry.zeroize();
    }
}

impl PasswordManager {
    pub fn new() -> Self {
        Self {
            current_vault: None,
            decrypted_entries: Vec::new(),
            master_password: String::new(),
            vault_path: None,
            new_service: String::new(),
            new_login: String::new(),
            new_password: String::new(),
            edit_entry: None,
            show_edit_dialog: false,
            hovered_password_index: None,
            error_message: String::new(),
            app_state: true,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        // Заголовок с информацией о хранилище
        self.show_header(ui);
        ui.separator();

        // Показываем ошибки
        if !self.error_message.is_empty() {
            ui.colored_label(egui::Color32::RED, &self.error_message);
            ui.separator();
        }

        // Форма для добавления новой записи
        self.show_add_form(ui);
        ui.separator();

        // Таблица с паролями
        self.show_password_table(ui);

        // Показываем диалог редактирования если нужно
        if self.show_edit_dialog {
            self.show_edit_dialog_ui(ui.ctx());
        }

        if !self.app_state {
            self.zeroize();
        }

        self.app_state
    }

    fn show_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(format!(
                "🗄️ Хранилище: {}",
                self.current_vault
                    .as_ref()
                    .map(|v| v.name.clone())
                    .unwrap_or_default()
            ));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Кнопка выхода
                if ui.button("🚪 Выйти").clicked() {
                    self.app_state = false;
                }
            });
        });
    }

    fn show_add_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("➕ Добавить новую запись");

        ui.horizontal(|ui| {
            ui.label("🌐 Сервис:");
            ui.text_edit_singleline(&mut self.new_service);
        });

        ui.horizontal(|ui| {
            ui.label("👤 Логин:");
            ui.text_edit_singleline(&mut self.new_login);
        });

        ui.horizontal(|ui| {
            ui.label("🔑 Пароль:");
            ui.text_edit_singleline(&mut self.new_password);
        });

        let all_filled = !self.new_service.is_empty()
            && !self.new_login.is_empty()
            && !self.new_password.is_empty();

        ui.horizontal(|ui| {
            if ui
                .add_enabled(all_filled, egui::Button::new("➕ Добавить"))
                .clicked()
            {
                self.add_new_entry();
            }

            if ui.button("🔄 Сгенерировать").clicked() {
                self.new_password = self.generate_password(16);
            }

            if ui.button("💾 Сохранить").clicked() {
                self.save_vault();
            }
        });
    }

    fn show_password_table(&mut self, ui: &mut egui::Ui) {
        if self.decrypted_entries.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("📭 Нет сохранённых паролей");
            });
            return;
        }

        // Заголовок таблицы
        egui::Grid::new("passwords_header")
            .spacing([20.0, 8.0])
            .min_col_width(150.0)
            .show(ui, |ui| {
                ui.heading("🌐 Сервис");
                ui.heading("👤 Логин");
                ui.heading("🔑 Пароль");
                ui.heading(""); // Для кнопок действий
                ui.end_row();
            });

        ui.separator();

        // Скроллируемая область с паролями
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("passwords_grid")
                .striped(true)
                .spacing([20.0, 8.0])
                .min_col_width(150.0)
                .show(ui, |ui| {
                    // Используем индексы для избежания конфликтов заимствования
                    let entries_count = self.decrypted_entries.len();
                    for index in 0..entries_count {
                        if let Some(entry) = self.decrypted_entries.get(index) {
                            let service = entry.service.clone();
                            let login = entry.login.clone();
                            let password = entry.password.clone();

                            ui.label(&service);
                            ui.label(&login);

                            // Скрытие/показ пароля при наведении
                            let password_text = if self.hovered_password_index == Some(index) {
                                password.clone()
                            } else {
                                "••••••••".to_string()
                            };

                            let response = ui.add(egui::Label::new(password_text));
                            if response.hovered() {
                                self.hovered_password_index = Some(index);
                            } else if self.hovered_password_index == Some(index) {
                                self.hovered_password_index = None;
                            }

                            // Кнопки действий (создаем копии для замыканий)
                            let index_copy = index;
                            let password_copy = password.clone();

                            ui.horizontal(|ui| {
                                if ui.button("📋").on_hover_text("Копировать").clicked()
                                {
                                    ui.ctx().copy_text(password_copy);
                                }

                                if ui.button("✏️").on_hover_text("Редактировать").clicked()
                                {
                                    // Клонируем запись для редактирования
                                    if let Some(entry) =
                                        self.decrypted_entries.get(index_copy).cloned()
                                    {
                                        self.edit_entry = Some(EditEntry {
                                            index: index_copy,
                                            service: entry.service,
                                            login: entry.login,
                                            password: entry.password,
                                        });
                                        self.show_edit_dialog = true;
                                    }
                                }

                                if ui.button("🗑️").on_hover_text("Удалить").clicked() {
                                    self.decrypted_entries.remove(index_copy);
                                    // Корректируем индексы при наведении
                                    if let Some(hovered_index) = self.hovered_password_index {
                                        if hovered_index == index_copy {
                                            self.hovered_password_index = None;
                                        } else if hovered_index > index_copy {
                                            self.hovered_password_index = Some(hovered_index - 1);
                                        }
                                    }
                                    // Автоматически сохраняем изменения
                                    self.save_vault();
                                }
                            });

                            ui.end_row();
                        }
                    }
                });
        });
    }

    // Диалог редактирования записи
    fn show_edit_dialog_ui(&mut self, ctx: &egui::Context) {
        // Создаем временную переменную вместо прямого заимствования
        let mut show_dialog = self.show_edit_dialog;
        if !show_dialog {
            return;
        }

        // Создаем копии данных для диалога
        let (mut service, mut login, mut password, index) = if let Some(ref entry) = self.edit_entry
        {
            (
                entry.service.clone(),
                entry.login.clone(),
                entry.password.clone(),
                entry.index,
            )
        } else {
            (String::new(), String::new(), String::new(), 0)
        };

        // Используем временную переменную
        egui::Window::new("✏️ Редактировать запись")
            .open(&mut show_dialog)
            .resizable(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.label("🌐 Сервис:");
                ui.text_edit_singleline(&mut service);

                ui.label("👤 Логин:");
                ui.text_edit_singleline(&mut login);

                ui.label("🔑 Пароль:");
                ui.text_edit_singleline(&mut password);

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("✅ Сохранить").clicked() {
                        // Сохраняем изменения в основном списке
                        if index < self.decrypted_entries.len() {
                            self.decrypted_entries[index] = Entry {
                                service: service,
                                login: login,
                                password: password,
                            };
                            self.show_edit_dialog = false; // Закрываем диалог
                            self.error_message.clear();
                            self.save_vault();
                        }
                    }

                    if ui.button("❌ Отмена").clicked() {
                        self.show_edit_dialog = false; // Закрываем диалог
                    }
                });
            });

        // Обновляем состояние после закрытия диалога
        self.show_edit_dialog = show_dialog;
        if !show_dialog {
            self.edit_entry = None;
        }
    }

    // Добавляем новую запись
    fn add_new_entry(&mut self) {
        if self.new_service.is_empty() || self.new_login.is_empty() || self.new_password.is_empty()
        {
            self.error_message = "Все поля должны быть заполнены".to_string();
            return;
        }

        let new_entry = Entry {
            service: self.new_service.clone(),
            login: self.new_login.clone(),
            password: self.new_password.clone(),
        };

        self.decrypted_entries.push(new_entry);

        // Очищаем поля
        self.new_service.clear();
        self.new_login.clear();
        self.new_password.clear();
        self.error_message.clear();

        // Автоматически сохраняем изменения
        self.save_vault();
    }

    // Сохраняем хранилище
    fn save_vault(&mut self) {
        if let (Some(path), ref master_password) = (&self.vault_path, &self.master_password) {
            match storage::create_encrypted_vault(&self.decrypted_entries, master_password, path) {
                Ok(new_vault) => {
                    self.current_vault = Some(new_vault);
                    self.error_message = "✅ Сохранено!".to_string();
                }
                Err(e) => {
                    self.error_message = format!("❌ Ошибка сохранения: {}", e);
                }
            }
        } else {
            self.error_message = "❌ Нет данных для сохранения".to_string();
        }
    }

    // Генерируем случайный пароль
    fn generate_password(&self, length: usize) -> String {
        use rand::{Rng, distr::Alphanumeric};
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }

    // Устанавливаем текущее хранилище
    pub fn set_vault(
        &mut self,
        vault: Vault,
        entries: Vec<Entry>,
        master_password: String,
        path: PathBuf,
    ) {
        self.current_vault = Some(vault);
        self.decrypted_entries = entries;
        self.master_password = master_password;
        self.vault_path = Some(path);
    }

    // Очищаем данные при выходе (с zeroize)
    pub fn clear(&mut self) {
        self.clear_sensitive_data();
        self.current_vault = None;
        self.vault_path = None;
        self.new_service.clear();
        self.new_login.clear();
        self.edit_entry = None;
        self.show_edit_dialog = false;
        self.hovered_password_index = None;
        self.error_message.clear();
    }

    // Безопасная очистка конфиденциальных данных
    pub fn clear_sensitive_data(&mut self) {
        // Очищаем мастер-пароль
        self.master_password.zeroize();

        // Очищаем новый пароль
        self.new_password.zeroize();

        // Очищаем все записи
        for entry in &mut self.decrypted_entries {
            entry.zeroize();
        }
        self.decrypted_entries.clear();

        // Очищаем данные редактирования
        if let Some(ref mut edit_entry) = self.edit_entry {
            edit_entry.zeroize();
        }
    }
}
