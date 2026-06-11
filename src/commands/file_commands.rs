use crate::models::ModelInfo;
use crate::state::get_state;
use tracing::info;

#[tauri::command]
pub async fn open_ifc_file(file_path: String) -> Result<ModelInfo, String> {
    eprintln!("[FILE_CMD] open_ifc_file called with: {}", file_path);
    
    // Check if already open and parsed - sync path
    let app = get_state();
    {
        let guard = app.model.lock().map_err(|e| e.to_string())?;
        if let Some(existing_path) = &guard.model_path {
            if existing_path == &file_path && !guard.entities.is_empty() {
                return Ok(ModelInfo {
                    path: file_path,
                    entity_count: guard.entities.len(),
                    project_name: None,
                });
            }
        }
    }
    
    let model_path = std::path::Path::new(&file_path);
    if !model_path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    
    // Parse IFC - async path
    let parsed_path = file_path.clone();
    let result = crate::services::parser::parse_and_index(&parsed_path).await;
    
    match result {
        Ok((metadata_count, project_name)) => {
            info!("Opened IFC file: {} ({} entities)", file_path, metadata_count);
            
            let app = get_state();
            let mut guard = app.model.lock().map_err(|e| e.to_string())?;
            guard.model_path = Some(file_path.clone());
            
            Ok(ModelInfo {
                path: file_path,
                entity_count: metadata_count,
                project_name,
            })
        }
        Err(e) => Err(format!("Failed to parse IFC file: {}", e)),
    }
}

#[tauri::command]
pub fn get_file_info(path: String) -> Option<serde_json::Value> {
    std::fs::metadata(&path).ok().map(|meta| {
        serde_json::json!({
            "path": path,
            "size": meta.len(),
            "is_file": meta.is_file(),
            "created": meta.created().ok().map(|t| t.elapsed().unwrap_or_default().as_secs() as f64).unwrap_or(0.0),
            "modified": meta.modified().ok().map(|t| t.elapsed().unwrap_or_default().as_secs() as f64).unwrap_or(0.0),
        })
    })
}
