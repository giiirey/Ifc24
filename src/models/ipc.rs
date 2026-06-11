use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub path: String,
    pub entity_count: usize,
    pub project_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadGeometryRequest {
    pub entity_ids: Vec<u64>,
    pub lod_level: LodLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LodLevel {
    Full,
    Simplified,
    BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub ifc_type_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub results: Vec<u64>,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrustumQuery {
    pub frustum_planes: [[f64; 4]; 6],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub phase: String,
    pub progress: u8,
    pub message: Option<String>,
}
