use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a project with its associated secrets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub secrets: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Project {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            name,
            secrets: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_secret(&mut self, key: String, value: String) {
        self.secrets.insert(key, value);
        self.updated_at = chrono::Utc::now();
    }

    pub fn remove_secret(&mut self, key: &str) -> Option<String> {
        let result = self.secrets.remove(key);
        if result.is_some() {
            self.updated_at = chrono::Utc::now();
        }
        result
    }

    pub fn get_secret(&self, key: &str) -> Option<&String> {
        self.secrets.get(key)
    }

    pub fn list_secrets(&self) -> Vec<&String> {
        self.secrets.keys().collect()
    }
}

/// Encrypted data structure for storage
#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedProject {
    pub encrypted_data: String,
    pub salt: String,
    pub nonce: String,
}

/// Export format options
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Shell,
    EnvFile,
    Json,
}
