use ifctauri_lib::models::entity::IfcType;
use ifctauri_lib::services::hierarchy_builder;
use ifctauri_lib::services::parser;

// Regression test for element selection from the viewport: every product element
// must have a node in the tree, otherwise the tree cannot expand to it.
//
// In samples/AC20-Institute-Var-2.ifc:
//   #198367  IfcOpeningElement 'Fenster-002' (global id 0vi5mKOu_s6IuSmGyDRNU6)
//            is not contained in a storey; it belongs to wall #197961 through
//            IfcRelVoidsElement #198370.
//   #197961  IFCWALLSTANDARDCASE (an IfcWall subtype, previously parsed as Unknown).
#[test]
fn opening_element_is_placed_under_its_parent_wall() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/samples/AC20-Institute-Var-2.ifc");
    let (entities, _) = parser::raw_parse(&path).expect("parse sample file");

    let root = hierarchy_builder::build_hierarchy(&entities).expect("build hierarchy");

    let opening = root.find_by_id(198367).expect("opening element must be present in the tree");
    assert_eq!(opening.ifc_type, "IfcOpeningElement");

    let wall = root.find_by_id(197961).expect("standard-case wall must be present in the tree");
    assert_eq!(wall.ifc_type, "IfcWall");
    assert!(
        wall.children.iter().any(|c| c.id == 198367),
        "opening must be placed under its parent wall"
    );
}

#[test]
fn parser_maps_building_element_subtypes_but_not_type_objects() {
    // Product subtypes must resolve to their base family so they appear in the tree.
    assert_eq!(parser::parse_ifc_type("IFCWALLSTANDARDCASE"), IfcType::IfcWall);
    assert_eq!(parser::parse_ifc_type("IFCSLAB"), IfcType::IfcSlab);

    // Property sets and type objects must stay Unknown so they don't pollute the tree.
    assert_eq!(parser::parse_ifc_type("IFCWALLTYPE"), IfcType::Unknown);
    assert_eq!(parser::parse_ifc_type("IFCDOORLININGPROPERTIES"), IfcType::Unknown);
    assert_eq!(parser::parse_ifc_type("IFCDOORPANELPROPERTIES"), IfcType::Unknown);
}
