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

pub fn decrypt_data(
    ciphertext: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 12],
) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Invalid key: {}", e))?;
    let nonce_array = Nonce::from_slice(nonce);
    cipher
        .decrypt(nonce_array, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))
}
