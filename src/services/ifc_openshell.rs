use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::os::raw::c_int;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfcGeometryData {
    pub entity_id: u64,
    pub entity_type: String,
    pub entity_name: String,
    pub matrix: [f64; 16],
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub normals: Vec<f32>,
}

pub struct IfcModelWrapper {
    handle: IfcModelHandle,
}

impl IfcModelWrapper {
    pub fn open(path: &str) -> Result<Self, String> {
        let c_path = CString::new(path).map_err(|e| e.to_string())?;

        let handle = unsafe { ifc_open(c_path.as_ptr()) };

        if handle.is_null() {
            return Err(format!("Failed to open IFC file: {}", path));
        }

        Ok(Self { handle })
    }

    pub fn entity_count(&self) -> i32 {
        unsafe { ifc_entity_count(self.handle) }
    }

    pub fn entity_type(&self, id: u64) -> String {
        unsafe {
            let ptr = ifc_entity_type(self.handle, id);
            if ptr.is_null() {
                return String::new();
            }
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }

    pub fn entity_name(&self, id: u64) -> String {
        unsafe {
            let ptr = ifc_entity_name(self.handle, id);
            if ptr.is_null() {
                return String::new();
            }
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }

    pub fn entity_global_id(&self, id: u64) -> String {
        unsafe {
            let ptr = ifc_entity_global_id(self.handle, id);
            if ptr.is_null() {
                return String::new();
            }
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }

    pub fn extract_all_geometry(&self) -> Result<Vec<IfcGeometryData>, String> {
        let mut out_len: c_int = 0;

        let json_ptr = unsafe { ifc_extract_all_geometry(self.handle, &mut out_len) };

        if json_ptr.is_null() {
            return Err("Failed to extract geometry".to_string());
        }

        let json_str = unsafe {
            let slice = std::slice::from_raw_parts(json_ptr as *const u8, out_len as usize);
            String::from_utf8_lossy(slice).into_owned()
        };

        unsafe {
            ifc_geometry_free(json_ptr);
        }

        let geometries: Vec<IfcGeometryData> = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse geometry JSON: {}", e))?;

        Ok(geometries)
    }
}

impl Drop for IfcModelWrapper {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ifc_close(self.handle);
            }
        }
    }
}

unsafe impl Send for IfcModelWrapper {}
unsafe impl Sync for IfcModelWrapper {}
