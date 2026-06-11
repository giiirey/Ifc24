use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::models::{EntitySnapshot, TreeNode};

pub struct ModelState {
    pub entities: HashMap<u64, EntitySnapshot>,
    pub hierarchy_root: Option<TreeNode>,
    pub model_path: Option<String>,
    pub mesh_cache: HashMap<u64, Vec<crate::models::MeshData>>,
}

impl ModelState {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            hierarchy_root: None,
            model_path: None,
            mesh_cache: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.entities.clear();
        self.hierarchy_root = None;
        self.model_path = None;
        self.mesh_cache.clear();
    }
}

pub struct AppState {
    pub model: Mutex<ModelState>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            model: Mutex::new(ModelState::new()),
        }
    }
}

static mut GLOBAL_STATE: Option<Arc<AppState>> = None;

pub fn get_state() -> Arc<AppState> {
    unsafe {
        if GLOBAL_STATE.is_none() {
            GLOBAL_STATE = Some(Arc::new(AppState::new()));
        }
    }
    unsafe { GLOBAL_STATE.as_ref().unwrap().clone() }
}
