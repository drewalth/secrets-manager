use clap::{Parser, Subcommand};
use anyhow::Result;
use rpassword::read_password;
use std::io::{self, Write};

use crate::models::{Project, ExportFormat};
use crate::storage::SecretStorage;

#[derive(Parser)]
#[command(name = "secrets-manager")]
#[command(about = "A secure local secrets manager for development")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new project
    Create {
        /// Name of the project
        project_name: String,
    },
    /// List all projects
    List,
    /// Add a secret to a project
    Add {
        /// Name of the project
        project_name: String,
        /// Secret key
        key: String,
        /// Secret value (if not provided, will prompt)
        value: Option<String>,
    },
    /// Remove a secret from a project
    Remove {
        /// Name of the project
        project_name: String,
        /// Secret key to remove
        key: String,
    },
    /// List secrets in a project
    Show {
        /// Name of the project
        project_name: String,
    },
    /// Export secrets in various formats
    Export {
        /// Name of the project
        project_name: String,
        /// Export format (shell, env, json)
        #[arg(short, long, default_value = "shell")]
        format: String,
        /// Output file (optional, defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Delete a project
    Delete {
        /// Name of the project
        project_name: String,
    },
}

pub struct SecretManager {
    storage: SecretStorage,
}

impl SecretManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            storage: SecretStorage::new()?,
        })
    }
    
    /// Prompts for password with confirmation
    fn get_password() -> Result<String> {
        print!("Enter master password: ");
        io::stdout().flush()?;
        let password = read_password()?;
        
        if password.is_empty() {
            return Err(anyhow::anyhow!("Password cannot be empty"));
        }
        
        Ok(password)
    }
    
    /// Prompts for password with confirmation for new projects
    fn get_password_with_confirmation() -> Result<String> {
        print!("Enter master password: ");
        io::stdout().flush()?;
        let password = read_password()?;
        
        if password.is_empty() {
            return Err(anyhow::anyhow!("Password cannot be empty"));
        }
        
        print!("Confirm master password: ");
        io::stdout().flush()?;
        let confirm_password = read_password()?;
        
        if password != confirm_password {
            return Err(anyhow::anyhow!("Passwords do not match"));
        }
        
        Ok(password)
    }
    
    /// Prompts for secret value
    fn get_secret_value(key: &str) -> Result<String> {
        print!("Enter value for '{}': ", key);
        io::stdout().flush()?;
        let value = read_password()?;
        Ok(value)
    }
    
    pub fn handle_command(&self, command: Commands) -> Result<()> {
        match command {
            Commands::Create { project_name } => {
                self.create_project(&project_name)?;
            }
            Commands::List => {
                self.list_projects()?;
            }
            Commands::Add { project_name, key, value } => {
                self.add_secret(&project_name, &key, value)?;
            }
            Commands::Remove { project_name, key } => {
                self.remove_secret(&project_name, &key)?;
            }
            Commands::Show { project_name } => {
                self.show_project(&project_name)?;
            }
            Commands::Export { project_name, format, output } => {
                self.export_project(&project_name, &format, output)?;
            }
            Commands::Delete { project_name } => {
                self.delete_project(&project_name)?;
            }
        }
        Ok(())
    }
    
    fn create_project(&self, project_name: &str) -> Result<()> {
        if self.storage.project_exists(project_name) {
            return Err(anyhow::anyhow!("Project '{}' already exists", project_name));
        }
        
        let password = Self::get_password_with_confirmation()?;
        let project = Project::new(project_name.to_string());
        self.storage.save_project(&project, &password)?;
        
        println!("✅ Project '{}' created successfully!", project_name);
        Ok(())
    }
    
    fn list_projects(&self) -> Result<()> {
        let projects = self.storage.list_projects()?;
        
        if projects.is_empty() {
            println!("No projects found. Create one with: secrets-manager create <project-name>");
            return Ok(());
        }
        
        println!("📁 Available projects:");
        for project in projects {
            println!("  • {}", project);
        }
        Ok(())
    }
    
    fn add_secret(&self, project_name: &str, key: &str, value: Option<String>) -> Result<()> {
        let password = Self::get_password()?;
        let mut project = self.storage.load_project(project_name, &password)?;
        
        let secret_value = match value {
            Some(v) => v,
            None => Self::get_secret_value(key)?,
        };
        
        project.add_secret(key.to_string(), secret_value);
        self.storage.save_project(&project, &password)?;
        
        println!("✅ Secret '{}' added to project '{}'", key, project_name);
        Ok(())
    }
    
    fn remove_secret(&self, project_name: &str, key: &str) -> Result<()> {
        let password = Self::get_password()?;
        let mut project = self.storage.load_project(project_name, &password)?;
        
        if project.remove_secret(key).is_some() {
            self.storage.save_project(&project, &password)?;
            println!("✅ Secret '{}' removed from project '{}'", key, project_name);
        } else {
            println!("❌ Secret '{}' not found in project '{}'", key, project_name);
        }
        Ok(())
    }
    
    fn show_project(&self, project_name: &str) -> Result<()> {
        let password = Self::get_password()?;
        let project = self.storage.load_project(project_name, &password)?;
        
        println!("🔐 Project: {}", project.name);
        println!("📅 Created: {}", project.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("📅 Updated: {}", project.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
        println!();
        
        if project.secrets.is_empty() {
            println!("No secrets found. Add one with: secrets-manager add {} <key>", project_name);
        } else {
            println!("Secrets:");
            for key in project.list_secrets() {
                println!("  • {}", key);
            }
        }
        Ok(())
    }
    
    fn export_project(&self, project_name: &str, format: &str, output: Option<String>) -> Result<()> {
        let password = Self::get_password()?;
        let project = self.storage.load_project(project_name, &password)?;
        
        let export_format = match format.to_lowercase().as_str() {
            "shell" => ExportFormat::Shell,
            "env" => ExportFormat::EnvFile,
            "json" => ExportFormat::Json,
            _ => return Err(anyhow::anyhow!("Invalid format. Use: shell, env, or json")),
        };
        
        let content = self.format_export(&project, &export_format)?;
        
        match output {
            Some(file_path) => {
                std::fs::write(&file_path, content)?;
                println!("✅ Exported to: {}", file_path);
            }
            None => {
                print!("{}", content);
            }
        }
        Ok(())
    }
    
    fn format_export(&self, project: &Project, format: &ExportFormat) -> Result<String> {
        match format {
            ExportFormat::Shell => {
                let mut output = String::new();
                for (key, value) in &project.secrets {
                    output.push_str(&format!("export {}='{}'\n", key, value));
                }
                Ok(output)
            }
            ExportFormat::EnvFile => {
                let mut output = String::new();
                for (key, value) in &project.secrets {
                    output.push_str(&format!("{}={}\n", key, value));
                }
                Ok(output)
            }
            ExportFormat::Json => {
                serde_json::to_string_pretty(&project.secrets).map_err(|e| e.into())
            }
        }
    }
    
    fn delete_project(&self, project_name: &str) -> Result<()> {
        if !self.storage.project_exists(project_name) {
            return Err(anyhow::anyhow!("Project '{}' not found", project_name));
        }
        
        print!("⚠️  Are you sure you want to delete project '{}'? (y/N): ", project_name);
        io::stdout().flush()?;
        
        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;
        
        if confirmation.trim().to_lowercase() == "y" || confirmation.trim().to_lowercase() == "yes" {
            self.storage.delete_project(project_name)?;
            println!("✅ Project '{}' deleted successfully!", project_name);
        } else {
            println!("❌ Deletion cancelled");
        }
        Ok(())
    }
}
