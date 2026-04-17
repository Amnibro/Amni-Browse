use aes_gcm::{aead::{Aead, KeyInit, OsRng}, Aes256Gcm, AeadCore};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;
use crate::storage::config::BrowserConfig;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressProfile {
    pub id: String,
    pub label: String,
    pub full_name: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
    pub phone: String,
    pub email: String,
}
impl AddressProfile {
    pub fn new(label: &str, name: &str, street: &str, city: &str, state: &str, zip: &str, country: &str, phone: &str, email: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(), label: label.into(), full_name: name.into(),
            street: street.into(), city: city.into(), state: state.into(), zip: zip.into(),
            country: country.into(), phone: phone.into(), email: email.into(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentCard {
    pub id: String,
    pub label: String,
    pub cardholder: String,
    pub enc_number: String,
    pub nonce: String,
    pub expiry: String,
    pub card_type: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutofillStore {
    pub addresses: Vec<AddressProfile>,
    pub cards: Vec<PaymentCard>,
}
pub struct AutofillManager {
    store: AutofillStore,
    enc_key: Option<[u8; 32]>,
}
impl AutofillManager {
    pub fn new() -> Self {
        Self { store: Self::load(), enc_key: None }
    }
    fn file_path() -> PathBuf { BrowserConfig::config_dir().join("autofill.json") }
    fn load() -> AutofillStore {
        let path = Self::file_path();
        path.exists().then(|| {
            fs::read_to_string(&path).ok().and_then(|d| serde_json::from_str(&d).ok())
        }).flatten().unwrap_or_default()
    }
    fn save(&self) {
        let path = Self::file_path();
        serde_json::to_string_pretty(&self.store).ok().map(|d| fs::write(&path, d).ok());
    }
    pub fn set_encryption_key(&mut self, key: [u8; 32]) { self.enc_key = Some(key); }
    pub fn add_address(&mut self, addr: AddressProfile) -> String {
        let id = addr.id.clone();
        self.store.addresses.push(addr);
        self.save();
        id
    }
    pub fn remove_address(&mut self, id: &str) {
        self.store.addresses.retain(|a| a.id != id);
        self.save();
    }
    pub fn list_addresses(&self) -> &[AddressProfile] { &self.store.addresses }
    pub fn add_card(&mut self, label: &str, cardholder: &str, number: &str, expiry: &str, card_type: &str) -> Result<String, String> {
        let key = self.enc_key.ok_or("Vault must be unlocked to add cards")?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Cipher err: {}", e))?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ct = cipher.encrypt(&nonce, number.as_bytes()).map_err(|e| format!("Encrypt err: {}", e))?;
        let card = PaymentCard {
            id: Uuid::new_v4().to_string(),
            label: label.into(), cardholder: cardholder.into(),
            enc_number: BASE64.encode(&ct), nonce: BASE64.encode(nonce.as_slice()),
            expiry: expiry.into(), card_type: card_type.into(),
        };
        let id = card.id.clone();
        self.store.cards.push(card);
        self.save();
        Ok(id)
    }
    pub fn decrypt_card_number(&self, card_id: &str) -> Result<String, String> {
        let key = self.enc_key.ok_or("Vault must be unlocked")?;
        let card = self.store.cards.iter().find(|c| c.id == card_id).ok_or("Card not found")?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Cipher err: {}", e))?;
        let nonce_bytes = BASE64.decode(&card.nonce).map_err(|_| "Invalid nonce")?;
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);
        let ct = BASE64.decode(&card.enc_number).map_err(|_| "Invalid ciphertext")?;
        let pt = cipher.decrypt(nonce, ct.as_ref()).map_err(|_| "Decryption failed")?;
        String::from_utf8(pt).map_err(|_| "Invalid UTF-8".into())
    }
    pub fn remove_card(&mut self, id: &str) {
        self.store.cards.retain(|c| c.id != id);
        self.save();
    }
    pub fn list_cards(&self) -> Vec<CardSummary> {
        self.store.cards.iter().map(|c| CardSummary {
            id: c.id.clone(), label: c.label.clone(), cardholder: c.cardholder.clone(),
            last_four: c.enc_number.chars().rev().take(4).collect::<String>().chars().rev().collect(),
            expiry: c.expiry.clone(), card_type: c.card_type.clone(),
        }).collect()
    }
    pub fn suggest_for_site(&self, _url: &str) -> AutofillSuggestion {
        AutofillSuggestion {
            addresses: self.store.addresses.clone(),
            cards: self.list_cards(),
        }
    }
    pub fn addresses_json(&self) -> String {
        serde_json::to_string(&self.store.addresses).unwrap_or_else(|_| "[]".into())
    }
    pub fn cards_json(&self) -> String {
        serde_json::to_string(&self.list_cards()).unwrap_or_else(|_| "[]".into())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardSummary {
    pub id: String,
    pub label: String,
    pub cardholder: String,
    pub last_four: String,
    pub expiry: String,
    pub card_type: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutofillSuggestion {
    pub addresses: Vec<AddressProfile>,
    pub cards: Vec<CardSummary>,
}
