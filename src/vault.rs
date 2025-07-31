use serde::{Deserialize, Serialize};
use std::time::SystemTime;
#[allow(unused_imports)]
use zeroize::Zeroize;

#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    pub service: String,
    pub login: String,
    #[serde(skip)]
    pub password: String,
}

// Реализуем Zeroize для Entry
impl zeroize::Zeroize for Entry {
    fn zeroize(&mut self) {
        self.service.zeroize();
        self.login.zeroize();
        self.password.zeroize();
    }
}

#[derive(Serialize, Deserialize)]
pub struct VaultHeader {
    pub version: u32,
    pub creation_date: SystemTime,
    pub salt: String,  // base64
    pub nonce: String, // base64
}

#[derive(Serialize, Deserialize)]
pub struct VaultFile {
    pub header: VaultHeader,
    pub data: String, // base64 зашифрованных данных
}

pub struct Vault {
    pub name: String,
    pub file: VaultFile,
}

impl Vault {
    pub fn new(name: String, file: VaultFile) -> Self {
        Self { name, file }
    }

    // Создаем Vault из VaultFile
    pub fn from_file(vault_file: VaultFile, name: String) -> Self {
        Self {
            name,
            file: vault_file,
        }
    }

    pub fn decrypt_entries(&self, master_password: &str) -> Result<Vec<Entry>, String> {
        use crate::crypto::*;
        use base64::{Engine as _, engine::general_purpose};

        // Декодируем метаданные с защитой от ошибок формата
        let salt = general_purpose::STANDARD
            .decode(&self.file.header.salt)
            .map_err(|_| "Неверный формат хранилища (соль)")?;

        let nonce = general_purpose::STANDARD
            .decode(&self.file.header.nonce)
            .map_err(|_| "Неверный формат хранилища (nonce)")?;

        let ciphertext = general_purpose::STANDARD
            .decode(&self.file.data)
            .map_err(|_| "Неверный формат хранилища (данные)")?;

        // Проверяем длину nonce
        if nonce.len() != 12 {
            return Err("Неверный формат хранилища".to_string());
        }

        // Безопасно конвертируем nonce в массив
        let nonce_array: [u8; 12] = match nonce.try_into() {
            Ok(arr) => arr,
            Err(_) => return Err("Неверный формат хранилища".to_string()),
        };

        // Получаем ключ
        let key = derive_key(master_password, &salt);

        // Расшифровываем с обработкой ошибок
        let plaintext = match decrypt_data(&ciphertext, &key, &nonce_array) {
            Ok(data) => data,
            Err(_) => return Err("Неверный мастер-пароль".to_string()), // Упрощенное сообщение
        };

        // Десериализуем записи
        match serde_json::from_slice(&plaintext) {
            Ok(entries) => Ok(entries),
            Err(_) => Err("Неверный формат записей в хранилище".to_string()),
        }
    }
}
