/**
 * Abstraction over Tauri's invoke() API.
 * Provides type-safe calls to Rust backend commands.
 */

// IPC response types matching the Rust side
export interface ModelInfo {
  path: string;
  entity_count: number;
  project_name?: string;
}

export interface EntitySnapshot {
  id: number;
  global_id: string;
  ifc_type: string;
  name?: string;
  description?: string;
  attributes: [string, string][];
}

export interface TreeNodeData {
  id: number;
  name: string;
  ifc_type: string;
  children: TreeNodeData[];
}

export interface MeshData {
  entity_id: number;
  vertices: number[];
  indices: number[];
  normals: number[];
  material: { name: string; color_diffuse: [number, number, number]; opacity: number; roughness: number; metalness: number };
  bbox: { min: [number, number, number]; max: [number, number, number] };
}

/**
 * Check if running inside Tauri (has __TAURI__ global)
 */
function isTauri(): boolean {
  return typeof window !== 'undefined' && 
         (window as any).__TAURI__ !== undefined ||
         import.meta.env.TAURI === 'true';
}

let invokeCounter = 0;

/**
 * Generic invoke wrapper with error handling and logging
 */
async function tauriInvoke<T>(command: string, args?: unknown): Promise<T> {
  if (isTauri()) {
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<T>(command, args ?? {});
  }
  
  // Mock implementation for development outside Tauri
  console.log(`[IPC] ${command}`, args ?? '');
  return mockInvoke(command, args as Record<string, unknown>);
}

async function mockInvoke(command: string, args?: Record<string, unknown>): Promise<unknown> {
  // Simulate network delay for realistic testing outside Tauri
  await new Promise(resolve => setTimeout(resolve, 100));
  
  switch (command) {
    case 'open_ifc_file':
      return { path: '/mock/example.ifc', entity_count: 523, project_name: 'Mock Building' };
    case 'get_hierarchy_root':
      return null; // Would return TreeNodeData from real backend
    default:
      return null;
  }
}

export function useTauriIPC() {
  return {
    openFile: () => tauriInvoke<ModelInfo | null>('open_file').then(r => r?.path ? tauriInvoke<ModelInfo>('open_ifc_file', { file_path: r.path }) : null),
    closeModel: () => tauriInvoke<void>('close_model'),
    getEntityCount: () => tauriInvoke<number>('get_entity_count'),
    getEntityById: (id: number) => tauriInvoke<EntitySnapshot>('get_entity_by_id', { entity_id: id }),
    getHierarchyRoot: () => tauriInvoke<TreeNodeData | null>('get_hierarchy_root'),
    getChildren: (parentId: number) => tauriInvoke<TreeNodeData[]>('get_children', { entity_id: parentId }),
    searchEntities: (query: string) => tauriInvoke<number[]>('search_entities', { query }),
    getEntityTypes: () => tauriInvoke<Array<[string, number]>>('get_entity_types'),
    loadGeometryBatch: (ids: number[]) => tauriInvoke<MeshData[]>('load_geometry_batch', { entity_ids: ids }),
    releaseGeometry: (ids: number[]) => tauriInvoke<number>('release_geometry', { entity_ids: ids }),
  };
}
