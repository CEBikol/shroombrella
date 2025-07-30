use crate::crypto::*;
use crate::vault::*;
use base64::{Engine as _, engine::general_purpose};
use dirs::config_dir;
use rand::RngCore;
use serde_json;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub fn load_vault_from_path(path: &Path) -> Result<Vault, String> {
    let data = fs::read_to_string(path).map_err(|_| "Файл хранилища не найден")?;

    let vault_file: VaultFile =
        serde_json::from_str(&data).map_err(|_| "Ошибка чтения файла хранилища")?;

    // Получаем имя файла
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "vault".to_string());

    Ok(Vault::from_file(vault_file, name))
}

pub fn save_vault_to_path(vault: &Vault, path: &Path) -> Result<(), String> {
    let json =
        serde_json::to_string_pretty(&vault.file).map_err(|_| "Ошибка сериализации хранилища")?;

    fs::write(path, json).map_err(|_| "Ошибка записи файла хранилища")?;

    Ok(())
}

pub fn get_vault_path(name: &str) -> Option<PathBuf> {
    config_dir().map(|config_dir| {
        config_dir
            .join("shroombrella")
            .join(name)
            .with_extension("vault")
    })
}

pub fn create_encrypted_vault(
    entries: &[Entry],
    master_password: &str,
    path: &Path,
) -> Result<Vault, String> {
    use base64::{Engine as _, engine::general_purpose};
    use rand::RngCore;
    use std::time::SystemTime;

    // Генерируем соль и nonce
    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];
    rand::rng().fill_bytes(&mut salt);
    rand::rng().fill_bytes(&mut nonce);

    // Получаем ключ
    let key = derive_key(master_password, &salt);

    // Сериализуем записи
    let plaintext = serde_json::to_vec(entries).map_err(|_| "Ошибка сериализации записей")?;

    // Шифруем данные
    let (ciphertext, used_nonce) = encrypt_data(&plaintext, &key);

    // Создаем файл хранилища
    let vault_file = VaultFile {
        header: VaultHeader {
            version: 1,
            creation_date: SystemTime::now(),
            salt: general_purpose::STANDARD.encode(salt),
            nonce: general_purpose::STANDARD.encode(used_nonce),
        },
        data: general_purpose::STANDARD.encode(ciphertext),
    };

    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "vault".to_string());

    let vault = Vault::new(name, vault_file);

    // Сохраняем в файл
    save_vault_to_path(&vault, path)?;

    Ok(vault)
}

pub fn create_new_vault(name: String, password: &str) -> Result<Vault, String> {
    if let Some(config_dir) = config_dir() {
        let app_dir = config_dir.join("shroombrella");
        std::fs::create_dir_all(&app_dir).map_err(|_| "Не удалось создать директорию")?;
        use std::time::SystemTime;

        // Добавляем расширение .vault к имени файла
        let path = app_dir.join(&name).with_extension("vault");

        // Генерируем соль и nonce
        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 12];
        rand::rng().fill_bytes(&mut salt);
        rand::rng().fill_bytes(&mut nonce);

        // Получаем ключ
        let key = derive_key(password, &salt);

        // Создаем пустой список записей
        let empty_entries: Vec<Entry> = Vec::new();
        let plaintext =
            serde_json::to_vec(&empty_entries).map_err(|_| "Ошибка сериализации пустого списка")?;

        // Шифруем данные
        let (ciphertext, used_nonce) = encrypt_data(&plaintext, &key);

        // Создаем файл хранилища
        let vault_file = VaultFile {
            header: VaultHeader {
                version: 1,
                creation_date: SystemTime::now(),
                salt: general_purpose::STANDARD.encode(salt),
                nonce: general_purpose::STANDARD.encode(used_nonce),
            },
            data: general_purpose::STANDARD.encode(ciphertext),
        };

        let vault = Vault::new(name, vault_file);

        // Сохраняем в файл
        save_vault_to_path(&vault, &path)?;

        Ok(vault)
    } else {
        Err("Конфигурационный путь недействителен".to_string())
    }
}

// Получение списка всех доступных хранилищ в директории
pub fn get_available_vaults() -> Vec<std::path::PathBuf> {
    use dirs;
    let mut vaults = Vec::new();

    if let Some(config_dir) = dirs::config_dir() {
        let app_dir = config_dir.join("shroombrella");
        if app_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&app_dir) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "vault") {
                        vaults.push(path);
                    }
                }
            }
        }
    }

    vaults
}
