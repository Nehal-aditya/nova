// NOVA Nebula Dependency Resolver
// Handles dependency resolution and manifest parsing

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a package manifest (nova.toml)
#[derive(Debug, Clone)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: String,
    pub dependencies: HashMap<String, String>,
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        
        Self::parse(&content)
    }
    
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut manifest = Manifest {
            name: String::new(),
            version: String::from("0.1.0"),
            authors: Vec::new(),
            description: String::new(),
            dependencies: HashMap::new(),
        };
        
        let mut current_section = None;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Check for section headers
            if line.starts_with('[') && line.ends_with(']') {
                current_section = Some(line[1..line.len()-1].to_string());
                continue;
            }
            
            // Parse key-value pairs
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim().trim_matches('"');
                
                match current_section.as_deref() {
                    Some("package") => {
                        match key {
                            "name" => manifest.name = value.to_string(),
                            "version" => manifest.version = value.to_string(),
                            "authors" => {
                                // Parse array format: ["Author1", "Author2"]
                                if value.starts_with('[') && value.ends_with(']') {
                                    let inner = &value[1..value.len()-1];
                                    manifest.authors = inner
                                        .split(',')
                                        .map(|s| s.trim().trim_matches('"').to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                }
                            }
                            "description" => manifest.description = value.to_string(),
                            _ => {}
                        }
                    }
                    Some("dependencies") => {
                        manifest.dependencies.insert(key.to_string(), value.to_string());
                    }
                    _ => {}
                }
            }
        }
        
        Ok(manifest)
    }
}

/// Resolves dependencies for a project
pub struct DependencyResolver {
    pub manifest: Manifest,
    pub resolved: HashMap<String, ResolvedDependency>,
}

#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: String,
    pub path: Option<PathBuf>,
}

impl DependencyResolver {
    pub fn new(manifest: Manifest) -> Self {
        DependencyResolver {
            manifest,
            resolved: HashMap::new(),
        }
    }
    
    /// Resolve all dependencies from the manifest
    pub fn resolve_all(&mut self) -> Result<(), String> {
        for (name, version_spec) in &self.manifest.dependencies {
            self.resolve_dependency(name, version_spec)?;
        }
        Ok(())
    }
    
    /// Resolve a single dependency
    fn resolve_dependency(&mut self, name: &str, version_spec: &str) -> Result<(), String> {
        // Check if already resolved
        if self.resolved.contains_key(name) {
            return Ok(());
        }
        
        // For now, we just record the dependency spec
        // In a full implementation, this would:
        // 1. Query the Nebula Registry
        // 2. Download the constellation
        // 3. Verify checksums
        // 4. Resolve transitive dependencies
        
        let resolved = ResolvedDependency {
            name: name.to_string(),
            version: version_spec.clone(),
            path: None, // Would be set to downloaded location
        };
        
        self.resolved.insert(name.to_string(), resolved);
        Ok(())
    }
    
    /// Get the list of resolved dependencies
    pub fn get_resolved(&self) -> Vec<&ResolvedDependency> {
        self.resolved.values().collect()
    }
}

/// Create a default manifest for a new project
pub fn create_default_manifest(project_name: &str) -> String {
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
authors = [""]
description = "A NOVA project"

[dependencies]
"#,
        project_name
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_minimal_manifest() {
        let content = r#"
[package]
name = "test_project"
version = "0.1.0"
"#;
        let manifest = Manifest::parse(content).unwrap();
        assert_eq!(manifest.name, "test_project");
        assert_eq!(manifest.version, "0.1.0");
    }
    
    #[test]
    fn test_parse_manifest_with_deps() {
        let content = r#"
[package]
name = "my_project"
version = "1.0.0"
authors = ["Alice", "Bob"]
description = "Test project"

[dependencies]
cosmos.stats = "0.1.0"
nova.fs = "latest"
"#;
        let manifest = Manifest::parse(content).unwrap();
        assert_eq!(manifest.name, "my_project");
        assert_eq!(manifest.dependencies.len(), 2);
        assert_eq!(manifest.dependencies.get("cosmos.stats"), Some(&"0.1.0".to_string()));
    }
}