use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::Argon2;
use rand::RngCore;

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .expect("Argon2 failed");
    key
}

pub fn encrypt_data(data: &[u8], key: &[u8; 32]) -> (Vec<u8>, [u8; 12]) {
    let cipher = Aes256Gcm::new_from_slice(key).expect("Invalid key length");
    let mut nonce = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce);
    let nonce_array = Nonce::from_slice(&nonce);
    let ciphertext = cipher
        .encrypt(nonce_array, data)
        .expect("Encryption failed");
    (ciphertext, nonce)
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> Vec<u8> {
    let cipher = Aes256Gcm::new_from_slice(key).expect("Invalid key length");
    let nonce_array = Nonce::from_slice(nonce);
    cipher
        .decrypt(nonce_array, ciphertext)
        .expect("Decryption failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let original_data = b"Hello, World!";
        let password = "test_password";
        let salt = [1; 16];

        let key = derive_key(password, &salt);
        let (ciphertext, nonce) = encrypt_data(original_data, &key);
        let decrypted_data = decrypt_data(&ciphertext, &key, &nonce);

        assert_eq!(original_data.as_slice(), decrypted_data.as_slice());
    }

    #[test]
    fn test_unicode_password() {
        let original_data = b"Test data";
        let password = "пароль_🦀";
        let salt = [2; 16];

        let key = derive_key(password, &salt);
        let (ciphertext, nonce) = encrypt_data(original_data, &key);
        let decrypted_data = decrypt_data(&ciphertext, &key, &nonce);

        assert_eq!(original_data, decrypted_data.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip_big_sault() {
        let original_data = b"Hello, World! This is a secret message.";
        let password = "my_secret_password_123";
        let salt = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

        let key = derive_key(password, &salt);
        let (ciphertext, nonce) = encrypt_data(original_data, &key);
        let decrypted_data = decrypt_data(&ciphertext, &key, &nonce);

        assert_eq!(original_data.as_slice(), decrypted_data.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_empty_data() {
        let original_data = b"";
        let password = "empty_test_password";
        let salt = [0; 16];

        let key = derive_key(password, &salt);
        let (ciphertext, nonce) = encrypt_data(original_data, &key);
        let decrypted_data = decrypt_data(&ciphertext, &key, &nonce);

        assert_eq!(original_data.as_slice(), decrypted_data.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_large_data() {
        let original_data: Vec<u8> = (0..10000u16).map(|x| (x % 256) as u8).collect();
        let password = "large_data_password";
        let salt = [255; 16];

        let key = derive_key(password, &salt);
        let (ciphertext, nonce) = encrypt_data(&original_data, &key);
        let decrypted_data = decrypt_data(&ciphertext, &key, &nonce);

        assert_eq!(original_data, decrypted_data);
    }

    #[test]
    fn test_different_passwords_produce_different_keys() {
        let salt = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        let password1 = "password1";
        let password2 = "password2";

        let key1 = derive_key(password1, &salt);
        let key2 = derive_key(password2, &salt);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_same_password_and_salt_produce_same_key() {
        let salt = [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5];
        let password = "consistent_password";

        let key1 = derive_key(password, &salt);
        let key2 = derive_key(password, &salt);

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_different_salts_produce_different_keys() {
        let password = "salt_test_password";
        let salt1 = [1; 16];
        let salt2 = [2; 16];

        let key1 = derive_key(password, &salt1);
        let key2 = derive_key(password, &salt2);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let original_data = b"Secret data";
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";
        let salt = [
            10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160,
        ];

        let correct_key = derive_key(correct_password, &salt);
        let (ciphertext, nonce) = encrypt_data(original_data, &correct_key);

        let wrong_key = derive_key(wrong_password, &salt);

        let result = std::panic::catch_unwind(|| decrypt_data(&ciphertext, &wrong_key, &nonce));

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_wrong_nonce_fails() {
        let original_data = b"Data with wrong nonce";
        let password = "nonce_test_password";
        let salt = [7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7];
        let wrong_nonce = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

        let key = derive_key(password, &salt);
        let (ciphertext, _correct_nonce) = encrypt_data(original_data, &key);

        let result = std::panic::catch_unwind(|| decrypt_data(&ciphertext, &key, &wrong_nonce));

        assert!(result.is_err());
    }

    #[test]
    fn test_key_derivation_validity() {
        let password = "test_password_123";
        let salt = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

        let key = derive_key(password, salt);

        assert_eq!(key.len(), 32);
        assert!(!key.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_unicode_password_support() {
        // Используем правильные UTF-8 байты для юникодной строки
        let original_data = "Unicode test data: Привет, мир! 🦀".as_bytes();
        let password = "пароль_с_юникодом_🦀";
        let salt = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

        let key = derive_key(password, &salt);
        let (ciphertext, nonce) = encrypt_data(original_data, &key);
        let decrypted_data = decrypt_data(&ciphertext, &key, &nonce);

        assert_eq!(original_data, decrypted_data.as_slice());
    }

    #[test]
    fn test_special_characters_in_password() {
        let original_data = b"Data with special chars";
        // Пароль с различными специальными символами
        let password = "p@ssw0rd!#$%^&*()_+-=[]{}|;':\",./<>?";
        let salt = [42; 16];

        let key = derive_key(password, &salt);
        let (ciphertext, nonce) = encrypt_data(original_data, &key);
        let decrypted_data = decrypt_data(&ciphertext, &key, &nonce);

        assert_eq!(original_data, decrypted_data.as_slice());
    }
}
