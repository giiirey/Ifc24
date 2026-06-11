use crate::models::{BoundingBox, MeshData};
use crate::state::get_state;

#[tauri::command]
pub fn load_geometry_batch(entity_ids: Vec<u64>) -> Result<Vec<MeshData>, String> {
    let app = get_state();
    let guard = app.model.lock().map_err(|e| e.to_string())?;
    
    let meshes_vec: Vec<MeshData> = guard.mesh_cache.iter()
        .filter(|(id, _)| entity_ids.contains(id))
        .flat_map(|(_, v)| v.clone())
        .collect();
    
    if !meshes_vec.is_empty() { return Ok(meshes_vec); }
    drop(guard);
    
    let meshes = crate::services::geometry_engine::extract_for_entities(&entity_ids)?;
    {
        let app = get_state();
        let mut g = app.model.lock().map_err(|e| e.to_string())?;
        for mesh in &meshes {
            g.mesh_cache.entry(mesh.entity_id).or_default().push(mesh.clone());
        }
    }
    Ok(meshes)
}

#[tauri::command]
pub fn release_geometry(entity_ids: Vec<u64>) -> Result<usize, String> {
    let app = get_state();
    let mut guard = app.model.lock().map_err(|e| e.to_string())?;
    
    let released = entity_ids.iter()
        .filter(|id| guard.mesh_cache.remove(id).is_some())
        .count();
    Ok(released)
}

#[tauri::command]
pub fn get_bounding_box(entity_id: u64) -> Result<BoundingBox, String> {
    let app = get_state();
    let guard = app.model.lock().map_err(|e| e.to_string())?;
    
    if let Some(meshes) = guard.mesh_cache.get(&entity_id) {
        return Ok(meshes[0].bbox.clone());
    }
    drop(guard);
    
    crate::services::geometry_engine::compute_bbox(entity_id)
}

#[tauri::command]
pub fn get_material_properties(entity_id: u64) -> Result<Vec<crate::models::MaterialProperties>, String> {
    let app = get_state();
    let guard = app.model.lock().map_err(|e| e.to_string())?;
    
    if let Some(meshes) = guard.mesh_cache.get(&entity_id) {
        return Ok(meshes.iter().map(|m| m.material.clone()).collect());
    }
    Ok(vec![])
}
