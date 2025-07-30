use crate::storage;
use eframe::egui;
use std::path::PathBuf;
use zeroize::Zeroize;

pub struct VaultCreator {
    pub show: bool,
    pub vault_name: String,
    pub master_password: String,
    pub confirm_password: String,
    pub error_message: String,
    pub success_message: String,
    pub created_vault_path: Option<PathBuf>,
}

impl VaultCreator {
    pub fn new() -> Self {
        Self {
            show: false,
            vault_name: String::new(),
            master_password: String::new(),
            confirm_password: String::new(),
            error_message: String::new(),
            success_message: String::new(),
            created_vault_path: None,
        }
    }

    pub fn clear_messages(&mut self) {
        self.error_message.clear();
        self.success_message.clear();
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let mut show = self.show;
        egui::Window::new("➕ Создать новое хранилище")
            .open(&mut show)
            .resizable(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });
        self.show = show;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Создание нового хранилища");
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

        // Название хранилища
        ui.label("Название хранилища:");
        ui.text_edit_singleline(&mut self.vault_name);

        ui.separator();

        // Мастер-пароль
        ui.label("Мастер-пароль:");
        ui.horizontal(|ui| {
            ui.label("🔑");
            ui.add(egui::TextEdit::singleline(&mut self.master_password).password(true));
        });

        ui.label("Подтвердите пароль:");
        ui.horizontal(|ui| {
            ui.label("🔑");
            ui.add(egui::TextEdit::singleline(&mut self.confirm_password).password(true));
        });

        ui.separator();

        // Кнопки
        ui.horizontal(|ui| {
            if ui.button("✅ Создать").clicked() {
                self.create_vault();
            }

            if ui.button("❌ Отмена").clicked() {
                self.show = false;
                self.clear_messages();
            }
        });
    }

    // Создаем хранилище
    fn create_vault(&mut self) {
        self.clear_messages();

        // Валидация
        if self.vault_name.is_empty() {
            self.error_message = "Введите название хранилища".to_string();
            return;
        }

        if self.master_password.is_empty() {
            self.error_message = "Введите мастер-пароль".to_string();
            return;
        }

        if self.master_password != self.confirm_password {
            self.error_message = "Пароли не совпадают".to_string();
            return;
        }

        // Создаем хранилище
        match self.save_new_vault() {
            Ok(path) => {
                self.success_message = format!("Хранилище '{}' успешно создано!", self.vault_name);
                self.created_vault_path = Some(path);
            }
            Err(e) => {
                self.error_message = format!("Ошибка создания хранилища: {}", e);
            }
        }

        if self.save_new_vault().is_ok() {
            self.success_message = format!("Хранилище '{}' успешно создано!", self.vault_name);
            // Очищаем пароли!
            self.master_password.zeroize();
            self.confirm_password.zeroize();
        }
    }

    // Сохраняем новое хранилище
    fn save_new_vault(&self) -> Result<PathBuf, String> {
        // Создаем новое хранилище
        storage::create_new_vault(self.vault_name.clone(), &self.master_password)?;

        // Получаем путь до созданного хранилища
        storage::get_vault_path(&self.vault_name)
            .ok_or_else(|| "Не удалось получить путь к хранилищу".to_string())
    }

    // Проверяем, было ли успешно создано хранилище
    pub fn was_vault_created(&self) -> bool {
        !self.success_message.is_empty() && self.success_message.contains("успешно создано")
    }

    // Получаем путь к созданному хранилищу
    pub fn take_created_vault_path(&mut self) -> Option<PathBuf> {
        self.created_vault_path.take()
    }
}
