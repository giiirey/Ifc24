use crate::models::{EntitySnapshot, IfcType};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Instant;
use tracing::{info, warn};

pub async fn parse_and_index(file_path: &str) -> Result<(usize, Option<String>), String> {
    let path = file_path.to_string();
    let result = tokio::task::spawn_blocking(move || raw_parse(&path)).await;
    
    match result {
        Ok(Ok((entities, project_name))) => {
            let app = crate::state::get_state();
            let mut guard = app.model.lock().map_err(|e| e.to_string())?;
            guard.entities = entities.clone();
            
            let hierarchy = crate::services::hierarchy_builder::build_hierarchy(&entities)?;
            let app = crate::state::get_state();
            let mut guard = app.model.lock().map_err(|e| e.to_string())?;
            guard.hierarchy_root = Some(hierarchy);
            
            create_index_file(file_path).map_err(|e| e.to_string())?;
            info!("Parsed {} entities from {}", entities.len(), file_path);
            Ok((entities.len(), project_name))
        }
        Ok(Err(e)) => Err(e),
        Err(e) => Err(format!("Spawn error: {}", e)),
    }
}

fn raw_parse(file_path: &str) -> Result<(HashMap<u64, EntitySnapshot>, Option<String>), String> {
    let path = Path::new(file_path);
    let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", file_path, e))?;
    let reader = BufReader::new(file);
    
    let mut entities: HashMap<u64, EntitySnapshot> = HashMap::new();
    let mut project_name: Option<String> = None;
    let start = Instant::now();
    
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
        let trimmed = line.trim();
        
        if let Some(snapshot) = parse_step_entity(trimmed) {
            let id = snapshot.id;
            entities.insert(id, snapshot);
            
            if project_name.is_none() {
                if let Some(ent) = entities.get(&id) {
                    if matches!(ent.ifc_type.0, IfcType::IfcProject) {
                        project_name = ent.name.clone();
                    }
                }
            }
        }
    }
    
    let elapsed = start.elapsed();
    info!("Step entity parsing: {} entities in {:.1}s", entities.len(), elapsed.as_secs_f64());
    
    Ok((entities, project_name))
}

fn parse_step_entity(line: &str) -> Option<EntitySnapshot> {
    if !line.starts_with('#') || !line.contains('=') {
        return None;
    }
    
    let (id_part, rest) = line.split_once('=').unwrap_or(("", ""));
    let id: u64 = id_part[1..].trim().parse().ok()?;
    
    let rest = rest.trim();
    let paren_pos = rest.find('(')?;
    let type_str = rest[..paren_pos].trim();
    
    let ifc_type = parse_ifc_type(type_str);
    let wrapper = crate::models::entity::IfcTypeWrapper(ifc_type);
    let entity = EntitySnapshot::new(id, String::new(), wrapper);
    let attrs = parse_attributes(&rest[paren_pos..]);
    
    Some(entity.with_attributes(attrs))
}

fn parse_ifc_type(s: &str) -> IfcType {
    match s {
        "IFCPROJECT" => IfcType::IfcProject,
        "IFCSITE" => IfcType::IfcSite,
        "IFCBUILDING" => IfcType::IfcBuilding,
        "IFCBUILDINGSTOREY" => IfcType::IfcBuildingStorey,
        "IFCWALL" => IfcType::IfcWall,
        "IFCSLAB" => IfcType::IfcSlab,
        "IFCCOLUMN" => IfcType::IfcColumn,
        "IFCBEAM" => IfcType::IfcBeam,
        "IFCDOOR" => IfcType::IfcDoor,
        "IFCWINDOW" => IfcType::IfcWindow,
        "IFCSTAIR" => IfcType::IfcStair,
        "IFCRAILING" => IfcType::IfcRailing,
        "IFCROOF" => IfcType::IfcRoof,
        "IFCCURTAINWALL" => IfcType::IfcCurtainWall,
        "IFCSPACE" => IfcType::IfcSpace,
        "IFCOPENINGELEMENT" => IfcType::IfcOpeningElement,
        "IFCFURNITURE" => IfcType::IfcFurniture,
        "IFCFLOWSEGMENT" => IfcType::IfcFlowSegment,
        "IFCFLOWTERMINAL" => IfcType::IfcFlowTerminal,
        "IFCFLOWCONTROLLER" => IfcType::IfcFlowController,
        _ => IfcType::Unknown,
    }
}

fn parse_attributes(paren_content: &str) -> Vec<(String, String)> {
    let mut attrs = Vec::new();
    let content = paren_content.strip_prefix('(').unwrap_or(paren_content);
    let content = content.strip_suffix(')').unwrap_or(content);
    
    let parts: Vec<&str> = split_attributes(content);
    for part in parts {
        let part = part.trim();
        if part.is_empty() || part == "$" || part == ".UNDEFINED." {
            continue;
        }
        if part.contains('=') {
            let (k, v) = part.split_once('=').unwrap_or(("", part));
            attrs.push((k.trim().to_string(), v.trim().replace('"', "").trim().to_string()));
        } else {
            attrs.push((format!("attr_{}", attrs.len()), parse_simple_value(part)));
        }
    }
    attrs
}

fn split_attributes(content: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut current_start = 0;
    let chars: Vec<char> = content.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&content[current_start..i]);
                current_start = i + 1;
            }
            _ => {}
        }
    }
    if current_start < chars.len() {
        parts.push(&content[current_start..]);
    }
    parts
}

fn parse_simple_value(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('(') && s.ends_with(')') {
        return s[1..s.len()-1].trim().to_string();
    }
    s.replace('"', "")
}

fn create_index_file(file_path: &str) -> std::io::Result<()> {
    let path = Path::new(file_path);
    if let Some(index_dir) = path.parent() {
        let idx_path = index_dir.join(format!("{}.idx", path.file_stem().unwrap_or_default().to_string_lossy()));
        std::fs::write(&idx_path, format!("# Index file for {}\n# Created: {}\n", 
            path.file_name().unwrap_or_default().to_string_lossy(),
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        ))?;
    }
    Ok(())
}
