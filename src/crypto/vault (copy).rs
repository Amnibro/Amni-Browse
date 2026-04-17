use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, AeadCore,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::fs;
use uuid::Uuid;

use crate::storage::config::BrowserConfig;

const PBKDF2_ITERATIONS: u32 = 600_000;
const SALT_LEN: usize = 32;
const KEY_LEN: usize = 32;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: String,
    pub site: String,
    pub username: String,
    pub encrypted_password: String,
    pub nonce: String,
    pub created_at: String,
    pub updated_at: String,
    pub notes: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordVault {
    pub salt: String,
    pub verify_hash: String,
    pub verify_nonce: String,
    pub credentials: Vec<Credential>,
}

impl Default for PasswordVault {
    fn default() -> Self {
        Self {
            salt: String::new(),
            verify_hash: String::new(),
            verify_nonce: String::new(),
            credentials: Vec::new(),
        }
    }
}

pub struct PasswordManager {
    vault: PasswordVault,
    dk: Option<[u8; KEY_LEN]>,
    is_unlocked: bool,
}
impl PasswordManager {
    pub fn new() -> Self {
        let vault = Self::load_vault();
        Self { vault, dk: None, is_unlocked: false }
    }
    pub fn derived_key(&self) -> Option<[u8; KEY_LEN]> { self.dk }

    fn vault_path() -> std::path::PathBuf {
        BrowserConfig::config_dir().join("vault.enc.json")
    }

    fn load_vault() -> PasswordVault {
        let path = Self::vault_path();
        if path.exists() {
            let data = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            PasswordVault::default()
        }
    }

    fn save_vault(&self) {
        let path = Self::vault_path();
        if let Ok(data) = serde_json::to_string_pretty(&self.vault) {
            fs::write(&path, data).ok();
        }
    }

    fn derive_key(master_password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
        let mut key = [0u8; KEY_LEN];
        pbkdf2_hmac::<Sha256>(master_password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
        key
    }

    pub fn initialize(&mut self, master_password: &str) -> Result<(), String> {
        if !self.vault.salt.is_empty() {
            return Err("Vault already initialized. Use unlock instead.".to_string());
        }

        let mut salt = [0u8; SALT_LEN];
        rand::rngs::OsRng.fill_bytes(&mut salt);

        let key = Self::derive_key(master_password, &salt);

        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| format!("Cipher init failed: {}", e))?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let verify_plaintext = b"AMNI_VAULT_VERIFY_v1";
        let ciphertext = cipher
            .encrypt(&nonce, verify_plaintext.as_ref())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        self.vault.salt = BASE64.encode(salt);
        self.vault.verify_hash = BASE64.encode(&ciphertext);
        self.vault.verify_nonce = BASE64.encode(nonce.as_slice());
        self.dk = Some(key);
        self.is_unlocked = true;

        self.save_vault();
        Ok(())
    }

    pub fn unlock(&mut self, master_password: &str) -> Result<(), String> {
        if self.vault.salt.is_empty() {
            return Err("Vault not initialized. Create a master password first.".to_string());
        }

        let salt = BASE64
            .decode(&self.vault.salt)
            .map_err(|_| "Corrupt vault: invalid salt".to_string())?;

        let key = Self::derive_key(master_password, &salt);

        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| format!("Cipher init failed: {}", e))?;

        let nonce_bytes = BASE64
            .decode(&self.vault.verify_nonce)
            .map_err(|_| "Corrupt vault: invalid nonce".to_string())?;
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);

        let ciphertext = BASE64
            .decode(&self.vault.verify_hash)
            .map_err(|_| "Corrupt vault: invalid verify hash".to_string())?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| "Wrong master password.".to_string())?;

        if plaintext != b"AMNI_VAULT_VERIFY_v1" {
            return Err("Wrong master password.".to_string());
        }

        self.dk = Some(key);
        self.is_unlocked = true;
        Ok(())
    }
    pub fn lock(&mut self) {
        if let Some(ref mut key) = self.dk {
            key.fill(0);
        }
        self.dk = None;
        self.is_unlocked = false;
    }

    pub fn is_unlocked(&self) -> bool {
        self.is_unlocked
    }

    pub fn is_initialized(&self) -> bool {
        !self.vault.salt.is_empty()
    }

    pub fn add_credential(
        &mut self,
        site: &str,
        username: &str,
        password: &str,
        notes: Option<&str>,
        category: Option<&str>,
    ) -> Result<String, String> {
        let key = self.dk.ok_or("Vault is locked. Unlock first.")?;

        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| format!("Cipher init failed: {}", e))?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, password.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let now = chrono::Utc::now().to_rfc3339();
        let cred = Credential {
            id: Uuid::new_v4().to_string(),
            site: site.to_string(),
            username: username.to_string(),
            encrypted_password: BASE64.encode(&ciphertext),
            nonce: BASE64.encode(nonce.as_slice()),
            created_at: now.clone(),
            updated_at: now,
            notes: notes.map(|n| n.to_string()),
            category: category.map(|c| c.to_string()),
        };

        let id = cred.id.clone();
        self.vault.credentials.push(cred);
        self.save_vault();
        Ok(id)
    }

    pub fn get_password(&self, credential_id: &str) -> Result<String, String> {
        let key = self.dk.ok_or("Vault is locked. Unlock first.")?;
        let cred = self.vault.credentials.iter().find(|c| c.id == credential_id).ok_or("Credential not found.")?;

        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| format!("Cipher init failed: {}", e))?;

        let nonce_bytes = BASE64
            .decode(&cred.nonce)
            .map_err(|_| "Corrupt credential: invalid nonce".to_string())?;
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);

        let ciphertext = BASE64
            .decode(&cred.encrypted_password)
            .map_err(|_| "Corrupt credential: invalid ciphertext".to_string())?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| "Decryption failed â€” vault may be corrupt.".to_string())?;

        String::from_utf8(plaintext).map_err(|_| "Decrypted data is not valid UTF-8".to_string())
    }

    pub fn remove_credential(&mut self, credential_id: &str) -> bool {
        let before = self.vault.credentials.len();
        self.vault.credentials.retain(|c| c.id != credential_id);
        let removed = self.vault.credentials.len() < before;
        if removed {
            self.save_vault();
        }
        removed
    }

    pub fn list_credentials(&self) -> Vec<CredentialSummary> {
        self.vault
            .credentials
            .iter()
            .map(|c| CredentialSummary {
                id: c.id.clone(),
                site: c.site.clone(),
                username: c.username.clone(),
                category: c.category.clone(),
                created_at: c.created_at.clone(),
            })
            .collect()
    }

    pub fn search_credentials(&self, query: &str) -> Vec<CredentialSummary> {
        let q = query.to_lowercase();
        self.vault
            .credentials
            .iter()
            .filter(|c| {
                c.site.to_lowercase().contains(&q) || c.username.to_lowercase().contains(&q)
            })
            .map(|c| CredentialSummary {
                id: c.id.clone(),
                site: c.site.clone(),
                username: c.username.clone(),
                category: c.category.clone(),
                created_at: c.created_at.clone(),
            })
            .collect()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.list_credentials()).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn generate_password(length: usize) -> String {
        const CHARSET: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+[]{}|;:,.<>?";
        let mut rng = rand::rngs::OsRng;
        let password: String = (0..length.max(12).min(128))
            .map(|_| {
                let idx = (rng.next_u32() as usize) % CHARSET.len();
                CHARSET[idx] as char
            })
            .collect();
        password
    }

    pub fn wipe_vault(&mut self) {
        self.lock();
        self.vault = PasswordVault::default();
        let path = Self::vault_path();
        fs::remove_file(&path).ok();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSummary {
    pub id: String,
    pub site: String,
    pub username: String,
    pub category: Option<String>,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation_is_deterministic() {
        let salt = [42u8; SALT_LEN];
        let key1 = PasswordManager::derive_key("test_password", &salt);
        let key2 = PasswordManager::derive_key("test_password", &salt);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_different_passwords_produce_different_keys() {
        let salt = [42u8; SALT_LEN];
        let key1 = PasswordManager::derive_key("password_a", &salt);
        let key2 = PasswordManager::derive_key("password_b", &salt);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_generate_password_length() {
        let pwd = PasswordManager::generate_password(20);
        assert_eq!(pwd.len(), 20);
    }

    #[test]
    fn test_generate_password_minimum_length() {
        let pwd = PasswordManager::generate_password(4);
        assert_eq!(pwd.len(), 12); // Minimum enforced
    }
}
