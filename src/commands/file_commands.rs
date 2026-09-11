use crate::models::ModelInfo;
use crate::state::get_state;
use crate::services::ifc_openshell::IfcModelWrapper;
use tracing::info;

#[tauri::command]
pub async fn open_ifc_file(file_path: String) -> Result<ModelInfo, String> {
    eprintln!("[FILE_CMD] open_ifc_file called with: {}", file_path);

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

    let parsed_path = file_path.clone();
    let result = crate::services::parser::parse_and_index(&parsed_path).await;

    match result {
        Ok((metadata_count, project_name)) => {
            info!("Opened IFC file: {} ({} entities)", file_path, metadata_count);

            let app = get_state();
            let mut guard = app.model.lock().map_err(|e| e.to_string())?;
            guard.model_path = Some(file_path.clone());

            match IfcModelWrapper::open(&file_path) {
                Ok(ifc_model) => {
                    eprintln!("[FILE_CMD] IfcOpenShell opened file successfully, entity count: {}", ifc_model.entity_count());
                    let entity_ids: Vec<u64> = guard.entities.keys().cloned().collect();
                    for id in entity_ids {
                        let schema_attrs = ifc_model.entity_attributes(id);
                        if !schema_attrs.is_empty() {
                            if let Some(entity) = guard.entities.get_mut(&id) {
                                entity.attributes = entity.attributes
                                    .iter()
                                    .enumerate()
                                    .map(|(pos, (_, val))| {
                                        let new_key = schema_attrs.get(pos).map(|(n, _)| n.clone()).unwrap_or_else(|| format!("attr_{}", pos));
                                        (new_key, val.clone())
                                    })
                                    .collect();
                            }
                        }
                    }
                    guard.ifc_model = Some(ifc_model);
                }
                Err(e) => {
                    eprintln!("[FILE_CMD] IfcOpenShell failed to open file: {}, falling back to placeholder geometry", e);
                    guard.ifc_model = None;
                }
            }

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
