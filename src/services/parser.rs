use crate::models::{EntitySnapshot, IfcType};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::Instant;
use tracing::{info, warn};

fn debug_log(msg: &str) {
    eprintln!("[PARSER] {}", msg);
    let debug_log = std::env::temp_dir().join("ifc_parser_debug.log");
    if let Ok(mut f) = File::options().append(true).open(&debug_log) {
        let _ = writeln!(f, "[PARSER] {}", msg);
    }
}

pub async fn parse_and_index(file_path: &str) -> Result<(usize, Option<String>), String> {
    let path = file_path.to_string();
    let result = tokio::task::spawn_blocking(move || {
        raw_parse(&path)
    }).await;
    
    match result {
        Ok(Ok((entities, project_name))) => {
            eprintln!("[PARSER] spawn_blocking returned {} entities", entities.len());
            
            let app = crate::state::get_state();
            let mut guard = app.model.lock().map_err(|e| e.to_string())?;
            guard.entities = entities.clone();
            
            let hierarchy = crate::services::hierarchy_builder::build_hierarchy(&entities)?;
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
    debug_log(&format!("Opening file: {}", file_path));
    
    let path = Path::new(file_path);
    let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", file_path, e))?;
    let reader = BufReader::new(file);
    
    let mut entities: HashMap<u64, EntitySnapshot> = HashMap::new();
    let mut project_name: Option<String> = None;
    let start = Instant::now();
    
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            continue;
        }
        
        if let Some(snapshot) = parse_step_entity(trimmed) {
            let id = snapshot.id;
            let is_project = matches!(snapshot.ifc_type.0, IfcType::IfcProject);
            let project_name_candidate = snapshot.name.clone();
            
            if project_name.is_none() && is_project {
                project_name = project_name_candidate;
            }
            
            entities.insert(id, snapshot);
        }
    }
    
    let elapsed = start.elapsed();
    debug_log(&format!("COMPLETE: {} entities parsed in {:.1}s", entities.len(), elapsed.as_secs_f64()));
    
    Ok((entities, project_name))
}

fn parse_step_entity(line: &str) -> Option<EntitySnapshot> {
    if !line.starts_with('#') || !line.contains('=') {
        return None;
    }
    
    let (id_part, rest) = line.split_once('=').unwrap_or(("", ""));
    let id_str = id_part.strip_prefix('#').unwrap_or(id_part);
    
    let id: u64 = match id_str.trim().parse() {
        Ok(v) => v,
        Err(_) => return None,
    };
    
    let rest = rest.trim();
    let paren_pos = rest.find('(')?;
    
    let type_str = rest[..paren_pos].trim().trim_end_matches(';');
    
    let ifc_type = parse_ifc_type(type_str);
    let wrapper = crate::models::entity::IfcTypeWrapper(ifc_type);
    
    let (global_id, attrs) = parse_attributes(&rest[paren_pos..]);
    let entity = EntitySnapshot::new(id, global_id, wrapper);
    
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
        "IFCASSEMBLY" => IfcType::IfcAssembly,
        t if t.starts_with("IFCASSEMBLY") => IfcType::IfcAssembly,
        "IFCELEMENTASSEMBLY" => IfcType::IfcElementAssembly,
        t if t.starts_with("IFCELEMENTASSEMBLY") => IfcType::IfcElementAssembly,
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
        "IFCPLATE" => IfcType::IfcPlate,
        "IFCMECHANICALFASTENER" => IfcType::IfcMechanicalFastener,
        "IFCFLOWSEGMENT" => IfcType::IfcFlowSegment,
        "IFCFLOWTERMINAL" => IfcType::IfcFlowTerminal,
        "IFCFLOWCONTROLLER" => IfcType::IfcFlowController,
        "IFCRELAGGREGATES" => IfcType::IfcRelAggregates,
        "IFCRELDECOMPOSES" => IfcType::IfcRelDecomposes,
        "IFCRELNESTS" => IfcType::IfcRelNests,
        "IFCRELCONTAINEDINSPATIALSTRUCTURE" => IfcType::IfcRelContainedInSpatialStructure,
        "IFCRELDEFINESBYPROPERTIES" => IfcType::IfcRelDefinesByProperties,
        "IFCRELASSIGNSTOGROUP" => IfcType::IfcRelAssignsToGroup,
        "IFCRELFILLSELEMENT" => IfcType::IfcRelFillsElement,
        "IFCRELVOIDSELEMENT" => IfcType::IfcRelVoidsElement,
        "IFCRELCONNECTSELEMENTS" => IfcType::IfcRelConnectsElements,
        "IFCRELCONNECTSPATHELEMENTS" => IfcType::IfcRelConnectsPathElements,
        "IFCRELREFERENCEDINSPATIALSTRUCTURE" => IfcType::IfcRelReferencedInSpatialStructure,
        "IFCLOCALPLACEMENT" => IfcType::IfcLocalPlacement,
        "IFCAXIS2PLACEMENT3D" => IfcType::IfcAxis2Placement3D,
        "IFCCARTESIANPOINT" => IfcType::IfcCartesianPoint,
        "IFCDIRECTION" => IfcType::IfcDirection,
        "IFCEXTRUDEDAREASOLID" => IfcType::IfcExtrudedAreaSolid,
        "IFCRECTANGLEPROFILEDEF" => IfcType::IfcRectangleProfileDef,
        "IFCCIRCLEPROFILEDEF" => IfcType::IfcCircleProfileDef,
        "IFCISHAPEPROFILEDEF" => IfcType::IfcIShapeProfileDef,
        "IFCARBITRARYCLOSEDPROFILEDEF" => IfcType::IfcArbitraryClosedProfileDef,
        "IFCPRODUCTDEFINITIONSHAPE" => IfcType::IfcProductDefinitionShape,
        "IFCSHAPEREPRESENTATION" => IfcType::IfcShapeRepresentation,
        "IFCPOLYLINE" => IfcType::IfcPolyline,
        "IFCLINE" => IfcType::IfcLine,
        "IFCVECTOR" => IfcType::IfcVector,
        "IFCREPRESENTATIONMAP" => IfcType::IfcRepresentationMap,
        "IFCMAPPEDITEM" => IfcType::IfcMappedItem,
        "IFCCARTESIANTRANSFORMATIONOPERATOR3D" => IfcType::IfcCartesianTransformationOperator3D,
        "IFCTRIANGULATEDFACESET" => IfcType::IfcTriangulatedFaceSet,
        "IFCPOLYGONALFACESET" => IfcType::IfcPolygonalFaceSet,
        "IFCINDEXEDPOLYGONALFACE" => IfcType::IfcIndexedPolygonalFace,
        "IFCFACETEDBREP" => IfcType::IfcFacetedBrep,
        "IFCFACEBASEDSURFACEMODEL" => IfcType::IfcFaceBasedSurfaceModel,
        "IFCCOLOURRGB" => IfcType::IfcColourRgb,
        "IFCSTYLEDITEM" => IfcType::IfcStyledItem,
        "IFCPRESENTATIONSTYLEASSIGNMENT" => IfcType::IfcPresentationStyleAssignment,
        "IFCSURFACESTYLE" => IfcType::IfcSurfaceStyle,
        "IFCSURFACESTYLERENDERING" => IfcType::IfcSurfaceStyleRendering,
        _ => IfcType::Unknown,
    }
}

fn parse_attributes(paren_content: &str) -> (String, Vec<(String, String)>) {
    let mut attrs = Vec::new();
    let content = paren_content.strip_prefix('(').unwrap_or(paren_content);
    let content = content.strip_suffix(';').unwrap_or(content);
    let content = content.strip_suffix(')').unwrap_or(content);
    
    let parts: Vec<&str> = split_attributes(content);
    
    // Global ID - это первый атрибут без "=" (имя сущности)
    let mut global_id = String::new();
    for part in &parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if part == "$" || part == ".UNDEFINED." {
            continue;
        }
        // Первый атрибут БЕЗ "=" - это имя сущности (Global ID)
        if !part.contains('=') {
            global_id = part.to_string();
            // Добавляем как первый атрибут
            attrs.push((format!("Name"), global_id.clone()));
            break;
        }
    }
    
    // Оставшиеся атрибуты с "="
    for part in &parts {
        let part = part.trim();
        if part.is_empty() {
            attrs.push((format!("attr_{}", attrs.len()), String::new()));
            continue;
        }
        if part == "$" || part == ".UNDEFINED." {
            attrs.push((format!("attr_{}", attrs.len()), part.to_string()));
            continue;
        }
        if part.contains('=') {
            let (k, v) = part.split_once('=').unwrap_or(("", part));
            attrs.push((k.trim().to_string(), v.trim().replace('"', "").trim().to_string()));
        } else {
            attrs.push((format!("attr_{}", attrs.len()), parse_simple_value(part)));
        }
    }
    
    (global_id, attrs)
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
