use crate::models::{MeshData};
use lru::LruCache;
use tracing::info;

pub struct GeometryCache {
    cache: LruCache<u64, Vec<MeshData>>,
    vram_budget_bytes: usize,
    current_usage_bytes: usize,
}

impl GeometryCache {
    pub fn new(vram_budget_mb: u64) -> Self {
        Self {
            cache: LruCache::unbounded(),
            vram_budget_bytes: (vram_budget_mb as usize) * 1024 * 1024,
            current_usage_bytes: 0,
        }
    }

    pub fn get(&mut self, entity_id: u64) -> Option<&Vec<MeshData>> {
        self.cache.get(&entity_id)
    }

    pub fn insert(&mut self, entity_id: u64, meshes: Vec<MeshData>) -> Result<(), String> {
        let size = mesh_data_size(&meshes);
        
        while self.current_usage_bytes + size > self.vram_budget_bytes {
            if let Some((evicted_id, _)) = self.cache.pop_lru() {
                self.current_usage_bytes -= mesh_data_size(&Vec::new());
                info!("Evicted geometry for entity {}", evicted_id);
            } else {
                break;
            }
        }
        
        self.current_usage_bytes += size;
        self.cache.put(entity_id, meshes);
        Ok(())
    }

    pub fn remove(&mut self, entity_id: &u64) -> Option<Vec<MeshData>> {
        if let Some(meshes) = self.cache.pop(entity_id) {
            self.current_usage_bytes -= mesh_data_size(&meshes);
            Some(meshes)
        } else { None }
    }

    pub fn contains(&self, entity_id: &u64) -> bool {
        self.cache.contains(entity_id)
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_usage_bytes = 0;
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            cached_entities: self.cache.len(),
            vram_used_bytes: self.current_usage_bytes,
            vram_budget_bytes: self.vram_budget_bytes,
            usage_percent: (self.current_usage_bytes as f64 / self.vram_budget_bytes as f64 * 100.0).round() as u8,
        }
    }

    pub fn with_frustum_culling(
        &mut self,
        entity_ids: &[u64],
        frustum_planes: [[f64; 4]; 6],
    ) -> Vec<u64> {
        let mut visible = Vec::new();
        for id in entity_ids {
            if let Some(meshes) = self.cache.get(id) {
                for mesh in meshes {
                    if mesh.bbox.intersects_frustum(&frustum_planes) {
                        visible.push(*id);
                        break;
                    }
                }
            }
        }
        visible
    }
}

fn mesh_data_size(meshes: &[MeshData]) -> usize {
    let mut total = 0;
    for mesh in meshes {
        total += mesh.vertices.len() * std::mem::size_of::<f32>();
        total += mesh.indices.len() * std::mem::size_of::<u32>();
        total += mesh.normals.len() * std::mem::size_of::<f32>();
    }
    total
}

pub struct CacheStats {
    pub cached_entities: usize,
    pub vram_used_bytes: usize,
    pub vram_budget_bytes: usize,
    pub usage_percent: u8,
}

pub const DESKTOP_VRAM_BUDGET_MB: u64 = 4096;
pub const MOBILE_VRAM_BUDGET_MB: u64 = 512;

impl Default for GeometryCache {
    fn default() -> Self {
        Self::new(DESKTOP_VRAM_BUDGET_MB)
    }
}
