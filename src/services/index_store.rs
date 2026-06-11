use crate::models::EntitySnapshot;
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

pub struct IndexStore {
    db_path: String,
}

impl IndexStore {
    pub fn new(file_path: &str) -> Self {
        let path = Path::new(file_path);
        let idx_path = path.parent()
            .map(|d| d.join(format!("{}.idx", path.file_stem().unwrap_or_default().to_string_lossy())))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("{}.idx", file_path));
        
        Self { db_path: idx_path }
    }

    pub fn exists(&self) -> bool {
        std::path::Path::new(&self.db_path).exists()
    }

    pub fn create(&self, entities: &HashMap<u64, EntitySnapshot>) -> Result<(), String> {
        info!("Creating index with {} entities at {}", entities.len(), self.db_path);
        
        // Write as simple JSON for now; upgrade to SQLite in phase 2
        let json = serde_json::to_string_pretty(&entities)
            .map_err(|e| format!("Failed to serialize entities: {}", e))?;
        
        std::fs::write(&self.db_path, json)
            .map_err(|e| format!("Failed to write index file: {}", e))?;
        
        Ok(())
    }

    pub fn load() -> Result<HashMap<u64, EntitySnapshot>, String> {
        if !std::path::Path::new(&Self::default_path()).exists() {
            return Ok(HashMap::new());
        }
        
        let json = std::fs::read_to_string(Self::default_path())
            .map_err(|e| format!("Failed to read index file: {}", e))?;
        
        serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize index: {}", e))
    }

    fn default_path() -> String {
        "default.idx".to_string()
    }
}
