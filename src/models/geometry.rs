use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshData {
    pub entity_id: u64,
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub normals: Vec<f32>,
    pub material: MaterialProperties,
    pub bbox: BoundingBox,
    pub vertex_count: usize,
    pub triangle_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialProperties {
    pub name: String,
    pub color_diffuse: [f32; 3],
    pub opacity: f32,
    pub roughness: f32,
    pub metalness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [f64; 3],
    pub max: [f64; 3],
    pub center: [f64; 3],
    pub size: [f64; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodGeometry {
    pub entity_id: u64,
    pub lod0: Option<MeshData>,
    pub lod1: Option<MeshData>,
    pub lod2_bounding_box: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkResult {
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub meshes: Vec<MeshData>,
}

impl MeshData {
    pub fn volume(&self) -> f64 {
        let dx = self.bbox.size[0];
        let dy = self.bbox.size[1];
        let dz = self.bbox.size[2];
        dx * dy * dz
    }

    pub fn surface_area(&self) -> f64 {
        let dx = self.bbox.size[0];
        let dy = self.bbox.size[1];
        let dz = self.bbox.size[2];
        2.0 * (dx * dy + dy * dz + dz * dx)
    }
}

impl BoundingBox {
    pub fn new(min: [f64; 3], max: [f64; 3]) -> Self {
        let center = [
            (min[0] + max[0]) / 2.0,
            (min[1] + max[1]) / 2.0,
            (min[2] + max[2]) / 2.0,
        ];
        let size = [
            max[0] - min[0],
            max[1] - min[1],
            max[2] - min[2],
        ];
        Self { min, max, center, size }
    }

    pub fn intersects_frustum(&self, frustum_planes: &[[f64; 4]; 6]) -> bool {
        for plane in frustum_planes {
            let d_min = self.min[0] * plane[0] + self.min[1] * plane[1] + self.min[2] * plane[2];
            let d_max = self.max[0] * plane[0] + self.max[1] * plane[1] + self.max[2] * plane[2];
            
            let distance = if plane[0] > 0.0 { self.min[0] } else { self.max[0] } * plane[0]
                         + if plane[1] > 0.0 { self.min[1] } else { self.max[1] } * plane[1]
                         + if plane[2] > 0.0 { self.min[2] } else { self.max[2] } * plane[2];
            
            if distance < -plane[3] {
                // Check if box is entirely outside this plane
                let d_max_for_plane = 
                    if plane[0] > 0.0 { self.max[0] } else { self.min[0] } * plane[0]
                  + if plane[1] > 0.0 { self.max[1] } else { self.min[1] } * plane[1]
                  + if plane[2] > 0.0 { self.max[2] } else { self.min[2] } * plane[2];

                if d_max_for_plane < -plane[3] {
                    return false;
                }
            }
        }
        true
    }
}
