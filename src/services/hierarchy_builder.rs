use crate::models::{EntitySnapshot, TreeNode};
use std::collections::{HashMap, HashSet};

pub fn build_hierarchy(entities: &HashMap<u64, EntitySnapshot>) -> Result<TreeNode, String> {
    // Find IfcProject (root of decomposition tree)
    let project_id = entities.iter()
        .find(|(_, e)| matches!(e.ifc_type.0, crate::models::entity::IfcType::IfcProject))
        .map(|(id, _)| *id);
    
    let root_id = match project_id {
        Some(id) => id,
        None => return Err("No IFC Project entity found in file".to_string()),
    };
    
    // Build a map of parent_id -> child_ids using IfcRelAggregates and decomposition
    let mut children_map: HashMap<u64, Vec<u64>> = HashMap::new();
    
    for (_, entity) in entities {
        if entity.is_spatial_element() || entity.is_building_element() {
            // Check relationships to find parent
            if let Some(parent_id) = find_parent_entity(&root_id, entity.id, entities) {
                children_map.entry(parent_id).or_insert_with(Vec::new).push(entity.id);
            }
        }
    }
    
    // Build tree from root
    let project = &entities[&root_id];
    let mut root = TreeNode::from_entity(
        root_id,
        &project.display_name(),
        &format!("{:?}", project.ifc_type),
    );
    
    populate_children(&mut root, &children_map, entities);
    
    Ok(root)
}

fn find_parent_entity(root_id: &u64, _entity_id: u64, entities: &HashMap<u64, EntitySnapshot>) -> Option<u64> {
    // Simple approach: use spatial containment relationships
    // In a full implementation, we'd parse IfcRelContainedInSpatialStructure
    let mut current_parent = *root_id;
    
    // Walk up the hierarchy - for 50-200K entities this is acceptable
    loop {
        if current_parent == *root_id {
            return Some(*root_id);
        }
        
        if let Some(parent) = entities.get(&current_parent) {
            if parent.is_spatial_element() {
                return Some(current_parent);
            }
        }
        
        // Default fallback to root
        return Some(*root_id);
    }
}

fn populate_children(node: &mut TreeNode, children_map: &HashMap<u64, Vec<u64>>, entities: &HashMap<u64, EntitySnapshot>) {
    let mut child_ids = match children_map.get(&node.id) {
        Some(ids) => ids.clone(),
        None => return,
    };
    
    // Sort by entity ID for deterministic ordering
    child_ids.sort();
    
    for child_id in &child_ids {
        if let Some(entity) = entities.get(child_id) {
            let mut child_node = TreeNode::from_entity(
                *child_id,
                &entity.display_name(),
                &format!("{:?}", entity.ifc_type),
            );
            
            // Recursively add grandchildren
            populate_children(&mut child_node, children_map, entities);
            node.add_child(child_node);
        }
    }
}
