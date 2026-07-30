use crate::models::{BoundingBox, MeshData, MaterialProperties};
use crate::state::get_state;
use nalgebra::{Matrix4, Vector4};

#[tauri::command]
pub fn load_geometry_batch(entity_ids: Vec<u64>) -> Result<Vec<MeshData>, String> {
    eprintln!("[GEOM] load_geometry_batch called with {} entity_ids", entity_ids.len());
    
    let app = get_state();
    let guard = app.model.lock().map_err(|e| e.to_string())?;

    if let Some(ifc_model) = &guard.ifc_model {
        eprintln!("[GEOM] Using IfcOpenShell to extract geometry...");
        let geometries = ifc_model.extract_all_geometry()?;
        eprintln!("[GEOM] IfcOpenShell returned {} geometries", geometries.len());

        let mut all_meshes = Vec::new();

        for geom in geometries {
            let matrix = Matrix4::from_column_slice(&geom.matrix);

            let mut transformed_vertices = Vec::with_capacity(geom.vertices.len());
            for chunk in geom.vertices.chunks(3) {
                if chunk.len() == 3 {
                    let v = Vector4::new(chunk[0] as f64, chunk[1] as f64, chunk[2] as f64, 1.0);
                    let transformed = matrix * v;
                    transformed_vertices.push(transformed.x as f32);
                    transformed_vertices.push(transformed.y as f32);
                    transformed_vertices.push(transformed.z as f32);
                }
            }

            let bbox = compute_bbox_from_vertices(&transformed_vertices);
            let color = get_color_for_type(&geom.entity_type);
            let vertex_count = geom.vertices.len() / 3;
            let triangle_count = geom.indices.len() / 3;

            let mesh = MeshData {
                entity_id: geom.entity_id,
                vertices: transformed_vertices,
                indices: geom.indices,
                normals: geom.normals,
                material: MaterialProperties {
                    name: format!("{}_material", geom.entity_type),
                    color_diffuse: color,
                    opacity: 1.0,
                    roughness: 0.7,
                    metalness: 0.1,
                },
                bbox,
                vertex_count,
                triangle_count,
            };

            all_meshes.push(mesh);
        }

        eprintln!("[GEOM] Returning {} meshes from IfcOpenShell", all_meshes.len());
        return Ok(all_meshes);
    }

    eprintln!("[GEOM] IfcOpenShell not available, falling back to placeholder geometry");
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

fn compute_bbox_from_vertices(vertices: &[f32]) -> BoundingBox {
    if vertices.is_empty() {
        return BoundingBox::new([0.0; 3], [0.0; 3]);
    }

    let mut min = [f64::MAX; 3];
    let mut max = [f64::MIN; 3];

    for chunk in vertices.chunks(3) {
        if chunk.len() == 3 {
            for i in 0..3 {
                let v = chunk[i] as f64;
                if v < min[i] { min[i] = v; }
                if v > max[i] { max[i] = v; }
            }
        }
    }

    BoundingBox::new(min, max)
}

fn get_color_for_type(entity_type: &str) -> [f32; 3] {
    match entity_type {
        "IfcWall" => [0.7, 0.7, 0.7],
        "IfcSlab" => [0.6, 0.6, 0.6],
        "IfcColumn" => [0.5, 0.5, 0.5],
        "IfcBeam" => [0.55, 0.55, 0.55],
        "IfcDoor" => [0.6, 0.4, 0.2],
        "IfcWindow" => [0.3, 0.6, 0.8],
        "IfcRoof" => [0.4, 0.2, 0.1],
        _ => [0.5, 0.5, 0.5],
    }
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
