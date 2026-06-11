#include "ifc_openshell.h"
#include <cstdint>
#include <cstring>
#include <string>


struct IfcModel {
    // IfcParse::Model* model; // Placeholder - will wire up in phase 2
    bool initialized = false;
};

extern "C" {

IfcModelHandle ifc_open(const char* path) {
    IfcModel* model = new IfcModel();
    model->initialized = true;
    return static_cast<void*>(model);
}

void ifc_close(IfcModelHandle handle) {
    IfcModel* model = static_cast<IfcModel*>(handle);
    delete model;
}

int32_t ifc_entity_count(IfcModelHandle handle) {
    return 0; // Placeholder
}

int ifc_get_entities_chunk(IfcModelHandle handle, int start, int count, void* output_buffer, int buffer_size) {
    return 0; // Placeholder
}

const char* ifc_entity_type(IfcModelHandle handle, IfcEntityId id) {
    static thread_local std::string result;
    (void)handle;
    (void)id;
    result = "Unknown";
    return result.c_str();
}

const char* ifc_entity_name(IfcModelHandle handle, IfcEntityId id) {
    static thread_local std::string name;
    (void)handle;
    (void)id;
    name = "";
    return name.c_str();
}

const char* ifc_entity_global_id(IfcModelHandle handle, IfcEntityId id) {
    static thread_local std::string gid;
    (void)handle;
    (void)id;
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

} // extern "C"
