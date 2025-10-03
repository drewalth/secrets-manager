use clap::{Parser, Subcommand};
use anyhow::Result;
use rpassword::read_password;
use std::io::{self, Write};
use std::fs;
use std::path::Path;

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
        /// Force export (skip .gitignore check)
        #[arg(short = 'F', long)]
        force: bool,
    },
    /// Delete a project
    Delete {
        /// Name of the project
        project_name: String,
    },
    /// Import secrets from a .env file
    Import {
        /// Name of the project
        project_name: String,
        /// Path to the .env file
        env_file: String,
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
            Commands::Export { project_name, format, output, force } => {
                self.export_project(&project_name, &format, output, force)?;
            }
            Commands::Delete { project_name } => {
                self.delete_project(&project_name)?;
            }
            Commands::Import { project_name, env_file } => {
                self.import_project(&project_name, &env_file)?;
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
    
    fn export_project(&self, project_name: &str, format: &str, output: Option<String>, force: bool) -> Result<()> {
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
                // Check .gitignore guardrail before writing file
                if force {
                    println!("⚠️  WARNING: Exporting to '{}' without checking .gitignore!", file_path);
                    println!("   This may result in accidental commits.");
                    println!();
                } else {
                    self.check_gitignore_guardrail(&file_path)?;
                }
                
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
    
    fn import_project(&self, project_name: &str, env_file: &str) -> Result<()> {
        // Check if the .env file exists
        if !Path::new(env_file).exists() {
            return Err(anyhow::anyhow!("File '{}' not found", env_file));
        }

        // Load the project
        let password = Self::get_password()?;
        let mut project = self.storage.load_project(project_name, &password)?;

        // Parse the .env file
        let env_content = fs::read_to_string(env_file)?;
        let env_vars = self.parse_env_file(&env_content)?;

        if env_vars.is_empty() {
            println!("No environment variables found in '{}'", env_file);
            return Ok(());
        }

        println!("Found {} environment variables in '{}'", env_vars.len(), env_file);
        
        let mut imported_count = 0;
        let mut skipped_count = 0;

        for (key, value) in env_vars {
            if project.get_secret(&key).is_some() {
                // Key already exists, prompt for confirmation
                print!("Key '{}' already exists. Overwrite? (y/N): ", key);
                io::stdout().flush()?;
                
                let mut confirmation = String::new();
                io::stdin().read_line(&mut confirmation)?;
                
                if confirmation.trim().to_lowercase() == "y" || confirmation.trim().to_lowercase() == "yes" {
                    project.add_secret(key.clone(), value);
                    imported_count += 1;
                    println!("✅ Imported '{}'", key);
                } else {
                    skipped_count += 1;
                    println!("⏭️  Skipped '{}'", key);
                }
            } else {
                // Key doesn't exist, add it directly
                project.add_secret(key.clone(), value);
                imported_count += 1;
                println!("✅ Imported '{}'", key);
            }
        }

        // Save the updated project
        self.storage.save_project(&project, &password)?;

        println!();
        println!("📊 Import Summary:");
        println!("  • Imported: {}", imported_count);
        println!("  • Skipped: {}", skipped_count);
        println!("  • Total processed: {}", imported_count + skipped_count);

        Ok(())
    }

    /// Parses a .env file content and returns a HashMap of key-value pairs
    fn parse_env_file(&self, content: &str) -> Result<std::collections::HashMap<String, String>> {
        let mut env_vars = std::collections::HashMap::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Find the first '=' character
            if let Some(equal_pos) = line.find('=') {
                let key = line[..equal_pos].trim().to_string();
                let value = line[equal_pos + 1..].trim().to_string();
                
                // Remove quotes if present
                let value = if (value.starts_with('"') && value.ends_with('"')) || 
                              (value.starts_with('\'') && value.ends_with('\'')) {
                    value[1..value.len()-1].to_string()
                } else {
                    value
                };
                
                if !key.is_empty() {
                    env_vars.insert(key, value);
                }
            }
        }
        
        Ok(env_vars)
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

    /// Checks if the output file is properly ignored by .gitignore to prevent accidental commits
    fn check_gitignore_guardrail(&self, file_path: &str) -> Result<()> {
        let gitignore_path = ".gitignore";
        
        // Check if .gitignore exists
        if !Path::new(gitignore_path).exists() {
            println!("⚠️  WARNING: No .gitignore file found in current directory!");
            println!("   Exporting secrets to '{}' may result in accidental commits.", file_path);
            println!("   Consider creating a .gitignore file and adding this file pattern.");
            println!();
            
            print!("Do you want to continue with the export? (y/N): ");
            io::stdout().flush()?;
            
            let mut confirmation = String::new();
            io::stdin().read_line(&mut confirmation)?;
            
            if confirmation.trim().to_lowercase() != "y" && confirmation.trim().to_lowercase() != "yes" {
                return Err(anyhow::anyhow!("Export cancelled by user"));
            }
            return Ok(());
        }
        
        // Read and parse .gitignore
        let gitignore_content = match fs::read_to_string(gitignore_path) {
            Ok(content) => content,
            Err(_) => {
                println!("⚠️  WARNING: Could not read .gitignore file!");
                return self.prompt_for_unsafe_export(file_path);
            }
        };
        
        // Check if the file is ignored
        if !self.is_file_ignored(file_path, &gitignore_content) {
            println!("⚠️  WARNING: '{}' is not listed in .gitignore!", file_path);
            println!("   This file may be accidentally committed to version control.");
            println!("   Consider adding this file pattern to your .gitignore file.");
            println!();
            
            return self.prompt_for_unsafe_export(file_path);
        }
        
        // File is properly ignored, safe to proceed
        Ok(())
    }
    
    /// Prompts user for confirmation when exporting to an unsafe location
    fn prompt_for_unsafe_export(&self, _file_path: &str) -> Result<()> {
        print!("Do you want to continue with the export? (y/N): ");
        io::stdout().flush()?;
        
        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;
        
        if confirmation.trim().to_lowercase() != "y" && confirmation.trim().to_lowercase() != "yes" {
            return Err(anyhow::anyhow!("Export cancelled by user"));
        }
        
        Ok(())
    }
    
    /// Checks if a file path matches any pattern in .gitignore
    fn is_file_ignored(&self, file_path: &str, gitignore_content: &str) -> bool {
        let file_path = file_path.trim_start_matches("./");
        
        for line in gitignore_content.lines() {
            let pattern = line.trim();
            
            // Skip empty lines and comments
            if pattern.is_empty() || pattern.starts_with('#') {
                continue;
            }
            
            // Handle directory patterns (ending with /)
            if pattern.ends_with('/') {
                let dir_pattern = &pattern[..pattern.len() - 1];
                if file_path.starts_with(dir_pattern) {
                    return true;
                }
            }
            
            // Handle exact matches
            if pattern == file_path {
                return true;
            }
            
            // Handle wildcard patterns
            if self.matches_wildcard_pattern(file_path, pattern) {
                return true;
            }
            
            // Handle patterns that match any file with that name
            if pattern.starts_with('*') && file_path.ends_with(&pattern[1..]) {
                return true;
            }
        }
        
        false
    }
    
    /// Simple wildcard pattern matching (supports * and ?)
    fn matches_wildcard_pattern(&self, text: &str, pattern: &str) -> bool {
        // Convert glob pattern to a simple regex-like matching
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();
        
        self.wildcard_match(&text_chars, &pattern_chars, 0, 0)
    }
    
    /// Recursive wildcard matching helper
    fn wildcard_match(&self, text: &[char], pattern: &[char], text_idx: usize, pattern_idx: usize) -> bool {
        if pattern_idx == pattern.len() {
            return text_idx == text.len();
        }
        
        if text_idx == text.len() {
            return pattern[pattern_idx..].iter().all(|&c| c == '*');
        }
        
        match pattern[pattern_idx] {
            '*' => {
                // Try matching 0 or more characters
                self.wildcard_match(text, pattern, text_idx, pattern_idx + 1) ||
                self.wildcard_match(text, pattern, text_idx + 1, pattern_idx)
            }
            '?' => {
                // Match any single character
                self.wildcard_match(text, pattern, text_idx + 1, pattern_idx + 1)
            }
            c => {
                // Match exact character
                text[text_idx] == c && self.wildcard_match(text, pattern, text_idx + 1, pattern_idx + 1)
            }
        }
    }
}
