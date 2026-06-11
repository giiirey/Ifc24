use crate::models::{EntitySnapshot, TreeNode};
use crate::state::get_state;

#[tauri::command]
pub fn get_entity_count() -> Result<usize, String> {
    let app = get_state();
    let guard = app.model.lock().map_err(|e| e.to_string())?;
    Ok(guard.entities.len())
}

#[tauri::command]
pub fn get_entity_by_id(entity_id: u64) -> Result<EntitySnapshot, String> {
    let app = get_state();
    let guard = app.model.lock().map_err(|e| e.to_string())?;
    guard.entities.get(&entity_id).cloned()
        .ok_or_else(|| format!("Entity not found: {}", entity_id))
}

#[tauri::command]
pub fn get_hierarchy_root() -> Result<Option<TreeNode>, String> {
    let app = get_state();
    let guard = app.model.lock().map_err(|e| e.to_string())?;
    Ok(guard.hierarchy_root.clone())
}

#[tauri::command]
pub fn get_children(entity_id: u64) -> Result<Vec<TreeNode>, String> {
    let app = get_state();
    let guard = app.model.lock().map_err(|e| e.to_string())?;
    
    if let Some(root) = &guard.hierarchy_root {
        find_node_and_children(root, entity_id)
            .map(|node| node.children.clone())
            .ok_or_else(|| format!("Node not found: {}", entity_id))
    } else {
        Err("Hierarchy not loaded".to_string())
    }
}

fn find_node_and_children<'a>(node: &'a TreeNode, id: u64) -> Option<&'a TreeNode> {
    if node.id == id { return Some(node); }
    for child in &node.children {
        if let Some(found) = find_node_and_children(child, id) { return Some(found); }
    }
    None
}

#[tauri::command]
pub fn search_entities(query: String) -> Result<Vec<u64>, String> {
    let app = get_state();
    let guard = app.model.lock().map_err(|e| e.to_string())?;
    
    let query_lower = query.to_lowercase();
    let results: Vec<u64> = guard.entities.values()
        .filter(|entity| {
            entity.global_id.contains(&query)
                || entity.name.as_ref().map(|n| n.to_lowercase().contains(&query_lower)) == Some(true)
                || entity.description.as_ref().map(|d| d.to_lowercase().contains(&query_lower)) == Some(true)
        })
        .map(|e| e.id)
        .collect();
    
    Ok(results)
}

#[tauri::command]
pub fn get_all_entity_ids() -> Result<Vec<u64>, String> {
    let app = get_state();
    let guard = app.model.lock().map_err(|e| e.to_string())?;
    Ok(guard.entities.keys().cloned().collect())
}

#[tauri::command]
pub fn get_all_entities() -> Result<Vec<EntitySnapshot>, String> {
    let app = get_state();
    let guard = app.model.lock().map_err(|e| e.to_string())?;
    Ok(guard.entities.values().cloned().collect())
}

#[tauri::command]
pub fn get_entity_types() -> Result<Vec<(String, usize)>, String> {
    let app = get_state();
    let guard = app.model.lock().map_err(|e| e.to_string())?;
    
    let mut type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for entity in guard.entities.values() {
        let type_name = format!("{:?}", entity.ifc_type);
        *type_counts.entry(type_name).or_insert(0) += 1;
    }
    
    let mut results: Vec<(String, usize)> = type_counts.into_iter().collect();
    results.sort_by(|a, b| b.1.cmp(&a.1));
    
    Ok(results)
}
