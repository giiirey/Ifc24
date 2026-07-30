use crate::models::{BoundingBox, MeshData};
use crate::services::geometry_resolver::GeometryResolver;
use nalgebra::Matrix4;

pub fn extract_for_entities(entity_ids: &[u64]) -> Result<Vec<MeshData>, String> {
    let app = crate::state::get_state();
    let guard = app.model.lock().map_err(|e| e.to_string())?;

    let resolver = GeometryResolver::new(&guard.entities);
    let mut meshes = Vec::new();

    for id in entity_ids {
        if let Some(entity) = guard.entities.get(id) {
            let mesh = generate_geometry_for_type(entity, &resolver);
            meshes.push(mesh);
        }
    }
    Ok(meshes)
}

fn generate_geometry_for_type(entity: &crate::models::EntitySnapshot, resolver: &GeometryResolver) -> MeshData {
    let (fallback_size, color) = get_type_dimensions_and_color(&entity.ifc_type);

    let size = resolver.extract_dimensions(entity)
        .unwrap_or(fallback_size);

    let placement = resolver.resolve_placement(entity);

    let (center, transformed_size) = if let Some(matrix) = placement {
        let local_center = nalgebra::Vector3::new(0.0, 0.0, size[2] / 2.0);
        let world_center = matrix.transform_point(&nalgebra::Point3::from_homogeneous(
            nalgebra::Vector4::new(local_center.x, local_center.y, local_center.z, 1.0)
        ).unwrap());

        let transformed = apply_rotation_to_size(matrix, size);
        (
            [world_center.x, world_center.y, world_center.z],
            transformed
        )
    } else {
        ([0.0, 0.0, size[2] / 2.0], size)
    };

    let half = [transformed_size[0] / 2.0, transformed_size[1] / 2.0, transformed_size[2] / 2.0];
    let bbox = BoundingBox::new(
        [center[0] - half[0], center[1] - half[1], center[2] - half[2]],
        [center[0] + half[0], center[1] + half[1], center[2] + half[2]],
    );

    MeshData {
        entity_id: entity.id,
        vertices: create_box_vertices(&bbox),
        indices: vec![],
        normals: Vec::new(),
        material: crate::models::MaterialProperties {
            name: format!("{:?}_material", entity.ifc_type),
            color_diffuse: color,
            opacity: 1.0,
            roughness: 0.8,
            metalness: 0.1,
        },
        bbox,
        vertex_count: 24,
        triangle_count: 12,
    }
}

fn apply_rotation_to_size(matrix: Matrix4<f64>, size: [f64; 3]) -> [f64; 3] {
    let sx = matrix.column(0).xyz().norm();
    let sy = matrix.column(1).xyz().norm();
    let sz = matrix.column(2).xyz().norm();

    [size[0] * sx, size[1] * sy, size[2] * sz]
}

fn get_type_dimensions_and_color(ifc_type: &crate::models::IfcType) -> ([f64; 3], [f32; 3]) {
    match ifc_type {
        crate::models::IfcType::IfcWall => ([5.0, 3.0, 0.2], [0.7, 0.7, 0.7]),
        crate::models::IfcType::IfcSlab => ([10.0, 0.2, 10.0], [0.6, 0.6, 0.6]),
        crate::models::IfcType::IfcColumn => ([0.3, 3.0, 0.3], [0.5, 0.5, 0.5]),
        crate::models::IfcType::IfcBeam => ([4.0, 0.3, 0.3], [0.55, 0.55, 0.55]),
        crate::models::IfcType::IfcDoor => ([1.0, 2.1, 0.05], [0.6, 0.4, 0.2]),
        crate::models::IfcType::IfcWindow => ([1.2, 1.2, 0.05], [0.3, 0.6, 0.8]),
        crate::models::IfcType::IfcRoof => ([10.0, 0.1, 10.0], [0.4, 0.2, 0.1]),
        _ => ([2.0, 2.0, 2.0], [0.5, 0.5, 0.5]),
    }
}

fn create_box_vertices(bbox: &BoundingBox) -> Vec<f32> {
    let min = bbox.min;
    let max = bbox.max;

    vec![
        min[0] as f32, max[1] as f32, min[2] as f32,
        max[0] as f32, max[1] as f32, min[2] as f32,
        max[0] as f32, min[1] as f32, min[2] as f32,
        min[0] as f32, min[1] as f32, min[2] as f32,

        min[0] as f32, max[1] as f32, max[2] as f32,
        min[0] as f32, min[1] as f32, max[2] as f32,
        max[0] as f32, min[1] as f32, max[2] as f32,
        max[0] as f32, max[1] as f32, max[2] as f32,

        min[0] as f32, max[1] as f32, min[2] as f32,
        min[0] as f32, min[1] as f32, min[2] as f32,
        min[0] as f32, min[1] as f32, max[2] as f32,
        min[0] as f32, max[1] as f32, max[2] as f32,

        max[0] as f32, max[1] as f32, min[2] as f32,
        max[0] as f32, min[1] as f32, min[2] as f32,
        max[0] as f32, min[1] as f32, max[2] as f32,
        max[0] as f32, max[1] as f32, max[2] as f32,

        min[0] as f32, max[1] as f32, min[2] as f32,
        min[0] as f32, max[1] as f32, max[2] as f32,
        max[0] as f32, max[1] as f32, max[2] as f32,
        max[0] as f32, max[1] as f32, min[2] as f32,

        min[0] as f32, min[1] as f32, min[2] as f32,
        max[0] as f32, min[1] as f32, min[2] as f32,
        max[0] as f32, min[1] as f32, max[2] as f32,
        min[0] as f32, min[1] as f32, max[2] as f32,
    ]
}

pub fn compute_bbox(_entity_id: u64) -> Result<BoundingBox, String> {
    Ok(BoundingBox::new([-1.0; 3], [1.0; 3]))
}

pub fn simplify_geometry(lod0: &MeshData) -> MeshData {
    let vertices = &lod0.vertices;
    let step = if vertices.len() >= 9 { vertices.len() / 6 } else { 1 };
    let mut simplified_vertices = Vec::new();
    for (i, &v) in vertices.iter().enumerate() {
        if i % step == 0 {
            simplified_vertices.push(v);
        }
    }
    
    MeshData {
        entity_id: lod0.entity_id,
        vertices: simplified_vertices,
        indices: vec![],
        normals: Vec::new(),
        material: crate::models::MaterialProperties {
            name: format!("{}_simplified", lod0.material.name),
            color_diffuse: [lod0.material.color_diffuse[0] * 0.9; 3],
            opacity: lod0.material.opacity * 0.85,
            roughness: lod0.material.roughness + 0.1,
            metalness: lod0.material.metalness,
        },
        bbox: BoundingBox::new([lod0.bbox.min[0] * 0.9; 3], [lod0.bbox.max[0] * 0.9; 3]),
        vertex_count: vertices.len() / step / 3,
        triangle_count: vertices.len() / step / 9,
    }
}

pub fn lod_thresholds() -> [f64; 2] {
    [15.0, 50.0]
}
