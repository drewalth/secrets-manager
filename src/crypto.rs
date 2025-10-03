use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, AeadCore, KeyInit};
use base64::{Engine as _, engine::general_purpose};
use rand::{RngCore, rngs::OsRng};
use anyhow::{Result, anyhow};

use crate::models::{Project, EncryptedProject};

/// Derives a key from a password using PBKDF2
fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;
    
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);
    Ok(key)
}

/// Encrypts a project with the given password
pub fn encrypt_project(project: &Project, password: &str) -> Result<EncryptedProject> {
    // Generate random salt and nonce
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    // Derive key from password and salt
    let key_bytes = derive_key(password, &salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    
    // Serialize project to JSON
    let json_data = serde_json::to_string(project)?;
    
    // Encrypt the data
    let ciphertext = cipher.encrypt(&nonce, json_data.as_bytes())
        .map_err(|_| anyhow!("Encryption failed"))?;
    
    Ok(EncryptedProject {
        encrypted_data: general_purpose::STANDARD.encode(&ciphertext),
        salt: general_purpose::STANDARD.encode(&salt),
        nonce: general_purpose::STANDARD.encode(&nonce),
    })
}

/// Decrypts a project with the given password
pub fn decrypt_project(encrypted: &EncryptedProject, password: &str) -> Result<Project> {
    // Decode base64 data
    let salt = general_purpose::STANDARD.decode(&encrypted.salt)?;
    let nonce_bytes = general_purpose::STANDARD.decode(&encrypted.nonce)?;
    let ciphertext = general_purpose::STANDARD.decode(&encrypted.encrypted_data)?;
    
    // Derive key from password and salt
    let key_bytes = derive_key(password, &salt)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Decrypt the data
    let plaintext = cipher.decrypt(nonce, ciphertext.as_slice())
        .map_err(|_| anyhow!("Decryption failed - wrong password or corrupted data"))?;
    
    // Deserialize back to Project
    let project: Project = serde_json::from_slice(&plaintext)?;
    Ok(project)
}

/// Validates a password by attempting to decrypt a test project
pub fn validate_password(encrypted: &EncryptedProject, password: &str) -> bool {
    decrypt_project(encrypted, password).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let mut project = Project::new("test_project".to_string());
        project.add_secret("API_KEY".to_string(), "secret123".to_string());
        project.add_secret("DB_URL".to_string(), "postgres://localhost".to_string());
        
        let password = "test_password";
        let encrypted = encrypt_project(&project, password).unwrap();
        let decrypted = decrypt_project(&encrypted, password).unwrap();
        
        assert_eq!(project.name, decrypted.name);
        assert_eq!(project.secrets, decrypted.secrets);
    }
    
    #[test]
    fn test_wrong_password() {
        let project = Project::new("test_project".to_string());
        let password = "correct_password";
        let encrypted = encrypt_project(&project, password).unwrap();
        
        assert!(decrypt_project(&encrypted, "wrong_password").is_err());
    }
}
