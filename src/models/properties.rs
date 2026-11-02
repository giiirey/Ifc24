use serde::{Deserialize, Serialize};

/// A property set (IfcPropertySet) or quantity (IfcElementQuantity)
/// associated with an element via IfcRelDefinesByProperties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySet {
    pub name: String,
    pub definition_type: String,
    pub properties: Vec<Property>,
}

/// A single name/value property inside a property set or quantity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub value: String,
}
