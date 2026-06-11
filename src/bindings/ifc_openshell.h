#ifndef IFC_OPENShell_H
#define IFC_OPENShell_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void* IfcModelHandle;
typedef uint64_t IfcEntityId;

// Model lifecycle
IfcModelHandle ifc_open(const char* path);
void ifc_close(IfcModelHandle model);
int32_t ifc_entity_count(IfcModelHandle model);

// Entity iteration (chunked)
int ifc_get_entities_chunk(IfcModelHandle model, int start, int count, void* output_buffer, int buffer_size);

// Entity property access
const char* ifc_entity_type(IfcModelHandle model, IfcEntityId id);
const char* ifc_entity_name(IfcModelHandle model, IfcEntityId id);
const char* ifc_entity_global_id(IfcModelHandle model, IfcEntityId id);

// Geometry extraction
char* ifc_geometry_extract(IfcModelHandle model, IfcEntityId entity_id, int* out_len);
void ifc_geometry_free(char* json);

#ifdef __cplusplus
}
#endif

#endif // IFC_OPENShell_H
