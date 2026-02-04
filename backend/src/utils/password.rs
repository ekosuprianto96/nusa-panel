//! # Password Utilities
//!
//! Modul untuk hashing, verifikasi, dan enkripsi password.
//! - Hashing: Argon2id untuk user authentication
//! - Enkripsi: AES-256-GCM untuk SSO (reversible)

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;

use crate::config::CONFIG;
use crate::errors::ApiError;

/// Hash password menggunakan Argon2id
///
/// Argon2id adalah algoritma yang direkomendasikan oleh OWASP untuk password hashing.
/// Lebih aman dari bcrypt karena resistant terhadap GPU attacks.
///
/// # Arguments
/// * `password` - Plain text password
///
/// # Returns
/// Hashed password string (PHC format)
///
/// # Errors
/// Returns ApiError jika hashing gagal
///
/// # Example
/// ```rust
/// let hashed = hash_password("MySecureP@ss123")?;
/// ```
pub fn hash_password(password: &str) -> Result<String, ApiError> {
    // Generate random salt
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 dengan konfigurasi default (sudah aman)
    let argon2 = Argon2::default();

    // Hash password
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| {
            tracing::error!("Password hashing failed: {}", e);
            ApiError::InternalError("Failed to hash password".to_string())
        })
}

/// Verifikasi password terhadap hash
///
/// # Arguments
/// * `password` - Plain text password yang akan diverifikasi
/// * `hash` - Hash password dari database
///
/// # Returns
/// true jika password cocok, false jika tidak
///
/// # Errors
/// Returns ApiError jika verifikasi gagal karena error teknis
///
/// # Example
/// ```rust
/// if verify_password("MySecureP@ss123", &stored_hash)? {
///     println!("Password valid!");
/// }
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<bool, ApiError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| {
        tracing::error!("Failed to parse password hash: {}", e);
        ApiError::InternalError("Invalid password hash format".to_string())
    })?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Validasi kekuatan password
///
/// Memeriksa apakah password memenuhi requirement keamanan:
/// - Panjang minimum
/// - Mengandung huruf besar
/// - Mengandung angka
/// - Mengandung karakter spesial
///
/// # Arguments
/// * `password` - Password yang akan divalidasi
///
/// # Returns
/// Ok(()) jika password valid
///
/// # Errors
/// Returns ApiError::WeakPassword jika password tidak memenuhi requirement
///
/// # Example
/// ```rust
/// validate_password_strength("MySecureP@ss123")?;
/// ```
pub fn validate_password_strength(password: &str) -> Result<(), ApiError> {
    let security_config = &CONFIG.security;
    let min_length = security_config.password_min_length;

    // Check minimum length
    if password.len() < min_length {
        return Err(ApiError::WeakPassword(min_length));
    }

    // Check uppercase requirement
    if security_config.password_require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
        return Err(ApiError::ValidationError(
            "Password harus mengandung minimal satu huruf besar".to_string(),
        ));
    }

    // Check number requirement
    if security_config.password_require_number && !password.chars().any(|c| c.is_numeric()) {
        return Err(ApiError::ValidationError(
            "Password harus mengandung minimal satu angka".to_string(),
        ));
    }

    // Check special character requirement
    if security_config.password_require_special {
        let special_chars = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
        if !password.chars().any(|c| special_chars.contains(c)) {
            return Err(ApiError::ValidationError(
                "Password harus mengandung minimal satu karakter spesial".to_string(),
            ));
        }
    }

    // Check for common weak passwords
    let weak_passwords = [
        "password",
        "password123",
        "123456",
        "12345678",
        "qwerty",
        "admin",
        "letmein",
        "welcome",
    ];
    if weak_passwords
        .iter()
        .any(|&weak| password.to_lowercase() == weak)
    {
        return Err(ApiError::ValidationError(
            "Password terlalu umum dan mudah ditebak".to_string(),
        ));
    }

    Ok(())
}

/// Generate random password
///
/// Menghasilkan password acak yang memenuhi semua requirement keamanan.
///
/// # Arguments
/// * `length` - Panjang password yang diinginkan (minimal 12)
///
/// # Returns
/// Password acak yang aman
pub fn generate_random_password(length: usize) -> String {
    use rand::Rng;

    let length = length.max(12); // Minimal 12 karakter

    let lowercase = "abcdefghijklmnopqrstuvwxyz";
    let uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let digits = "0123456789";
    let special = "!@#$%^&*";

    let all_chars: Vec<char> = format!("{}{}{}{}", lowercase, uppercase, digits, special)
        .chars()
        .collect();

    let mut rng = rand::thread_rng();
    let mut password: Vec<char> = Vec::with_capacity(length);

    // Pastikan ada minimal satu dari setiap tipe
    password.push(lowercase.chars().nth(rng.gen_range(0..lowercase.len())).unwrap());
    password.push(uppercase.chars().nth(rng.gen_range(0..uppercase.len())).unwrap());
    password.push(digits.chars().nth(rng.gen_range(0..digits.len())).unwrap());
    password.push(special.chars().nth(rng.gen_range(0..special.len())).unwrap());

    // Fill sisanya dengan karakter random
    for _ in 4..length {
        password.push(all_chars[rng.gen_range(0..all_chars.len())]);
    }

    // Shuffle password
    use rand::seq::SliceRandom;
    password.shuffle(&mut rng);

    password.into_iter().collect()
}

// ==========================================
// PASSWORD ENCRYPTION (untuk SSO)
// ==========================================

/// Encrypt password menggunakan AES-256-GCM
///
/// Password dienkripsi (bukan di-hash) sehingga bisa didekripsi untuk SSO.
/// Format output: base64(nonce + ciphertext)
///
/// # Arguments
/// * `password` - Plain text password
///
/// # Returns
/// Encrypted password string (base64 encoded)
pub fn encrypt_password(password: &str) -> Result<String, ApiError> {
    let key_b64 = &CONFIG.security.encryption_master_key;
    let key_bytes = BASE64.decode(key_b64).map_err(|e| {
        tracing::error!("Invalid encryption key format: {}", e);
        ApiError::InternalError("Invalid encryption key".to_string())
    })?;

    if key_bytes.len() != 32 {
        return Err(ApiError::InternalError(format!(
            "Encryption key must be 32 bytes, got {}",
            key_bytes.len()
        )));
    }

    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| {
        tracing::error!("Failed to create cipher: {}", e);
        ApiError::InternalError("Encryption setup failed".to_string())
    })?;

    // Generate random 12-byte nonce
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, password.as_bytes())
        .map_err(|e| {
            tracing::error!("Encryption failed: {}", e);
            ApiError::InternalError("Failed to encrypt password".to_string())
        })?;

    // Combine nonce + ciphertext and encode as base64
    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

/// Decrypt password yang dienkripsi dengan encrypt_password
///
/// # Arguments
/// * `encrypted` - Base64 encoded encrypted password (nonce + ciphertext)
///
/// # Returns
/// Plain text password
pub fn decrypt_password(encrypted: &str) -> Result<String, ApiError> {
    let key_b64 = &CONFIG.security.encryption_master_key;
    let key_bytes = BASE64.decode(key_b64).map_err(|e| {
        tracing::error!("Invalid encryption key format: {}", e);
        ApiError::InternalError("Invalid encryption key".to_string())
    })?;

    if key_bytes.len() != 32 {
        return Err(ApiError::InternalError(format!(
            "Encryption key must be 32 bytes, got {}",
            key_bytes.len()
        )));
    }

    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| {
        tracing::error!("Failed to create cipher: {}", e);
        ApiError::InternalError("Decryption setup failed".to_string())
    })?;

    // Decode base64
    let combined = BASE64.decode(encrypted).map_err(|e| {
        tracing::error!("Invalid encrypted password format: {}", e);
        ApiError::InternalError("Invalid encrypted password".to_string())
    })?;

    if combined.len() < 12 {
        return Err(ApiError::InternalError(
            "Encrypted password too short".to_string(),
        ));
    }

    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt
    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
        tracing::error!("Decryption failed: {}", e);
        ApiError::InternalError("Failed to decrypt password".to_string())
    })?;

    String::from_utf8(plaintext).map_err(|e| {
        tracing::error!("Invalid UTF-8 in decrypted password: {}", e);
        ApiError::InternalError("Decrypted password is invalid".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "Admin@123";
        let hash = hash_password(password).expect("Failed to hash password");

        print!("Hashed password: {}", hash);

        // Verify correct password
        assert!(verify_password(password, &hash).expect("Verification failed"));

        // Verify wrong password
        assert!(!verify_password("WrongPassword", &hash).expect("Verification failed"));
    }

    #[test]
    fn test_validate_password_strength() {
        // Valid password
        assert!(validate_password_strength("MyStr0ng@Pass").is_ok());

        // Too short
        assert!(validate_password_strength("Short1!").is_err());
    }

    #[test]
    fn test_generate_random_password() {
        let password = generate_random_password(16);
        assert_eq!(password.len(), 16);
        
        // Should pass validation
        assert!(validate_password_strength(&password).is_ok());
    }
}
