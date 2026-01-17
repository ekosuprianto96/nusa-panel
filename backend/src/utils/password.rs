//! # Password Utilities
//!
//! Modul untuk hashing dan verifikasi password.
//! Menggunakan Argon2id - algoritma yang direkomendasikan untuk password hashing.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "TestP@ssword123";
        let hash = hash_password(password).expect("Failed to hash password");

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
