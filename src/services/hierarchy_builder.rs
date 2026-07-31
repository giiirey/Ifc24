use crate::models::{EntitySnapshot, IfcType, TreeNode};
use std::collections::{HashMap, HashSet};

pub fn build_hierarchy(entities: &HashMap<u64, EntitySnapshot>) -> Result<TreeNode, String> {
    let project_id = entities.iter()
        .find(|(_, e)| matches!(e.ifc_type.0, IfcType::IfcProject))
        .map(|(id, _)| *id)
        .ok_or("No IFC Project entity found")?;

    let project = &entities[&project_id];
    let mut root = TreeNode::from_entity(
        project_id,
        &project.display_name(),
        &format!("{:?}", project.ifc_type.0),
    );

    // Build spatial decomposition map: parent_id -> [child_ids]
    let mut decomposes_map: HashMap<u64, Vec<u64>> = HashMap::new();
    
    // Build spatial containment map: spatial_id -> [element_ids]
    let mut containment_map: HashMap<u64, Vec<u64>> = HashMap::new();

    // Collect spatial nodes, assembly nodes, and building elements
    let mut spatial_nodes: HashMap<u64, &EntitySnapshot> = HashMap::new();
    let mut assembly_nodes: HashMap<u64, &EntitySnapshot> = HashMap::new();
    let mut building_elements: Vec<&EntitySnapshot> = Vec::new();

    for (&id, entity) in entities.iter() {
        match entity.ifc_type.0 {
            IfcType::IfcProject | IfcType::IfcSite | IfcType::IfcBuilding | IfcType::IfcBuildingStorey => {
                if id != project_id {
                    spatial_nodes.insert(id, entity);
                }
            }
            IfcType::IfcAssembly | IfcType::IfcElementAssembly => {
                assembly_nodes.insert(id, entity);
            }
            _ if is_building_element(&entity.ifc_type.0) => {
                building_elements.push(entity);
            }
            _ => {}
        }
    }

    // Process relationship entities to build decomposition and containment maps
    for (_, entity) in entities.iter() {
        match entity.ifc_type.0 {
            IfcType::IfcRelDecomposes | IfcType::IfcRelAggregates => {
                let (relating_id, related_ids) = extract_aggregate_relation(entity);
                if let Some(relating_id) = relating_id {
                    decomposes_map.entry(relating_id).or_default().extend(related_ids);
                }
            }
            IfcType::IfcRelContainedInSpatialStructure => {
                let (spatial_id, element_ids) = extract_containment_relation(entity);
                if let Some(spatial_id) = spatial_id {
                    containment_map.entry(spatial_id).or_default().extend(element_ids);
                }
            }
            IfcType::IfcRelNests => {
                let (relating_id, related_ids) = extract_nest_relation(entity);
                if let Some(relating_id) = relating_id {
                    decomposes_map.entry(relating_id).or_default().extend(related_ids);
                }
            }
            _ => {}
        }
    }

    // Build spatial hierarchy: Project -> Site -> Building -> BuildingStorey
    // Find sites directly under project
    let site_ids = decomposes_map.get(&project_id).cloned().unwrap_or_default();
    let has_decomposition_hierarchy = !site_ids.is_empty();
    for site_id in site_ids {
        if let Some(site_entity) = spatial_nodes.get(&site_id) {
            let mut site_node = TreeNode::from_entity(
                site_id,
                &site_entity.display_name(),
                &format!("{:?}", site_entity.ifc_type.0),
            );
            
            // Find buildings under this site
            let building_ids = decomposes_map.get(&site_id).cloned().unwrap_or_default();
            for building_id in building_ids {
                if let Some(building_entity) = spatial_nodes.get(&building_id) {
                    let mut building_node = TreeNode::from_entity(
                        building_id,
                        &building_entity.display_name(),
                        &format!("{:?}", building_entity.ifc_type.0),
                    );
                    
                    // Find building storeys under this building
                    let storey_ids = decomposes_map.get(&building_id).cloned().unwrap_or_default();
                    for storey_id in storey_ids {
                        if let Some(storey_entity) = spatial_nodes.get(&storey_id) {
                            let mut storey_node = TreeNode::from_entity(
                                storey_id,
                                &storey_entity.display_name(),
                                &format!("{:?}", storey_entity.ifc_type.0),
                            );

                            // Add assemblies under building storey (with their child elements)
                            add_assemblies_and_elements(&mut storey_node, storey_id, &entities, &assembly_nodes, &containment_map, &decomposes_map);

                            building_node.add_child(storey_node);
                        }
                    }
                    
                    // Add assemblies directly under building (if not under a storey) with their child elements
                    let mut building_assembly_ids: Vec<u64> = decomposes_map.get(&building_id).cloned().unwrap_or_default();
                    let building_elem_ids: Vec<u64> = containment_map.get(&building_id).cloned().unwrap_or_default();
                    let building_elem_ids_clone = building_elem_ids.clone();
                    let mut building_elem_set: HashSet<u64> = HashSet::new();
                    for eid in building_elem_ids_clone {
                        building_elem_set.insert(eid);
                    }
                    let mut storey_assembly_set: HashSet<u64> = HashSet::new();
                    for sid in decomposes_map.get(&building_id).cloned().unwrap_or_default() {
                        if let Some(sass) = decomposes_map.get(&sid) {
                            for a in sass {
                                storey_assembly_set.insert(*a);
                            }
                        }
                    }
                    for (asm_id, _asm_entity) in assembly_nodes.iter() {
                        if !building_assembly_ids.contains(asm_id) {
                            if building_elem_set.contains(asm_id) {
                                building_assembly_ids.push(*asm_id);
                            } else {
                                let asm_elem_ids = containment_map.get(asm_id).cloned().unwrap_or_default();
                                let mut found = false;
                                for elem_id in asm_elem_ids {
                                    if building_elem_set.contains(&elem_id) {
                                        building_assembly_ids.push(*asm_id);
                                        found = true;
                                        break;
                                    }
                                }
                                if !found {
                                    let asm_decompose_ids = decomposes_map.get(asm_id).cloned().unwrap_or_default();
                                    for elem_id in asm_decompose_ids {
                                        if building_elem_set.contains(&elem_id) {
                                            building_assembly_ids.push(*asm_id);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    for assembly_id in &building_assembly_ids {
                        if !storey_assembly_set.contains(assembly_id) {
                            if let Some(assembly_entity) = assembly_nodes.get(assembly_id) {
                                let mut asm_node = TreeNode::from_entity(
                                    *assembly_id,
                                    &assembly_entity.display_name(),
                                    &format!("{:?}", assembly_entity.ifc_type.0),
                                );
                                // Add elements from containment map
                                let elem_ids = containment_map.get(assembly_id).cloned().unwrap_or_default();
                                for elem_id in elem_ids {
                                    if let Some(elem_entity) = entities.get(&elem_id) {
                                        if is_building_element(&elem_entity.ifc_type.0) {
                                            asm_node.add_child(TreeNode::from_entity(
                                                elem_id,
                                                &elem_entity.display_name(),
                                                &format!("{:?}", elem_entity.ifc_type.0),
                                            ));
                                        }
                                    }
                                }
                                // Add elements from decomposition map
                                let decompose_ids = decomposes_map.get(assembly_id).cloned().unwrap_or_default();
                                for elem_id in decompose_ids {
                                    if let Some(elem_entity) = entities.get(&elem_id) {
                                        if is_building_element(&elem_entity.ifc_type.0) {
                                            asm_node.add_child(TreeNode::from_entity(
                                                elem_id,
                                                &elem_entity.display_name(),
                                                &format!("{:?}", elem_entity.ifc_type.0),
                                            ));
                                        }
                                    }
                                }
                                building_node.add_child(asm_node);
                            }
                        }
                    }

                    site_node.add_child(building_node);
                }
            }
            
            // Add assemblies directly under site with their child elements
            let mut site_assembly_ids: Vec<u64> = decomposes_map.get(&site_id).cloned().unwrap_or_default();
            let site_elem_ids: Vec<u64> = containment_map.get(&site_id).cloned().unwrap_or_default();
            let site_elem_ids_clone = site_elem_ids.clone();
            let mut site_elem_set: HashSet<u64> = HashSet::new();
            for eid in site_elem_ids_clone {
                site_elem_set.insert(eid);
            }
            
            // Find assemblies that have elements belonging to this site
            // or that are contained in this site directly
            for (asm_id, _asm_entity) in assembly_nodes.iter() {
                if !site_assembly_ids.contains(asm_id) {
                    if site_elem_set.contains(asm_id) {
                        site_assembly_ids.push(*asm_id);
                    } else {
                        let asm_elem_ids = containment_map.get(asm_id).cloned().unwrap_or_default();
                        let mut found = false;
                        for elem_id in asm_elem_ids {
                            if site_elem_set.contains(&elem_id) {
                                site_assembly_ids.push(*asm_id);
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            let asm_decompose_ids = decomposes_map.get(asm_id).cloned().unwrap_or_default();
                            for elem_id in asm_decompose_ids {
                                if site_elem_set.contains(&elem_id) {
                                    site_assembly_ids.push(*asm_id);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            
            for assembly_id in site_assembly_ids {
                if let Some(assembly_entity) = assembly_nodes.get(&assembly_id) {
                    let mut asm_node = TreeNode::from_entity(
                        assembly_id,
                        &assembly_entity.display_name(),
                        &format!("{:?}", assembly_entity.ifc_type.0),
                    );
                    // Add elements from containment map
                    let elem_ids = containment_map.get(&assembly_id).cloned().unwrap_or_default();
                    for elem_id in elem_ids {
                        if let Some(elem_entity) = entities.get(&elem_id) {
                            if is_building_element(&elem_entity.ifc_type.0) {
                                asm_node.add_child(TreeNode::from_entity(
                                    elem_id,
                                    &elem_entity.display_name(),
                                    &format!("{:?}", elem_entity.ifc_type.0),
                                ));
                            }
                        }
                    }
                    // Add elements from decomposition map
                    let decompose_ids = decomposes_map.get(&assembly_id).cloned().unwrap_or_default();
                    for elem_id in decompose_ids {
                        if let Some(elem_entity) = entities.get(&elem_id) {
                            if is_building_element(&elem_entity.ifc_type.0) {
                                asm_node.add_child(TreeNode::from_entity(
                                    elem_id,
                                    &elem_entity.display_name(),
                                    &format!("{:?}", elem_entity.ifc_type.0),
                                ));
                            }
                        }
                    }
                    site_node.add_child(asm_node);
                }
            }

            root.add_child(site_node);
        }
    }

    if !has_decomposition_hierarchy {
        // Fallback: no decomposition relations found, build hierarchy purely by entity type
        // Fallback: no decomposition relations found, build hierarchy purely by entity type
        // IFC spec: Project contains Site(s), Site contains Building(s), Building contains BuildingStorey(s)
        let mut site_nodes: HashMap<u64, &EntitySnapshot> = HashMap::new();
        let mut building_nodes: HashMap<u64, &EntitySnapshot> = HashMap::new();
        let mut storey_nodes: HashMap<u64, &EntitySnapshot> = HashMap::new();
        
        for (&id, entity) in &spatial_nodes {
            match entity.ifc_type.0 {
                IfcType::IfcSite => { site_nodes.insert(id, entity); }
                IfcType::IfcBuilding => { building_nodes.insert(id, entity); }
                IfcType::IfcBuildingStorey => { storey_nodes.insert(id, entity); }
                _ => {}
            }
        }
        
        // If there are sites, build Site -> Building -> Storey chain
        if !site_nodes.is_empty() {
            for (&sid, site_entity) in &site_nodes {
                let mut site_node = TreeNode::from_entity(
                    sid,
                    &site_entity.display_name(),
                    &format!("{:?}", site_entity.ifc_type.0),
                );
                
                // Add all buildings under this site (or distribute if multiple sites)
                let buildings_for_site = if building_nodes.len() == 1 {
                    building_nodes.values().map(|e| (e.id, *e)).collect::<Vec<_>>()
                } else {
                    // Distribute buildings: assign each building to a site by name matching or round-robin
                    building_nodes.values()
                        .enumerate()
                        .filter(|(i, _)| *i % site_nodes.len() == site_nodes.keys().position(|&k| k == sid).unwrap_or(0))
                        .map(|(_, e)| (e.id, *e))
                        .collect::<Vec<_>>()
                };
                
                for (bid, building_entity) in buildings_for_site {
                    let mut building_node = TreeNode::from_entity(
                        bid,
                        &building_entity.display_name(),
                        &format!("{:?}", building_entity.ifc_type.0),
                    );
                    
                    // Add all storeys under this building
                    let storeys_for_building = if storey_nodes.is_empty() {
                        Vec::new()
                    } else if storey_nodes.len() == 1 {
                        storey_nodes.values().map(|e| (e.id, *e)).collect()
                    } else {
                        let building_idx = building_nodes.values().position(|x| x.id == bid).unwrap_or(0);
                        let num_buildings = building_nodes.len();
                        storey_nodes.values()
                            .enumerate()
                            .filter(move |(i, _)| *i % num_buildings == building_idx)
                            .map(|(_, e)| (e.id, *e))
                            .collect()
                    };
                    
                    for (storey_id, storey_entity) in storeys_for_building {
                        let mut storey_node = TreeNode::from_entity(
                            storey_id,
                            &storey_entity.display_name(),
                            &format!("{:?}", storey_entity.ifc_type.0),
                        );
                        add_assemblies_and_elements(&mut storey_node, storey_id, &entities, &assembly_nodes, &containment_map, &decomposes_map);
                        building_node.add_child(storey_node);
                    }
                    
                    // Add building elements contained in this building
                    let elem_ids = containment_map.get(&bid).cloned().unwrap_or_default();
                    for elem_id in elem_ids {
                        if let Some(elem_entity) = entities.get(&elem_id) {
                            if is_building_element(&elem_entity.ifc_type.0) {
                                building_node.add_child(TreeNode::from_entity(
                                    elem_id,
                                    &elem_entity.display_name(),
                                    &format!("{:?}", elem_entity.ifc_type.0),
                                ));
                            }
                        }
                    }
                    
                    // Add assemblies related to this building
                    let mut asm_ids: Vec<u64> = decomposes_map.get(&bid).cloned().unwrap_or_default();
                    let b_elem_ids: Vec<u64> = containment_map.get(&bid).cloned().unwrap_or_default();
                    let b_elem_ids_clone = b_elem_ids.clone();
                    let mut b_elem_set: HashSet<u64> = HashSet::new();
                    for eid in b_elem_ids_clone {
                        b_elem_set.insert(eid);
                    }
                    for (aid, _aent) in assembly_nodes.iter() {
                        if !asm_ids.contains(aid) {
                            if b_elem_set.contains(aid) {
                                asm_ids.push(*aid);
                            } else {
                                let a_elem_ids = containment_map.get(aid).cloned().unwrap_or_default();
                                for eid in a_elem_ids {
                                    if b_elem_set.contains(&eid) {
                                        asm_ids.push(*aid);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    for elem_id in &b_elem_ids {
                        if let Some(elem_entity) = entities.get(elem_id) {
                            if !is_building_element(&elem_entity.ifc_type.0) {
                                continue;
                            }
                            for (aid, _aent) in assembly_nodes.iter() {
                                if !asm_ids.contains(aid) && element_references(elem_entity, *aid) {
                                    asm_ids.push(*aid);
                                    break;
                                }
                            }
                        }
                    }
                    for asm_id in &asm_ids {
                        if let Some(asm_entity) = assembly_nodes.get(asm_id) {
                            building_node.add_child(TreeNode::from_entity(
                                *asm_id,
                                &asm_entity.display_name(),
                                &format!("{:?}", asm_entity.ifc_type.0),
                            ));
                        }
                    }
                    
                    site_node.add_child(building_node);
                }
                
                // Add assemblies related to site with their child elements
                let mut site_asm_ids: Vec<u64> = decomposes_map.get(&sid).cloned().unwrap_or_default();
                let site_elem_ids: Vec<u64> = containment_map.get(&sid).cloned().unwrap_or_default();
                let site_elem_ids_clone = site_elem_ids.clone();
                let mut site_elem_set: HashSet<u64> = HashSet::new();
                for eid in site_elem_ids_clone {
                    site_elem_set.insert(eid);
                }
                
                // Find assemblies that have elements belonging to this site
                // or that are contained in this site directly
                for (asm_id, _asm_entity) in assembly_nodes.iter() {
                    if !site_asm_ids.contains(asm_id) {
                        if site_elem_set.contains(asm_id) {
                            site_asm_ids.push(*asm_id);
                        } else {
                            let asm_elem_ids = containment_map.get(asm_id).cloned().unwrap_or_default();
                            let mut found = false;
                            for elem_id in asm_elem_ids {
                                if site_elem_set.contains(&elem_id) {
                                    site_asm_ids.push(*asm_id);
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                let asm_decompose_ids = decomposes_map.get(asm_id).cloned().unwrap_or_default();
                                for elem_id in asm_decompose_ids {
                                    if site_elem_set.contains(&elem_id) {
                                        site_asm_ids.push(*asm_id);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                for elem_id in &site_elem_ids {
                    if let Some(elem_entity) = entities.get(elem_id) {
                        if !is_building_element(&elem_entity.ifc_type.0) {
                            continue;
                        }
                        for (asm_id, _asm_entity) in assembly_nodes.iter() {
                            if !site_asm_ids.contains(asm_id) && element_references(elem_entity, *asm_id) {
                                site_asm_ids.push(*asm_id);
                                break;
                            }
                        }
                    }
                }
                
                let mut site_elems_per_assembly: HashMap<u64, Vec<u64>> = HashMap::new();
                let mut site_orphan_elems: Vec<u64> = Vec::new();
                
                for elem_id in site_elem_ids {
                    if let Some(elem_entity) = entities.get(&elem_id) {
                        if !is_building_element(&elem_entity.ifc_type.0) {
                            continue;
                        }
                        let mut assigned: Option<u64> = None;
                        for aid in &site_asm_ids {
                            if element_references(elem_entity, *aid) {
                                assigned = Some(*aid);
                                break;
                            }
                        }
                        if let Some(aid) = assigned {
                            site_elems_per_assembly.entry(aid).or_default().push(elem_id);
                        } else {
                            site_orphan_elems.push(elem_id);
                        }
                    }
                }
                
                // Also add elements contained in each assembly
                for asm_id in &site_asm_ids {
                    if let Some(asm_elem_ids) = containment_map.get(asm_id) {
                        for elem_id in asm_elem_ids {
                            if let Some(elem_entity) = entities.get(elem_id) {
                                if is_building_element(&elem_entity.ifc_type.0) {
                                    if !site_elems_per_assembly.get(asm_id).unwrap_or(&Vec::new()).contains(elem_id) {
                                        site_elems_per_assembly.entry(*asm_id).or_default().push(*elem_id);
                                    }
                                }
                            }
                        }
                    }
                }

                // Also add elements decomposed by each assembly
                for asm_id in &site_asm_ids {
                    if let Some(asm_child_ids) = decomposes_map.get(asm_id) {
                        for elem_id in asm_child_ids {
                            if let Some(elem_entity) = entities.get(elem_id) {
                                if is_building_element(&elem_entity.ifc_type.0) {
                                    let key = *asm_id;
                                    if !site_elems_per_assembly.contains_key(&key) {
                                        site_elems_per_assembly.entry(key).or_default().push(*elem_id);
                                    } else if !site_elems_per_assembly.get(&key).unwrap().contains(elem_id) {
                                        site_elems_per_assembly.get_mut(&key).unwrap().push(*elem_id);
                                    }
                                }
                            }
                        }
                    }
                }
                
                for asm_id in &site_asm_ids {
                    if let Some(asm_entity) = assembly_nodes.get(asm_id) {
                        let mut asm_node = TreeNode::from_entity(
                            *asm_id,
                            &asm_entity.display_name(),
                            &format!("{:?}", asm_entity.ifc_type.0),
                        );
                        if let Some(elems) = site_elems_per_assembly.get(asm_id) {
                            for eid in elems {
                                if let Some(ee) = entities.get(eid) {
                                    asm_node.add_child(TreeNode::from_entity(
                                        *eid,
                                        &ee.display_name(),
                                        &format!("{:?}", ee.ifc_type.0),
                                    ));
                                }
                            }
                        }
                        site_node.add_child(asm_node);
                    }
                }
                
                for eid in site_orphan_elems {
                    if let Some(ee) = entities.get(&eid) {
                        site_node.add_child(TreeNode::from_entity(
                            eid,
                            &ee.display_name(),
                            &format!("{:?}", ee.ifc_type.0),
                        ));
                    }
                }
                
                root.add_child(site_node);
            }
        } else if !building_nodes.is_empty() {
            // No sites, put buildings directly under project
            for (&bid, building_entity) in building_nodes.iter() {
                let mut building_node = TreeNode::from_entity(
                    bid,
                    &building_entity.display_name(),
                    &format!("{:?}", building_entity.ifc_type.0),
                );
                
                for (&storey_id, storey_entity) in storey_nodes.iter() {
                    let mut storey_node = TreeNode::from_entity(
                        storey_id,
                        &storey_entity.display_name(),
                        &format!("{:?}", storey_entity.ifc_type.0),
                    );
                    add_assemblies_and_elements(&mut storey_node, storey_id, &entities, &assembly_nodes, &containment_map, &decomposes_map);
                    building_node.add_child(storey_node);
                }
                
                // Add assemblies under building with their child elements
                let mut asm_ids: Vec<u64> = decomposes_map.get(&bid).cloned().unwrap_or_default();
                let elem_ids: Vec<u64> = containment_map.get(&bid).cloned().unwrap_or_default();
                let elem_ids_clone = elem_ids.clone();
                let mut elem_set: HashSet<u64> = HashSet::new();
                for eid in elem_ids_clone {
                    elem_set.insert(eid);
                }
                
                // Find assemblies that have elements belonging to this building
                // or that are contained in this building directly
                for (aid, _aent) in assembly_nodes.iter() {
                    if !asm_ids.contains(&aid) {
                        if elem_set.contains(aid) {
                            asm_ids.push(*aid);
                        } else {
                            let a_elem_ids = containment_map.get(&aid).cloned().unwrap_or_default();
                            for eid in a_elem_ids {
                                if elem_set.contains(&eid) {
                                    asm_ids.push(*aid);
                                    break;
                                }
                            }
                        }
                    }
                }
                for elem_id in &elem_ids {
                    if let Some(elem_entity) = entities.get(elem_id) {
                        if !is_building_element(&elem_entity.ifc_type.0) {
                            continue;
                        }
                        for (aid, _aent) in assembly_nodes.iter() {
                            if !asm_ids.contains(aid) && element_references(elem_entity, *aid) {
                                asm_ids.push(*aid);
                                break;
                            }
                        }
                    }
                }
                
                let mut elements_per_assembly: HashMap<u64, Vec<u64>> = HashMap::new();
                let mut orphan_elems: Vec<u64> = Vec::new();
                
                for elem_id in elem_ids {
                    if let Some(elem_entity) = entities.get(&elem_id) {
                        if !is_building_element(&elem_entity.ifc_type.0) {
                            continue;
                        }
                        let mut assigned: Option<u64> = None;
                        for aid in &asm_ids {
                            if element_references(elem_entity, *aid) {
                                assigned = Some(*aid);
                                break;
                            }
                        }
                        if let Some(aid) = assigned {
                            elements_per_assembly.entry(aid).or_default().push(elem_id);
                        } else {
                            orphan_elems.push(elem_id);
                        }
                    }
                }
                
                // Also add elements contained in each assembly
                for asm_id in &asm_ids {
                    if let Some(a_elem_ids) = containment_map.get(asm_id) {
                        for eid in a_elem_ids {
                            if let Some(ee) = entities.get(eid) {
                                if is_building_element(&ee.ifc_type.0) {
                                    if !elements_per_assembly.get(asm_id).unwrap_or(&Vec::new()).contains(eid) {
                                        elements_per_assembly.entry(*asm_id).or_default().push(*eid);
                                    }
                                }
                            }
                        }
                    }
                }

                // Also add elements decomposed by each assembly
                for asm_id in &asm_ids {
                    if let Some(asm_child_ids) = decomposes_map.get(asm_id) {
                        for elem_id in asm_child_ids {
                            if let Some(elem_entity) = entities.get(elem_id) {
                                if is_building_element(&elem_entity.ifc_type.0) {
                                    let key = *asm_id;
                                    if !elements_per_assembly.contains_key(&key) {
                                        elements_per_assembly.entry(key).or_default().push(*elem_id);
                                    } else if !elements_per_assembly.get(&key).unwrap().contains(elem_id) {
                                        elements_per_assembly.get_mut(&key).unwrap().push(*elem_id);
                                    }
                                }
                            }
                        }
                    }
                }
                
                for asm_id in &asm_ids {
                    if let Some(asm_entity) = assembly_nodes.get(asm_id) {
                        let mut asm_node = TreeNode::from_entity(
                            *asm_id,
                            &asm_entity.display_name(),
                            &format!("{:?}", asm_entity.ifc_type.0),
                        );
                        if let Some(elems) = elements_per_assembly.get(asm_id) {
                            for eid in elems {
                                if let Some(ee) = entities.get(eid) {
                                    asm_node.add_child(TreeNode::from_entity(
                                        *eid,
                                        &ee.display_name(),
                                        &format!("{:?}", ee.ifc_type.0),
                                    ));
                                }
                            }
                        }
                        building_node.add_child(asm_node);
                    }
                }
                
                for eid in orphan_elems {
                    if let Some(ee) = entities.get(&eid) {
                        building_node.add_child(TreeNode::from_entity(
                            eid,
                            &ee.display_name(),
                            &format!("{:?}", ee.ifc_type.0),
                        ));
                    }
                }
                
                root.add_child(building_node);
            }
        } else if !storey_nodes.is_empty() {
            // No sites or buildings, put storeys directly under project
            for (&storey_id, storey_entity) in storey_nodes.iter() {
                let mut storey_node = TreeNode::from_entity(
                    storey_id,
                    &storey_entity.display_name(),
                    &format!("{:?}", storey_entity.ifc_type.0),
                );
                add_assemblies_and_elements(&mut storey_node, storey_id, &entities, &assembly_nodes, &containment_map, &decomposes_map);
                root.add_child(storey_node);
            }
        }
    }

    // Add orphan assemblies and building elements under storeys
    let mut all_ids: HashSet<u64> = root.flatten().iter().map(|n| n.id).collect();
    
    // Find all building storey IDs
    let storey_ids: Vec<u64> = root
        .flatten()
        .into_iter()
        .filter(|n| n.ifc_type == "IfcBuildingStorey")
        .map(|n| n.id)
        .collect();
    
    // Add orphan assemblies under first available storey
    for (&id, entity) in &assembly_nodes {
        if !all_ids.contains(&id) {
            if let Some(&storey_id) = storey_ids.first() {
                let mut asm_node = TreeNode::from_entity(
                    id,
                    &entity.display_name(),
                    &format!("{:?}", entity.ifc_type.0),
                );
                if let Some(storey_mut) = root.find_by_id_mut(storey_id) {
                    storey_mut.add_child(asm_node);
                }
            } else {
                root.add_child(TreeNode::from_entity(
                    id,
                    &entity.display_name(),
                    &format!("{:?}", entity.ifc_type.0),
                ));
            }
            all_ids.insert(id);
        }
    }

    // Add orphan building elements under first available storey
    let contained_ids: HashSet<u64> = containment_map.values().flatten().cloned().collect();
    for entity in &building_elements {
        if !contained_ids.contains(&entity.id) && !all_ids.contains(&entity.id) {
            if let Some(&storey_id) = storey_ids.first() {
                if let Some(storey_mut) = root.find_by_id_mut(storey_id) {
                    let mut elem_node = TreeNode::from_entity(
                        entity.id,
                        &entity.display_name(),
                        &format!("{:?}", entity.ifc_type.0),
                    );
                    storey_mut.add_child(elem_node);
                }
            } else {
                root.add_child(TreeNode::from_entity(
                    entity.id,
                    &entity.display_name(),
                    &format!("{:?}", entity.ifc_type.0),
                ));
            }
            all_ids.insert(entity.id);
        }
    }

    Ok(root)
}

fn is_building_element(ifc_type: &IfcType) -> bool {
    matches!(
        *ifc_type,
        IfcType::IfcWall
            | IfcType::IfcSlab
            | IfcType::IfcColumn
            | IfcType::IfcBeam
            | IfcType::IfcDoor
            | IfcType::IfcWindow
            | IfcType::IfcStair
            | IfcType::IfcRailing
            | IfcType::IfcRoof
            | IfcType::IfcCurtainWall
            | IfcType::IfcFurniture
            | IfcType::IfcPlate
            | IfcType::IfcMechanicalFastener
    )
}

fn extract_id_refs(value: &str) -> Vec<u64> {
    let mut ids = Vec::new();
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '#' {
            let mut id_str = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    id_str.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if let Ok(id) = id_str.parse::<u64>() {
                ids.push(id);
            }
        }
    }
    ids
}

fn extract_aggregate_relation(entity: &EntitySnapshot) -> (Option<u64>, Vec<u64>) {
    // IfcRelAggregates/IfcRelDecomposes attributes:
    // IfcRelAggregates(GlobalId, OwnerHistory, Name, Description, RelatingObject, RelatedObjects)
    // RelatingObject is attribute index 4 (single entity ref)
    // RelatedObjects is attribute index 5 (list of entity refs)
    
    let mut relating_id: Option<u64> = None;
    let mut related_ids: Vec<u64> = Vec::new();
    
    for (i, attr) in entity.attributes.iter().enumerate() {
        let val = attr.1.trim();
        
        if i == 4 {
            // RelatingObject - single entity reference
            if val.starts_with('#') {
                if let Ok(id) = val.strip_prefix('#').unwrap_or(val).parse::<u64>() {
                    relating_id = Some(id);
                }
            }
        } else if i == 5 {
            // RelatedObjects - list of entity references
            let refs = extract_id_refs(val);
            related_ids = refs;
        }
    }
    
    (relating_id, related_ids)
}

fn extract_containment_relation(entity: &EntitySnapshot) -> (Option<u64>, Vec<u64>) {
    // IfcRelContainedInSpatialStructure attributes:
    // IfcRelContainedInSpatialStructure(GlobalId, OwnerHistory, Name, Description, RelatedElements, RelatingContext)
    // RelatedElements = list of building elements (#id, #id, ...) - attribute index 4
    // RelatingContext = single spatial structure (#id) - attribute index 5
    
    let mut relating_id: Option<u64> = None;
    let mut related_ids: Vec<u64> = Vec::new();
    
    for (i, attr) in entity.attributes.iter().enumerate() {
        let val = attr.1.trim();
        let refs = extract_id_refs(val);
        
        if i == 4 {
            // RelatedElements - attribute index 4
            related_ids = refs;
        } else if i == 5 {
            // RelatingContext - attribute index 5
            if val.starts_with('#') {
                if let Ok(id) = val.strip_prefix('#').unwrap_or(val).parse::<u64>() {
                    relating_id = Some(id);
                }
            }
        }
    }
    
    (relating_id, related_ids)
}

fn extract_nest_relation(entity: &EntitySnapshot) -> (Option<u64>, Vec<u64>) {
    // IfcRelNests: similar to IfcRelAggregates
    // IfcRelNests(GlobalId, OwnerHistory, Name, Description, RelatingObject, RelatedObjects)
    // RelatingObject is attribute index 4 (single entity ref)
    // RelatedObjects is attribute index 5 (list of entity refs)
    
    let mut relating_id: Option<u64> = None;
    let mut related_ids: Vec<u64> = Vec::new();
    
    for (i, attr) in entity.attributes.iter().enumerate() {
        let val = attr.1.trim();
        
        if i == 4 {
            // RelatingObject - single entity reference
            if val.starts_with('#') {
                if let Ok(id) = val.strip_prefix('#').unwrap_or(val).parse::<u64>() {
                    relating_id = Some(id);
                }
            }
        } else if i == 5 {
            // RelatedObjects - list of entity references
            let refs = extract_id_refs(val);
            related_ids = refs;
        }
    }
    
    (relating_id, related_ids)
}

fn add_assemblies_and_elements(
    node: &mut TreeNode,
    parent_id: u64,
    entities: &HashMap<u64, EntitySnapshot>,
    assembly_nodes: &HashMap<u64, &EntitySnapshot>,
    containment_map: &HashMap<u64, Vec<u64>>,
    decomposes_map: &HashMap<u64, Vec<u64>>,
) {
    // Get assemblies that belong to this spatial element via decomposition
    let mut asm_ids: Vec<u64> = decomposes_map.get(&parent_id).cloned().unwrap_or_default();
    
    // Recursively find nested assemblies through decomposes_map
    let mut to_check: Vec<u64> = asm_ids.clone();
    let mut checked: HashSet<u64> = asm_ids.iter().copied().collect();
    while let Some(current_id) = to_check.pop() {
        if let Some(child_asm_ids) = decomposes_map.get(&current_id) {
            for child_id in child_asm_ids {
                if assembly_nodes.contains_key(child_id) && !checked.contains(child_id) {
                    asm_ids.push(*child_id);
                    to_check.push(*child_id);
                    checked.insert(*child_id);
                }
            }
        }
    }
    
    // Also find assemblies that have elements contained in them (IfcRelContainedInSpatialStructure)
    // and check if those elements belong to this parent spatial element
    let parent_elem_ids: Vec<u64> = containment_map.get(&parent_id).cloned().unwrap_or_default();
    let parent_elem_ids_clone = parent_elem_ids.clone();
    let mut parent_elem_set: HashSet<u64> = HashSet::new();
    for eid in parent_elem_ids_clone {
        parent_elem_set.insert(eid);
    }
    
    // Check each assembly to see if it has elements that belong to this parent
    // or if the assembly itself is contained in this parent
    for (asm_id, _asm_entity) in assembly_nodes.iter() {
        if !asm_ids.contains(asm_id) {
            if parent_elem_set.contains(asm_id) {
                asm_ids.push(*asm_id);
            } else {
                // Check elements contained in the assembly
                let asm_elem_ids = containment_map.get(asm_id).cloned().unwrap_or_default();
                let mut found = false;
                for elem_id in asm_elem_ids {
                    if parent_elem_set.contains(&elem_id) {
                        asm_ids.push(*asm_id);
                        found = true;
                        break;
                    }
                }
                // Also check elements decomposed by the assembly
                if !found {
                    let asm_decompose_ids = decomposes_map.get(asm_id).cloned().unwrap_or_default();
                    for elem_id in asm_decompose_ids {
                        if parent_elem_set.contains(&elem_id) {
                            asm_ids.push(*asm_id);
                            found = true;
                            break;
                        }
                    }
                }
            }
        }
    }
    
    // Group all elements (from parent and assemblies) by assembly
    let mut elements_per_assembly: HashMap<u64, Vec<u64>> = HashMap::new();
    let mut orphan_elements: Vec<u64> = Vec::new();
    
    // Get elements directly contained in the parent
    for elem_id in parent_elem_ids {
        if let Some(elem_entity) = entities.get(&elem_id) {
            if !is_building_element(&elem_entity.ifc_type.0) {
                continue;
            }
            
            // Check if this element references an assembly
            let mut assigned_assembly: Option<u64> = None;
            for asm_id in &asm_ids {
                if element_references(elem_entity, *asm_id) {
                    assigned_assembly = Some(*asm_id);
                    break;
                }
            }
            
            // If not assigned, check if element references any assembly not yet in asm_ids
            if assigned_assembly.is_none() {
                for (asm_id, _asm_entity) in assembly_nodes.iter() {
                    if !asm_ids.contains(asm_id) && element_references(elem_entity, *asm_id) {
                        asm_ids.push(*asm_id);
                        assigned_assembly = Some(*asm_id);
                        break;
                    }
                }
            }
            
            if let Some(asm_id) = assigned_assembly {
                elements_per_assembly.entry(asm_id).or_default().push(elem_id);
            } else {
                orphan_elements.push(elem_id);
            }
        }
    }
    
    // Also add elements contained in each assembly
    for asm_id in &asm_ids {
        if let Some(asm_elem_ids) = containment_map.get(asm_id) {
            for elem_id in asm_elem_ids {
                if let Some(elem_entity) = entities.get(elem_id) {
                    if is_building_element(&elem_entity.ifc_type.0) && !elements_per_assembly.contains_key(asm_id) {
                        elements_per_assembly.entry(*asm_id).or_default().push(*elem_id);
                    }
                }
            }
        }
    }

    // Also add elements that are decomposed by each assembly (IfcRelDecomposes/IfcRelNests)
    for asm_id in &asm_ids {
        if let Some(asm_child_ids) = decomposes_map.get(asm_id) {
            for elem_id in asm_child_ids {
                if let Some(elem_entity) = entities.get(elem_id) {
                    if is_building_element(&elem_entity.ifc_type.0) {
                        let key = *asm_id;
                        if !elements_per_assembly.contains_key(&key) {
                            elements_per_assembly.entry(key).or_default().push(*elem_id);
                        } else if !elements_per_assembly.get(&key).unwrap().contains(elem_id) {
                            elements_per_assembly.get_mut(&key).unwrap().push(*elem_id);
                        }
                    }
                }
            }
        }
    }
    
    // Add assemblies with their elements
    for asm_id in &asm_ids {
        if let Some(asm_entity) = assembly_nodes.get(asm_id) {
            let mut asm_node = TreeNode::from_entity(
                *asm_id,
                &asm_entity.display_name(),
                &format!("{:?}", asm_entity.ifc_type.0),
            );
            
            // Add elements that belong to this assembly
            if let Some(elem_list) = elements_per_assembly.get(asm_id) {
                for elem_id in elem_list {
                    if let Some(elem_entity) = entities.get(elem_id) {
                        asm_node.add_child(TreeNode::from_entity(
                            *elem_id,
                            &elem_entity.display_name(),
                            &format!("{:?}", elem_entity.ifc_type.0),
                        ));
                    }
                }
            }
            
            node.add_child(asm_node);
        }
    }
    
    // Add orphan elements directly under the node
    for elem_id in orphan_elements {
        if let Some(elem_entity) = entities.get(&elem_id) {
            node.add_child(TreeNode::from_entity(
                elem_id,
                &elem_entity.display_name(),
                &format!("{:?}", elem_entity.ifc_type.0),
            ));
        }
    }
}

fn element_references(entity: &EntitySnapshot, target_id: u64) -> bool {
    for (_, value) in &entity.attributes {
        let val = value.trim();
        if val == &format!("#{}", target_id) || val.contains(&format!("#{}", target_id)) {
            return true;
        }
    }
    false
}

