use crate::models::{EntitySnapshot, TreeNode};
use std::collections::HashMap;

pub fn build_hierarchy(entities: &HashMap<u64, EntitySnapshot>) -> Result<TreeNode, String> {
    let root_id = entities.iter()
        .find(|(_, e)| matches!(e.ifc_type.0, crate::models::entity::IfcType::IfcProject))
        .map(|(id, _)| *id)
        .ok_or("No IFC Project entity found")?;
    
    let project = &entities[&root_id];
    let mut root = TreeNode::from_entity(
        root_id,
        &project.display_name(),
        &format!("{:?}", project.ifc_type),
    );
    
    // Collect all spatial nodes (excluding root) and building elements
    let mut spatial_nodes: Vec<&EntitySnapshot> = Vec::new();
    let mut building_elements: Vec<&EntitySnapshot> = Vec::new();
    
    for (_, entity) in entities.iter() {
        match entity.ifc_type.0 {
            crate::models::entity::IfcType::IfcProject => {}
            crate::models::entity::IfcType::IfcSite 
            | crate::models::entity::IfcType::IfcBuilding
            | crate::models::entity::IfcType::IfcBuildingStorey
            | crate::models::entity::IfcType::IfcSpace => {
                spatial_nodes.push(entity);
            }
            _ if entity.is_building_element() => {
                building_elements.push(entity);
            }
            _ => {}
        }
    }
    
    // Build children for each node using explicit iteration
    
    // 1. Add all spatial nodes as direct children of root
    let mut child_nodes: Vec<TreeNode> = Vec::new();
    for entity in &spatial_nodes {
        let mut node = TreeNode::from_entity(entity.id, &entity.display_name(), &format!("{:?}", entity.ifc_type));
        
        // Find building elements owned by this spatial element (check attribute "#<id>" references)
        for be in &building_elements {
            if is_referenced_in(be, entity.id, entities) || is_parent_of(&be.attributes, entity.id) {
                node.add_child(TreeNode::from_entity(be.id, &be.display_name(), &format!("{:?}", be.ifc_type)));
                
                // Limit per spatial element to prevent huge trees in memory
                if node.children.len() >= 1000 {
                    break;
                }
            }
        }
        
        child_nodes.push(node);
    }
    
    for cn in child_nodes {
        root.add_child(cn);
    }
    
    // 2. Add remaining building elements (not owned by any spatial element) directly under root
    let mut added: std::collections::HashSet<u64> = root.children.iter().flat_map(|n| n.children.iter()).map(|n| n.id).collect();
    for entity in &building_elements {
        if !added.contains(&entity.id) {
            root.add_child(TreeNode::from_entity(entity.id, &entity.display_name(), &format!("{:?}", entity.ifc_type)));
            added.insert(entity.id);
            
            if root.children.len() >= 2000 {
                break;
            }
        }
    }
    
    Ok(root)
}

fn is_referenced_in(entity: &EntitySnapshot, target_id: u64, _entities: &HashMap<u64, EntitySnapshot>) -> bool {
    for attr in &entity.attributes {
        let val = attr.1.trim();
        if val == &format!("#{}", target_id) || val.starts_with(&format!("#{}", target_id)) {
            return true;
        }
    }
    false
}

fn is_parent_of(attributes: &[(String, String)], target_id: u64) -> bool {
    for (_, val) in attributes.iter() {
        let val = val.trim();
        if val == &format!("#{}", target_id) || val.starts_with(&format!("#{}", target_id)) {
            return true;
        }
    }
    false
}
