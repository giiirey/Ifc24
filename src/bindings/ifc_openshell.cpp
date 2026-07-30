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
    if (!handle) {
        name = "";
        return name.c_str();
    }

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
        settings.get<ifcopenshell::geometry::settings::WeldVertices>().value = true;

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
