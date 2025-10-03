use std::path::PathBuf;
use std::fs;
use anyhow::{Result, anyhow};
use dirs;

use crate::models::{Project, EncryptedProject};
use crate::crypto::{encrypt_project, decrypt_project};

/// Manages encrypted storage of projects
pub struct SecretStorage {
    storage_dir: PathBuf,
}

impl SecretStorage {
    /// Creates a new SecretStorage instance
    pub fn new() -> Result<Self> {
        let storage_dir = Self::get_storage_dir()?;
        fs::create_dir_all(&storage_dir)?;
        Ok(Self { storage_dir })
    }
    
    /// Gets the storage directory path
    fn get_storage_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?;
        Ok(home_dir.join(".secrets_manager"))
    }
    
    /// Gets the file path for a project
    fn get_project_path(&self, project_name: &str) -> PathBuf {
        self.storage_dir.join(format!("{}.encrypted", project_name))
    }
    
    /// Saves a project with encryption
    pub fn save_project(&self, project: &Project, password: &str) -> Result<()> {
        let encrypted = encrypt_project(project, password)?;
        let project_path = self.get_project_path(&project.name);
        
        let json_data = serde_json::to_string_pretty(&encrypted)?;
        fs::write(project_path, json_data)?;
        Ok(())
    }
    
    /// Loads a project with decryption
    pub fn load_project(&self, project_name: &str, password: &str) -> Result<Project> {
        let project_path = self.get_project_path(project_name);
        
        if !project_path.exists() {
            return Err(anyhow!("Project '{}' not found", project_name));
        }
        
        let json_data = fs::read_to_string(project_path)?;
        let encrypted: EncryptedProject = serde_json::from_str(&json_data)?;
        
        decrypt_project(&encrypted, password)
    }
    
    /// Lists all available projects
    pub fn list_projects(&self) -> Result<Vec<String>> {
        let mut projects = Vec::new();
        
        if !self.storage_dir.exists() {
            return Ok(projects);
        }
        
        for entry in fs::read_dir(&self.storage_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("encrypted") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    projects.push(stem.to_string());
                }
            }
        }
        
        projects.sort();
        Ok(projects)
    }
    
    /// Deletes a project
    pub fn delete_project(&self, project_name: &str) -> Result<()> {
        let project_path = self.get_project_path(project_name);
        
        if !project_path.exists() {
            return Err(anyhow!("Project '{}' not found", project_name));
        }
        
        fs::remove_file(project_path)?;
        Ok(())
    }
    
    /// Checks if a project exists
    pub fn project_exists(&self, project_name: &str) -> bool {
        self.get_project_path(project_name).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_project() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SecretStorage {
            storage_dir: temp_dir.path().to_path_buf(),
        };
        
        let mut project = Project::new("test_project".to_string());
        project.add_secret("API_KEY".to_string(), "secret123".to_string());
        
        let password = "test_password";
        storage.save_project(&project, password).unwrap();
        
        let loaded = storage.load_project("test_project", password).unwrap();
        assert_eq!(project.name, loaded.name);
        assert_eq!(project.secrets, loaded.secrets);
    }
    
    #[test]
    fn test_list_projects() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SecretStorage {
            storage_dir: temp_dir.path().to_path_buf(),
        };
        
        let project1 = Project::new("project1".to_string());
        let project2 = Project::new("project2".to_string());
        
        storage.save_project(&project1, "password").unwrap();
        storage.save_project(&project2, "password").unwrap();
        
        let projects = storage.list_projects().unwrap();
        assert_eq!(projects.len(), 2);
        assert!(projects.contains(&"project1".to_string()));
        assert!(projects.contains(&"project2".to_string()));
    }
}
