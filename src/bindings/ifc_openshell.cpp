#include "ifc_openshell.h"
#include <cstdint>
#include <cstring>
#include <string>
#include <vector>
#include <memory>
#include <sstream>
#include <iostream>

#ifdef IFC_OPENSHELL_AVAILABLE
#include <ifcparse/IfcFile.h>
#include <ifcparse/IfcLogger.h>
#include <ifcgeom/Iterator.h>
#include <ifcgeom/IfcGeomElement.h>
#include <ifcgeom/IfcGeomRepresentation.h>
#include <ifcgeom/kernels/opencascade/OpenCascadeKernel.h>
#endif

struct IfcModel {
#ifdef IFC_OPENSHELL_AVAILABLE
    std::unique_ptr<IfcParse::IfcFile> file;
#endif
    bool initialized = false;
};

extern "C" {

IfcModelHandle ifc_open(const char* path) {
#ifdef IFC_OPENSHELL_AVAILABLE
    try {
        IfcModel* model = new IfcModel();
        model->file = std::make_unique<IfcParse::IfcFile>();

        if (!model->file->initialize(path)) {
            std::cerr << "[IFC] Failed to open file: " << path << std::endl;
            delete model;
            return nullptr;
        }

        model->initialized = true;
        return static_cast<void*>(model);
    } catch (const std::exception& e) {
        std::cerr << "[IFC] Exception opening file: " << e.what() << std::endl;
        return nullptr;
    }
#else
    (void)path;
    IfcModel* model = new IfcModel();
    model->initialized = true;
    return static_cast<void*>(model);
#endif
}

void ifc_close(IfcModelHandle handle) {
    if (handle) {
        IfcModel* model = static_cast<IfcModel*>(handle);
        delete model;
    }
}

int32_t ifc_entity_count(IfcModelHandle handle) {
#ifdef IFC_OPENSHELL_AVAILABLE
    if (!handle) return 0;
    IfcModel* model = static_cast<IfcModel*>(handle);
    if (!model->initialized || !model->file) return 0;

    try {
        auto products = model->file->instances_by_type("IfcProduct");
        return products ? products->size() : 0;
    } catch (...) {
        return 0;
    }
#else
    (void)handle;
    return 0;
#endif
}

int ifc_get_entities_chunk(IfcModelHandle handle, int start, int count, void* output_buffer, int buffer_size) {
    (void)handle;
    (void)start;
    (void)count;
    (void)output_buffer;
    (void)buffer_size;
    return 0;
}

const char* ifc_entity_type(IfcModelHandle handle, IfcEntityId id) {
    static thread_local std::string result;
#ifdef IFC_OPENSHELL_AVAILABLE
    if (!handle) {
        result = "Unknown";
        return result.c_str();
    }

    IfcModel* model = static_cast<IfcModel*>(handle);
    try {
        auto entity = model->file->instance_by_id(id);
        if (entity) {
            result = entity->declaration().name();
            return result.c_str();
        }
    } catch (...) {}
#else
    (void)handle;
    (void)id;
#endif

    result = "Unknown";
    return result.c_str();
}

const char* ifc_entity_name(IfcModelHandle handle, IfcEntityId id) {
    static thread_local std::string name;
#ifdef IFC_OPENSHELL_AVAILABLE
    IfcModel* model = static_cast<IfcModel*>(handle);
    try {
        auto entity = model->file->instance_by_id(id);
        if (entity) {
            auto* base_entity = entity->as<IfcUtil::IfcBaseEntity>();
            if (base_entity) {
                name = base_entity->get_value<std::string>("Name", "");
                return name.c_str();
            }
        }
    } catch (...) {}
#else
    (void)handle;
    (void)id;
#endif
    name = "";
    return name.c_str();
}

#ifdef IFC_OPENSHELL_AVAILABLE
namespace {
// Convert an IfcParse AttributeValue (typed / select value) into a display string.
std::string argument_to_display_string(const AttributeValue& value) {
    if (value.isNull()) {
        return "";
    }
    std::string str_value;
    auto type = value.type();
    switch (type) {
        case IfcUtil::Argument_INT:
            str_value = std::to_string((int)value);
            break;
        case IfcUtil::Argument_DOUBLE: {
            double d = (double)value;
            str_value = std::to_string(d);
            break;
        }
        case IfcUtil::Argument_STRING:
            str_value = (std::string)value;
            break;
        case IfcUtil::Argument_BOOL:
            str_value = (bool)value ? "true" : "false";
            break;
        case IfcUtil::Argument_LOGICAL: {
            boost::logic::tribool tb = (boost::logic::tribool)value;
            if (tb == true) {
                str_value = "true";
            } else if (tb == false) {
                str_value = "false";
            } else {
                str_value = "unknown";
            }
            break;
        }
        case IfcUtil::Argument_ENTITY_INSTANCE: {
            auto* ref = (IfcUtil::IfcBaseClass*)value;
            if (ref) {
                str_value = "#" + std::to_string(ref->id());
            }
            break;
        }
        case IfcUtil::Argument_AGGREGATE_OF_INT: {
            auto vec = (std::vector<int>)value;
            str_value = "(";
            for (size_t j = 0; j < vec.size(); j++) {
                if (j > 0) str_value += ",";
                str_value += std::to_string(vec[j]);
            }
            str_value += ")";
            break;
        }
        case IfcUtil::Argument_AGGREGATE_OF_DOUBLE: {
            auto vec = (std::vector<double>)value;
            str_value = "(";
            for (size_t j = 0; j < vec.size(); j++) {
                if (j > 0) str_value += ",";
                str_value += std::to_string(vec[j]);
            }
            str_value += ")";
            break;
        }
        case IfcUtil::Argument_AGGREGATE_OF_STRING: {
            auto vec = (std::vector<std::string>)value;
            str_value = "(";
            for (size_t j = 0; j < vec.size(); j++) {
                if (j > 0) str_value += ",";
                str_value += vec[j];
            }
            str_value += ")";
            break;
        }
        case IfcUtil::Argument_AGGREGATE_OF_ENTITY_INSTANCE: {
            auto agg = (boost::shared_ptr<aggregate_of_instance>)value;
            str_value = "(";
            if (agg) {
                bool first = true;
                for (auto it = agg->begin(); it != agg->end(); ++it) {
                    if (!first) str_value += ",";
                    first = false;
                    if (*it) {
                        str_value += "#" + std::to_string((*it)->id());
                    }
                }
            }
            str_value += ")";
            break;
        }
        case IfcUtil::Argument_AGGREGATE_OF_AGGREGATE_OF_INT: {
            auto vec = (std::vector<std::vector<int>>)value;
            str_value = "(";
            for (size_t j = 0; j < vec.size(); j++) {
                if (j > 0) str_value += ",";
                str_value += "(";
                for (size_t k = 0; k < vec[j].size(); k++) {
                    if (k > 0) str_value += ",";
                    str_value += std::to_string(vec[j][k]);
                }
                str_value += ")";
            }
            str_value += ")";
            break;
        }
        case IfcUtil::Argument_AGGREGATE_OF_AGGREGATE_OF_DOUBLE: {
            auto vec = (std::vector<std::vector<double>>)value;
            str_value = "(";
            for (size_t j = 0; j < vec.size(); j++) {
                if (j > 0) str_value += ",";
                str_value += "(";
                for (size_t k = 0; k < vec[j].size(); k++) {
                    if (k > 0) str_value += ",";
                    str_value += std::to_string(vec[j][k]);
                }
                str_value += ")";
            }
            str_value += ")";
            break;
        }
        case IfcUtil::Argument_AGGREGATE_OF_AGGREGATE_OF_ENTITY_INSTANCE: {
            auto agg = (boost::shared_ptr<aggregate_of_aggregate_of_instance>)value;
            str_value = "(";
            if (agg) {
                bool first_group = true;
                for (auto outer_it = agg->begin(); outer_it != agg->end(); ++outer_it) {
                    const auto& inner = *outer_it;
                    if (!first_group) str_value += ",";
                    first_group = false;
                    str_value += "(";
                    for (size_t k = 0; k < inner.size(); k++) {
                        if (k > 0) str_value += ",";
                        if (inner[k]) {
                            str_value += "#" + std::to_string(inner[k]->id());
                        }
                    }
                    str_value += ")";
                }
            }
            str_value += ")";
            break;
        }
        case IfcUtil::Argument_ENUMERATION: {
            auto enum_ref = (EnumerationReference)value;
            str_value = enum_ref.value();
            break;
        }
        default:
            str_value = "";
            break;
    }
    return str_value;
}

// True when the referenced entity is an IFC "measure" type (IfcBoolean,
// IfcReal, IfcLengthMeasure, ...) whose scalar payload lives in `Value`.
bool is_measure_entity(IfcUtil::IfcBaseClass* ref) {
    if (!ref) return false;
    const std::string& name = ref->declaration().name();
    return name.find("Measure") != std::string::npos
        || name == "IfcBoolean" || name == "IfcInteger" || name == "IfcReal"
        || name == "IfcLabel" || name == "IfcText" || name == "IfcIdentifier"
        || name.find("IfcTime") == 0;
}

// Resolve a measure entity reference to the string of its scalar value.
bool resolve_measure_value(IfcUtil::IfcBaseClass* ref, std::string& out) {
    if (!is_measure_entity(ref)) return false;
    try {
        // Measure types have a single parameter, Value, at index 0.
        AttributeValue v = ref->get_attribute_value(0);
        if (v.isNull()) return false;
        out = argument_to_display_string(v);
        return true;
    } catch (...) {
        return false;
    }
}

// Display string for a property value: resolves measure entity references
// (e.g. IfcBoolean(.T.)) to their scalar value, otherwise falls back to the
// generic Argument rendering.
std::string property_value_to_display(const AttributeValue& value) {
    if (value.type() == IfcUtil::Argument_ENTITY_INSTANCE) {
        auto* ref = (IfcUtil::IfcBaseClass*)value;
        if (ref) {
            std::string resolved;
            if (resolve_measure_value(ref, resolved)) return resolved;
            return "#" + std::to_string(ref->id());
        }
        return "";
    }
    return argument_to_display_string(value);
}

// Resolve a quantity entity (IfcPropertyQuantity, IfcQuantityLength,
// IfcQuantityArea, ...) to the display string of its scalar value.
// Each IfcQuantity* subtype stores its value under a differently named
// attribute (LengthValue, AreaValue, NominalValue, ...), so scan the
// attributes and take the first numeric or measure-reference value;
// Name/Description are strings and Unit is an IfcUnit reference, so
// both are skipped.
bool quantity_to_display(IfcUtil::IfcBaseEntity* q_entity, std::string& out) {
    const IfcParse::entity* ent_decl = q_entity->declaration().as_entity();
    if (!ent_decl) return false;
    const auto attrs = ent_decl->all_attributes();
    for (size_t i = 0; i < attrs.size(); ++i) {
        try {
            AttributeValue v = q_entity->get_attribute_value(i);
            if (v.isNull()) continue;
            if (v.type() == IfcUtil::Argument_ENTITY_INSTANCE) {
                auto* ref = (IfcUtil::IfcBaseClass*)v;
                if (ref && is_measure_entity(ref)) {
                    std::string resolved;
                    if (resolve_measure_value(ref, resolved)) {
                        out = resolved;
                        return true;
                    }
                }
            } else if (v.type() == IfcUtil::Argument_DOUBLE || v.type() == IfcUtil::Argument_INT) {
                out = argument_to_display_string(v);
                return true;
            }
        } catch (...) {}
    }
    return false;
}
} // namespace
#endif

char* ifc_entity_attributes(IfcModelHandle handle, IfcEntityId id, int* out_len) {
    const char* empty = "[]";
#ifdef IFC_OPENSHELL_AVAILABLE
    if (!handle) {
        *out_len = strlen(empty);
        char* result = new char[*out_len + 1];
        memcpy(result, empty, *out_len + 1);
        return result;
    }

    IfcModel* model = static_cast<IfcModel*>(handle);
    if (!model->initialized || !model->file) {
        *out_len = strlen(empty);
        char* result = new char[*out_len + 1];
        memcpy(result, empty, *out_len + 1);
        return result;
    }

    try {
        auto entity = model->file->instance_by_id(static_cast<int>(id));
        if (!entity) {
            *out_len = strlen(empty);
            char* result = new char[*out_len + 1];
            memcpy(result, empty, *out_len + 1);
            return result;
        }

        auto* base_entity = entity->as<IfcUtil::IfcBaseEntity>();
        if (!base_entity) {
            *out_len = strlen(empty);
            char* result = new char[*out_len + 1];
            memcpy(result, empty, *out_len + 1);
            return result;
        }

        const auto& decl = base_entity->declaration();
        const auto* entity_decl = decl.as_entity();
        if (!entity_decl) {
            *out_len = strlen(empty);
            char* result = new char[*out_len + 1];
            memcpy(result, empty, *out_len + 1);
            return result;
        }

        const auto all_attrs = entity_decl->all_attributes();

        std::ostringstream json;
        json << "[";
        for (size_t i = 0; i < all_attrs.size(); i++) {
            if (i > 0) json << ",";
            const auto* attr = all_attrs[i];
            const std::string& attr_name = attr->name();

            json << "{\"name\":\"";
            for (char c : attr_name) {
                if (c == '"') json << "\\\"";
                else if (c == '\\') json << "\\\\";
                else json << c;
            }
            json << "\",\"value\":\"";

            try {
                auto value = base_entity->get(attr_name);
                if (!value.isNull()) {
                    std::string str_value = argument_to_display_string(value);
                    for (char c : str_value) {
                        if (c == '"') json << "\\\"";
                        else if (c == '\\') json << "\\\\";
                        else if (c == '\n') json << "\\n";
                        else if (c == '\r') json << "\\r";
                        else if (c == '\t') json << "\\t";
                        else json << c;
                    }
                }
            } catch (...) {
                // leave value empty
            }
            json << "\"}";
        }
        json << "]";

        std::string json_str = json.str();
        *out_len = json_str.length();
        char* result = new char[*out_len + 1];
        memcpy(result, json_str.c_str(), *out_len + 1);
        return result;
    } catch (const std::exception& e) {
        std::cerr << "[IFC] Exception in ifc_entity_attributes: " << e.what() << std::endl;
        *out_len = strlen(empty);
        char* result = new char[*out_len + 1];
        memcpy(result, empty, *out_len + 1);
        return result;
    }
#else
    (void)handle;
    (void)id;
    *out_len = strlen(empty);
    char* result = new char[*out_len + 1];
    memcpy(result, empty, *out_len + 1);
    return result;
#endif
}

const char* ifc_entity_global_id(IfcModelHandle handle, IfcEntityId id) {
    static thread_local std::string gid;
#ifdef IFC_OPENSHELL_AVAILABLE
    if (!handle) {
        gid = "";
        return gid.c_str();
    }

    IfcModel* model = static_cast<IfcModel*>(handle);
    try {
        auto entity = model->file->instance_by_id(id);
        if (entity) {
            auto* base_entity = entity->as<IfcUtil::IfcBaseEntity>();
            if (base_entity) {
                gid = base_entity->get_value<std::string>("GlobalId", "");
                return gid.c_str();
            }
        }
    } catch (...) {}
#else
    (void)handle;
    (void)id;
#endif

    gid = "";
    return gid.c_str();
}

namespace {
// Escape a string for embedding inside a JSON string literal.
void append_json_escaped(std::ostringstream& json, const std::string& s) {
    json << "\"";
    for (char c : s) {
        if (c == '"') json << "\\\"";
        else if (c == '\\') json << "\\\\";
        else if (c == '\n') json << "\\n";
        else if (c == '\r') json << "\\r";
        else if (c == '\t') json << "\\t";
        else json << c;
    }
    json << "\"";
}
} // namespace

char* ifc_entity_property_sets(IfcModelHandle handle, IfcEntityId id, int* out_len) {
    const char* empty = "[]";
    auto write_json = [&](const std::string& json_str) {
        *out_len = json_str.length();
        char* result = new char[*out_len + 1];
        memcpy(result, json_str.c_str(), *out_len + 1);
        return result;
    };

#ifdef IFC_OPENSHELL_AVAILABLE
    if (!handle) return write_json(empty);

    IfcModel* model = static_cast<IfcModel*>(handle);
    if (!model->initialized || !model->file) return write_json(empty);

    try {
        auto target = model->file->instance_by_id(static_cast<int>(id));
        if (!target) return write_json(empty);

        auto rels = model->file->instances_by_type("IfcRelDefinesByProperties");
        if (!rels || rels->size() == 0) return write_json(empty);

        std::ostringstream json;
        json << "[";
        bool first = true;

        for (auto rel_it = rels->begin(); rel_it != rels->end(); ++rel_it) {
            auto* rel_base = *rel_it;
            if (!rel_base) continue;
            auto* rel = rel_base->as<IfcUtil::IfcBaseEntity>();
            if (!rel) continue;

            // The element can be related via RelatedObjects (IFC4+)
            // or RelatingBuildingElement (IFC2x3).
            bool matches = false;
            try {
                auto objs = rel->get_value<boost::shared_ptr<aggregate_of_instance>>("RelatedObjects");
                for (auto obj_it = objs->begin(); obj_it != objs->end(); ++obj_it) {
                    if ((*obj_it) && (*obj_it)->id() == target->id()) {
                        matches = true;
                        break;
                    }
                }
            } catch (...) {}
            if (!matches) {
                try {
                    auto* building_element = rel->get_value<IfcUtil::IfcBaseClass*>("RelatingBuildingElement");
                    if (building_element && building_element->id() == target->id()) {
                        matches = true;
                    }
                } catch (...) {}
            }
            if (!matches) continue;

            auto* def_base = rel->get_value<IfcUtil::IfcBaseClass*>("RelatingPropertyDefinition");
            if (!def_base) continue;
            auto* def_entity = def_base->as<IfcUtil::IfcBaseEntity>();
            if (!def_entity) continue;

            const std::string def_type = def_entity->declaration().name();
            if (def_type != "IfcPropertySet" && def_type != "IfcElementQuantity") continue;

            std::string def_name;
            try {
                def_name = def_entity->get_value<std::string>("Name", "");
            } catch (...) {}

            std::vector<std::pair<std::string, std::string>> props;

            if (def_type == "IfcPropertySet") {
                auto props_agg = def_entity->get_value<boost::shared_ptr<aggregate_of_instance>>("HasProperties");
                if (props_agg) {
                    for (auto prop_it = props_agg->begin(); prop_it != props_agg->end(); ++prop_it) {
                        auto* prop_base = *prop_it;
                        if (!prop_base) continue;
                        auto* prop = prop_base->as<IfcUtil::IfcBaseEntity>();
                        if (!prop) continue;

                        std::string p_name;
                        std::string p_value;
                        try {
                            p_name = prop->get_value<std::string>("Name", "");
                        } catch (...) {}

                        const std::string p_type = prop->declaration().name();
                        if (p_type.find("IfcPropertySingle") != std::string::npos) {
                            // NominalValue is an IfcValue select (boolean, label, real, ...),
                            // typically an entity reference to a measure type.
                            try {
                                p_value = property_value_to_display(prop->get("NominalValue"));
                            } catch (...) {}
                        } else if (p_type == "IfcPropertyEnumeratedValue") {
                            // Join the enumerated value names, e.g. "A/B/C".
                            try {
                                auto enum_values = prop->get_value<boost::shared_ptr<aggregate_of_instance>>("EnumeratedValues");
                                bool first_enum = true;
                                for (auto enum_it = enum_values->begin(); enum_it != enum_values->end(); ++enum_it) {
                                    auto* enum_base = *enum_it;
                                    if (!enum_base) continue;
                                    auto* enum_value = enum_base->as<IfcUtil::IfcBaseEntity>();
                                    if (!enum_value) continue;
                                    if (!first_enum) p_value += "/";
                                    first_enum = false;
                                    try {
                                        p_value += enum_value->get_value<std::string>("Name", "");
                                    } catch (...) {}
                                }
                            } catch (...) {}
                        } else if (p_type == "IfcPropertyTable") {
                            try {
                                p_value = property_value_to_display(prop->get("Rows"));
                                if (p_value.empty()) p_value = "(table)";
                            } catch (...) {
                                p_value = "(table)";
                            }
                        }
                        props.emplace_back(p_name, p_value);
                    }
                }
            } else { // IfcElementQuantity
                auto quantities = def_entity->get_value<boost::shared_ptr<aggregate_of_instance>>("Quantities");
                if (quantities) {
                    for (auto q_it = quantities->begin(); q_it != quantities->end(); ++q_it) {
                        auto* q_base = *q_it;
                        if (!q_base) continue;
                        auto* q_entity = q_base->as<IfcUtil::IfcBaseEntity>();
                        if (!q_entity) continue;

                        std::string q_name;
                        std::string q_value;
                        try {
                            q_name = q_entity->get_value<std::string>("Name", "");
                        } catch (...) {}

                        // The quantity entity stores its measure under a
                        // type-specific attribute (Length, Area, NominalValue...);
                        // resolve it to its scalar value.
                        quantity_to_display(q_entity, q_value);
                        props.emplace_back(q_name, q_value);
                    }
                }
            }

            if (!first) json << ",";
            first = false;

            json << "{";
            json << "\"name\": ";
            append_json_escaped(json, def_name);
            json << ", \"definition_type\": ";
            append_json_escaped(json, def_type);
            json << ", \"properties\": [";
            for (size_t i = 0; i < props.size(); i++) {
                if (i > 0) json << ",";
                json << "{\"name\": ";
                append_json_escaped(json, props[i].first);
                json << ", \"value\": ";
                append_json_escaped(json, props[i].second);
                json << "}";
            }
            json << "]}";
        }

        json << "]";
        return write_json(json.str());
    } catch (const std::exception& e) {
        std::cerr << "[IFC] Exception in ifc_entity_property_sets: " << e.what() << std::endl;
        return write_json(empty);
    }
#else
    (void)handle;
    (void)id;
    return write_json(empty);
#endif
}

char* ifc_geometry_extract(IfcModelHandle handle, IfcEntityId entity_id, int* out_len) {
    const char* placeholder = "{\"vertices\":[],\"indices\":[],\"entity_id\":0}";
    *out_len = strlen(placeholder);
    char* result = new char[*out_len + 1];
    memcpy(result, placeholder, *out_len + 1);
    (void)handle;
    (void)entity_id;
    return result;
}

void ifc_geometry_free(char* json) {
    delete[] json;
}

char* ifc_extract_all_geometry(IfcModelHandle handle, int* out_len) {
#ifdef IFC_OPENSHELL_AVAILABLE
    if (!handle) {
        const char* empty = "[]";
        *out_len = strlen(empty);
        char* result = new char[*out_len + 1];
        memcpy(result, empty, *out_len + 1);
        return result;
    }

    IfcModel* model = static_cast<IfcModel*>(handle);
    if (!model->initialized || !model->file) {
        const char* empty = "[]";
        *out_len = strlen(empty);
        char* result = new char[*out_len + 1];
        memcpy(result, empty, *out_len + 1);
        return result;
    }

    try {
        ifcopenshell::geometry::Settings settings;
        settings.get<ifcopenshell::geometry::settings::MesherLinearDeflection>().value = 0.001;
        settings.get<ifcopenshell::geometry::settings::MesherAngularDeflection>().value = 30.0;
        // The OCCT mesher only emits normals when vertices are NOT welded
        // (OpenCascadeConversionResult.cpp: calculate_normals is gated on
        // WeldVertices). Welded vertices share normals between faces, so no
        // normals are emitted here and the renderer averages them instead -
        // which soft-shades flat faceted geometry and makes elements look
        // inflated. Rendering wants per-corner normals, so keep them unwelded.
        settings.get<ifcopenshell::geometry::settings::WeldVertices>().value = false;

        auto kernel = std::make_unique<IfcGeom::OpenCascadeKernel>(settings);
        std::vector<IfcGeom::filter_t> filters;

        IfcGeom::Iterator iterator(std::move(kernel), settings, model->file.get(), filters, 1);

        if (!iterator.initialize()) {
            std::cerr << "[IFC] Iterator initialization failed" << std::endl;
            const char* empty = "[]";
            *out_len = strlen(empty);
            char* result = new char[*out_len + 1];
            memcpy(result, empty, *out_len + 1);
            return result;
        }

        std::ostringstream json;
        json << "[";
        bool first = true;

        do {
            IfcGeom::Element* element = iterator.get();
            if (!element) continue;

            auto* tri_element = dynamic_cast<IfcGeom::TriangulationElement*>(element);
            if (!tri_element) continue;

            const auto& geom = tri_element->geometry();
            const auto& verts = geom.verts();
            const auto& faces = geom.faces();
            const auto& normals = geom.normals();

            if (verts.empty() || faces.empty()) continue;

            const auto& matrix = element->transformation().data();

            if (!first) json << ",";
            first = false;

            json << "{";
            json << "\"entity_id\":" << element->id() << ",";
            json << "\"entity_type\":\"" << element->type() << "\",";
            json << "\"entity_name\":\"" << element->name() << "\",";

            json << "\"matrix\":[";
            if (matrix) {
                const auto& m = matrix->ccomponents();
                const double* m_data = m.data();
                for (int i = 0; i < 16; i++) {
                    if (i > 0) json << ",";
                    json << m_data[i];
                }
            } else {
                json << "1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1";
            }
            json << "],";

            json << "\"vertices\":[";
            for (size_t i = 0; i < verts.size(); i++) {
                if (i > 0) json << ",";
                json << static_cast<float>(verts[i]);
            }
            json << "],";

            json << "\"indices\":[";
            for (size_t i = 0; i < faces.size(); i++) {
                if (i > 0) json << ",";
                json << faces[i];
            }
            json << "],";

            json << "\"normals\":[";
            for (size_t i = 0; i < normals.size(); i++) {
                if (i > 0) json << ",";
                json << static_cast<float>(normals[i]);
            }
            json << "]";

            json << "}";

        } while (iterator.next());

        json << "]";

        std::string json_str = json.str();
        *out_len = json_str.length();
        char* result = new char[*out_len + 1];
        memcpy(result, json_str.c_str(), *out_len + 1);
        return result;

    } catch (const std::exception& e) {
        std::cerr << "[IFC] Exception extracting geometry: " << e.what() << std::endl;
        const char* empty = "[]";
        *out_len = strlen(empty);
        char* result = new char[*out_len + 1];
        memcpy(result, empty, *out_len + 1);
        return result;
    }
#else
    (void)handle;
    const char* empty = "[]";
    *out_len = strlen(empty);
    char* result = new char[*out_len + 1];
    memcpy(result, empty, *out_len + 1);
    return result;
#endif
}

} // extern "C"
