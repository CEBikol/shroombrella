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
        use serde_json;

        // Декодируем метаданные
        let salt = general_purpose::STANDARD
            .decode(&self.file.header.salt)
            .map_err(|_| "Ошибка декодирования соли")?;
        let nonce = general_purpose::STANDARD
            .decode(&self.file.header.nonce)
            .map_err(|_| "Ошибка декодирования nonce")?;
        let ciphertext = general_purpose::STANDARD
            .decode(&self.file.data)
            .map_err(|_| "Ошибка декодирования данных")?;

        if nonce.len() != 12 {
            return Err("Неверная длина nonce".to_string());
        }

        // Получаем ключ и расшифровываем
        let key = derive_key(master_password, &salt);
        let plaintext = decrypt_data(&ciphertext, &key, &nonce.try_into().unwrap());

        // Десериализуем записи
        let entries: Vec<Entry> =
            serde_json::from_slice(&plaintext).map_err(|_| "Ошибка десериализации записей")?;

        Ok(entries)
    }
}
