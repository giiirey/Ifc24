use crate::models::{EntitySnapshot, IfcType};
use nalgebra::{Matrix4, Vector3, Vector4};
use std::collections::HashMap;

pub struct GeometryResolver<'a> {
    entities: &'a HashMap<u64, EntitySnapshot>,
}

impl<'a> GeometryResolver<'a> {
    pub fn new(entities: &'a HashMap<u64, EntitySnapshot>) -> Self {
        Self { entities }
    }

    pub fn resolve_placement(&self, entity: &EntitySnapshot) -> Option<Matrix4<f64>> {
        let placement_id = self.find_placement_ref(entity)?;
        let mut transforms = Vec::new();
        let mut current_id = Some(placement_id);

        while let Some(id) = current_id {
            if let Some(local_placement) = self.entities.get(&id) {
                if matches!(local_placement.ifc_type.0, IfcType::IfcLocalPlacement) {
                    let parent_ref = self.get_ref_attr(local_placement, 0);
                    let relative_ref = self.get_ref_attr(local_placement, 1);

                    if let Some(rel_id) = relative_ref {
                        if let Some(matrix) = self.resolve_axis2_placement(rel_id) {
                            transforms.push(matrix);
                        }
                    }

                    current_id = parent_ref;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let mut result = Matrix4::identity();
        for t in transforms.iter().rev() {
            result = t * result;
        }

        Some(result)
    }

    fn resolve_axis2_placement(&self, id: u64) -> Option<Matrix4<f64>> {
        let entity = self.entities.get(&id)?;
        if !matches!(entity.ifc_type.0, IfcType::IfcAxis2Placement3D) {
            return None;
        }

        let location = self.get_ref_attr(entity, 0)
            .and_then(|loc_id| self.extract_cartesian_point(loc_id))
            .unwrap_or(Vector3::new(0.0, 0.0, 0.0));

        let axis = self.get_ref_attr(entity, 1)
            .and_then(|dir_id| self.extract_direction(dir_id))
            .unwrap_or(Vector3::new(0.0, 0.0, 1.0));

        let ref_dir = self.get_ref_attr(entity, 2)
            .and_then(|dir_id| self.extract_direction(dir_id))
            .unwrap_or(Vector3::new(1.0, 0.0, 0.0));

        let z = axis.normalize();
        let x = ref_dir.normalize();
        let y = z.cross(&x).normalize();
        let x = y.cross(&z).normalize();

        let mut matrix = Matrix4::identity();
        matrix.set_column(0, &Vector4::new(x.x, x.y, x.z, 0.0));
        matrix.set_column(1, &Vector4::new(y.x, y.y, y.z, 0.0));
        matrix.set_column(2, &Vector4::new(z.x, z.y, z.z, 0.0));
        matrix.set_column(3, &Vector4::new(location.x, location.y, location.z, 1.0));

        Some(matrix)
    }

    fn extract_cartesian_point(&self, id: u64) -> Option<Vector3<f64>> {
        let entity = self.entities.get(&id)?;
        if !matches!(entity.ifc_type.0, IfcType::IfcCartesianPoint) {
            return None;
        }

        let coords_str = entity.attributes.iter()
            .find(|(k, _)| k == "attr_1")
            .map(|(_, v)| v.as_str())?;

        self.parse_coordinate_tuple(coords_str)
    }

    fn extract_direction(&self, id: u64) -> Option<Vector3<f64>> {
        let entity = self.entities.get(&id)?;
        if !matches!(entity.ifc_type.0, IfcType::IfcDirection) {
            return None;
        }

        let coords_str = entity.attributes.iter()
            .find(|(k, _)| k == "attr_1")
            .map(|(_, v)| v.as_str())?;

        self.parse_coordinate_tuple(coords_str)
    }

    fn parse_coordinate_tuple(&self, s: &str) -> Option<Vector3<f64>> {
        let cleaned: String = s.chars()
            .filter(|c| !c.is_whitespace() && *c != '(' && *c != ')')
            .collect();

        let parts: Vec<&str> = cleaned.split(',').collect();
        if parts.len() >= 2 {
            let x = parts[0].parse::<f64>().unwrap_or(0.0);
            let y = parts[1].parse::<f64>().unwrap_or(0.0);
            let z = if parts.len() >= 3 {
                parts[2].parse::<f64>().unwrap_or(0.0)
            } else {
                0.0
            };
            Some(Vector3::new(x, y, z))
        } else {
            None
        }
    }

    pub fn extract_dimensions(&self, entity: &EntitySnapshot) -> Option<[f64; 3]> {
        let repr_id = self.find_representation_ref(entity)?;
        let repr = self.entities.get(&repr_id)?;

        if matches!(repr.ifc_type.0, IfcType::IfcProductDefinitionShape) {
            for (key, value) in &repr.attributes {
                if key.starts_with("attr_") && value.starts_with('#') {
                    if let Ok(shape_repr_id) = value[1..].parse::<u64>() {
                        if let Some(dims) = self.extract_from_shape_representation(shape_repr_id) {
                            return Some(dims);
                        }
                    }
                }
            }

            let items_str = repr.attributes.iter()
                .find(|(k, _)| k == "attr_2")
                .map(|(_, v)| v.as_str())
                .unwrap_or("");

            for ref_str in self.extract_references_from_list(items_str) {
                if let Some(dims) = self.extract_from_shape_representation(ref_str) {
                    return Some(dims);
                }
            }
        }

        None
    }

    fn extract_from_shape_representation(&self, id: u64) -> Option<[f64; 3]> {
        let entity = self.entities.get(&id)?;

        if matches!(entity.ifc_type.0, IfcType::IfcShapeRepresentation) {
            let items_str = entity.attributes.iter()
                .find(|(k, _)| k == "attr_3" || k == "attr_2")
                .map(|(_, v)| v.as_str())
                .unwrap_or("");

            for ref_id in self.extract_references_from_list(items_str) {
                if let Some(dims) = self.extract_from_solid(ref_id) {
                    return Some(dims);
                }
            }

            for (key, value) in &entity.attributes {
                if key.starts_with("attr_") && value.starts_with('#') {
                    if let Ok(item_id) = value[1..].parse::<u64>() {
                        if let Some(dims) = self.extract_from_solid(item_id) {
                            return Some(dims);
                        }
                    }
                }
            }
        }

        None
    }

    fn extract_from_solid(&self, id: u64) -> Option<[f64; 3]> {
        let entity = self.entities.get(&id)?;

        if matches!(entity.ifc_type.0, IfcType::IfcExtrudedAreaSolid) {
            let profile_id = self.get_ref_attr(entity, 0)?;
            let depth = self.get_numeric_attr(entity, 3).unwrap_or(1.0);

            let profile = self.entities.get(&profile_id)?;

            if matches!(profile.ifc_type.0, IfcType::IfcRectangleProfileDef) {
                let x_dim = self.get_numeric_attr(profile, 3).unwrap_or(1.0);
                let y_dim = self.get_numeric_attr(profile, 4).unwrap_or(1.0);
                return Some([x_dim, y_dim, depth]);
            }

            if matches!(profile.ifc_type.0, IfcType::IfcCircleProfileDef) {
                let radius = self.get_numeric_attr(profile, 3).unwrap_or(0.5);
                return Some([radius * 2.0, radius * 2.0, depth]);
            }
        }

        None
    }

    fn find_placement_ref(&self, entity: &EntitySnapshot) -> Option<u64> {
        let placement_attrs = ["attr_5", "attr_6", "attr_7"];
        for attr_key in &placement_attrs {
            if let Some(value) = entity.attributes.iter()
                .find(|(k, _)| k == attr_key)
                .map(|(_, v)| v.as_str())
            {
                if value.starts_with('#') {
                    if let Ok(id) = value[1..].parse::<u64>() {
                        if let Some(target) = self.entities.get(&id) {
                            if matches!(target.ifc_type.0, IfcType::IfcLocalPlacement) {
                                return Some(id);
                            }
                        }
                    }
                }
            }
        }

        for (_, value) in &entity.attributes {
            if value.starts_with('#') {
                if let Ok(id) = value[1..].parse::<u64>() {
                    if let Some(target) = self.entities.get(&id) {
                        if matches!(target.ifc_type.0, IfcType::IfcLocalPlacement) {
                            return Some(id);
                        }
                    }
                }
            }
        }

        None
    }

    fn find_representation_ref(&self, entity: &EntitySnapshot) -> Option<u64> {
        let repr_attrs = ["attr_6", "attr_7", "attr_8"];
        for attr_key in &repr_attrs {
            if let Some(value) = entity.attributes.iter()
                .find(|(k, _)| k == attr_key)
                .map(|(_, v)| v.as_str())
            {
                if value.starts_with('#') {
                    if let Ok(id) = value[1..].parse::<u64>() {
                        if let Some(target) = self.entities.get(&id) {
                            if matches!(target.ifc_type.0, IfcType::IfcProductDefinitionShape) {
                                return Some(id);
                            }
                        }
                    }
                }
            }
        }

        for (_, value) in &entity.attributes {
            if value.starts_with('#') {
                if let Ok(id) = value[1..].parse::<u64>() {
                    if let Some(target) = self.entities.get(&id) {
                        if matches!(target.ifc_type.0, IfcType::IfcProductDefinitionShape) {
                            return Some(id);
                        }
                    }
                }
            }
        }

        None
    }

    fn get_ref_attr(&self, entity: &EntitySnapshot, index: usize) -> Option<u64> {
        let key = format!("attr_{}", index);
        entity.attributes.iter()
            .find(|(k, _)| k == &key)
            .and_then(|(_, v)| {
                let v = v.trim();
                if v.starts_with('#') {
                    v[1..].parse::<u64>().ok()
                } else {
                    None
                }
            })
    }

    fn get_numeric_attr(&self, entity: &EntitySnapshot, index: usize) -> Option<f64> {
        let key = format!("attr_{}", index);
        entity.attributes.iter()
            .find(|(k, _)| k == &key)
            .and_then(|(_, v)| v.trim().parse::<f64>().ok())
    }

    fn extract_references_from_list(&self, s: &str) -> Vec<u64> {
        let mut refs = Vec::new();
        let mut current = String::new();
        let mut in_ref = false;

        for ch in s.chars() {
            if ch == '#' {
                in_ref = true;
                current.clear();
            } else if in_ref {
                if ch.is_ascii_digit() {
                    current.push(ch);
                } else {
                    if let Ok(id) = current.parse::<u64>() {
                        refs.push(id);
                    }
                    current.clear();
                    in_ref = false;
                }
            }
        }

        if in_ref {
            if let Ok(id) = current.parse::<u64>() {
                refs.push(id);
            }
        }

        refs
    }
}
