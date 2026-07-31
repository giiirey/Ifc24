use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: u64,
    pub name: String,
    pub ifc_type: String,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
    pub has_geometry: bool,
}

impl TreeNode {
    pub fn from_entity(id: u64, name: &str, ifc_type: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            ifc_type: ifc_type.to_string(),
            children: Vec::new(),
            expanded: false,
            has_geometry: false,
        }
    }

    pub fn add_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }

    pub fn flatten(&self) -> Vec<&TreeNode> {
        let mut result = vec![self];
        for child in &self.children {
            result.extend(child.flatten());
        }
        result
    }

    pub fn find_by_id(&self, id: u64) -> Option<&TreeNode> {
        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_by_id(id) {
                return Some(found);
            }
        }
        None
    }

    pub fn count_leaves(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            self.children.iter().map(|c| c.count_leaves()).sum()
        }
    }

    pub fn count_all_nodes(&self) -> usize {
        1 + self.children.iter().map(|c| c.count_all_nodes()).sum::<usize>()
    }

    pub fn is_building_storey(&self) -> bool {
        self.ifc_type == "IfcBuildingStorey"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyEdge {
    pub parent_id: u64,
    pub child_id: u64,
    pub relationship_type: String,
}
