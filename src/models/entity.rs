use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub id: u64,
    pub global_id: String,
    pub ifc_type: IfcTypeWrapper,
    pub name: Option<String>,
    pub description: Option<String>,
    pub attributes: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfcTypeWrapper(pub IfcType);

impl serde::Serialize for IfcTypeWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("IfcType", 1)?;
        state.serialize_field("name", &format!("{:?}", self.0))?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for IfcTypeWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = IfcTypeWrapper;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("IfcType")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where E: serde::de::Error {
                Ok(IfcTypeWrapper(parse_ifc_type_from_str(v)))
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

impl std::ops::Deref for IfcTypeWrapper {
    type Target = IfcType;
    fn deref(&self) -> &Self::Target { &self.0 }
}

fn parse_ifc_type_from_str(s: &str) -> IfcType {
    match s {
        "IfcProject" => IfcType::IfcProject,
        "IfcSite" => IfcType::IfcSite,
        "IfcBuilding" => IfcType::IfcBuilding,
        "IfcBuildingStorey" => IfcType::IfcBuildingStorey,
        "IfcWall" => IfcType::IfcWall,
        "IfcSlab" => IfcType::IfcSlab,
        "IfcAssembly" => IfcType::IfcAssembly,
        "IfcElementAssembly" => IfcType::IfcElementAssembly,
        "IfcColumn" => IfcType::IfcColumn,
        "IfcBeam" => IfcType::IfcBeam,
        "IfcDoor" => IfcType::IfcDoor,
        "IfcWindow" => IfcType::IfcWindow,
        "IfcStair" => IfcType::IfcStair,
        "IfcRailing" => IfcType::IfcRailing,
        "IfcRoof" => IfcType::IfcRoof,
        "IfcCurtainWall" => IfcType::IfcCurtainWall,
        "IfcSpace" => IfcType::IfcSpace,
        "IfcOpeningElement" => IfcType::IfcOpeningElement,
        "IfcFurniture" => IfcType::IfcFurniture,
        "IfcPlate" => IfcType::IfcPlate,
        "IfcMechanicalFastener" => IfcType::IfcMechanicalFastener,
        "IfcFlowSegment" => IfcType::IfcFlowSegment,
        "IfcFlowTerminal" => IfcType::IfcFlowTerminal,
        "IfcFlowController" => IfcType::IfcFlowController,
        "IfcRelAggregates" => IfcType::IfcRelAggregates,
        "IfcRelDecomposes" => IfcType::IfcRelDecomposes,
        "IfcRelNests" => IfcType::IfcRelNests,
        "IfcRelDefinesByProperties" => IfcType::IfcRelDefinesByProperties,
        "IfcRelAssignsToGroup" => IfcType::IfcRelAssignsToGroup,
        "IfcRelContainedInSpatialStructure" => IfcType::IfcRelContainedInSpatialStructure,
        "IfcRelFillsElement" => IfcType::IfcRelFillsElement,
        "IfcRelVoidsElement" => IfcType::IfcRelVoidsElement,
        "IfcRelConnectsElements" => IfcType::IfcRelConnectsElements,
        "IfcRelConnectsPathElements" => IfcType::IfcRelConnectsPathElements,
        "IfcRelReferencedInSpatialStructure" => IfcType::IfcRelReferencedInSpatialStructure,
        _ => IfcType::Unknown,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum IfcType {
    IfcProject,
    IfcSite,
    IfcBuilding,
    IfcBuildingStorey,
    IfcWall,
    IfcSlab,
    IfcAssembly,
    IfcElementAssembly,
    IfcColumn,
    IfcBeam,
    IfcDoor,
    IfcWindow,
    IfcStair,
    IfcRailing,
    IfcRoof,
    IfcCurtainWall,
    IfcSpace,
    IfcOpeningElement,
    IfcFurniture,
    IfcPlate,
    IfcMechanicalFastener,
    IfcFlowSegment,
    IfcFlowTerminal,
    IfcFlowController,
    IfcDistributionPort,
    IfcStructuralItem,
    IfcUnit,
    IfcSIUnit,
    IfcActor,
    IfcPerson,
    IfcOrganization,
    IfcApplication,
    IfcControl,
    IfcTimeControl,
    IfcDocumentInfo,
    IfcDocumentReference,
    IfcRelAggregates,
    IfcRelDecomposes,
    IfcRelNests,
    IfcRelDefinesByProperties,
    IfcRelAssignsToGroup,
    IfcRelContainedInSpatialStructure,
    IfcRelFillsElement,
    IfcRelVoidsElement,
    IfcRelConnectsElements,
    IfcRelConnectsPathElements,
    IfcRelReferencedInSpatialStructure,
    #[serde(other)]
    Unknown
}

impl EntitySnapshot {
    pub fn new(id: u64, global_id: String, ifc_type: IfcTypeWrapper) -> Self {
        Self {
            id,
            global_id,
            ifc_type,
            name: None,
            description: None,
            attributes: Vec::new(),
        }
    }

    pub fn with_attributes(mut self, attributes: Vec<(String, String)>) -> Self {
        for (key, value) in &attributes {
            match key.as_str() {
                "Name" => self.name = Some(value.clone()),
                "Description" => self.description = Some(value.clone()),
                _ => {}
            }
        }
        self.attributes = attributes;
        self
    }

    pub fn is_spatial_element(&self) -> bool {
        matches!(
            self.ifc_type.0,
            IfcType::IfcProject
                | IfcType::IfcSite
                | IfcType::IfcBuilding
                | IfcType::IfcBuildingStorey
                | IfcType::IfcSpace
        )
    }

    pub fn is_building_element(&self) -> bool {
        matches!(
            self.ifc_type.0,
            IfcType::IfcWall
                | IfcType::IfcSlab
                | IfcType::IfcColumn
                | IfcType::IfcBeam
                | IfcType::IfcDoor
                | IfcType::IfcWindow
                | IfcType::IfcStair
                | IfcType::IfcRailing
                | IfcType::IfcRoof
                | IfcType::IfcCurtainWall
        )
    }

    pub fn type_icon(&self) -> &str {
        match self.ifc_type.0 {
            IfcType::IfcProject => "🏢",
            IfcType::IfcSite => "🌍",
            IfcType::IfcBuilding => "🏗️",
            IfcType::IfcBuildingStorey => "📐",
            IfcType::IfcSpace => "📦",
            IfcType::IfcWall => "🧱",
            IfcType::IfcSlab => "▬",
            IfcType::IfcColumn => "|",
            IfcType::IfcBeam => "--",
            IfcType::IfcDoor => "🚪",
            IfcType::IfcWindow => "🪟",
            IfcType::IfcStair => "🪜",
            IfcType::IfcRailing => "|-",
            IfcType::IfcRoof => "△",
            IfcType::IfcAssembly => "📦",
            IfcType::IfcElementAssembly => "📦",
            IfcType::IfcPlate => "▬",
            IfcType::IfcMechanicalFastener => "•",
            _ => "•",
        }
    }

    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| self.global_id.clone())
    }
}
