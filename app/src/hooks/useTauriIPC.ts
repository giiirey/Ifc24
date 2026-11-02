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
  ifc_type: string | { name?: string };
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

/** Property set or quantity attached via IfcRelDefinesByProperties. */
export interface PropertySetData {
  name: string;
  definition_type: string;
  properties: { name: string; value: string }[];
}

/**
 * Check if running inside Tauri (has __TAURI__ global)
 */
let _cachedTauri: boolean | null = null;

function syncIsTauri(): boolean {
  if (_cachedTauri !== null) return _cachedTauri;
  
  // Check if we're in Tauri by checking the global context
  const tauiGlobal = (window as any).__TAURI__;
  console.log('[IPC] __TAURI__:', typeof tauiGlobal, !!tauiGlobal);
  
  if (tauiGlobal) {
    _cachedTauri = true;
    return true;
  }

  // Check for TAURI env variable from cargo tauri dev
  const tauriEnv = import.meta.env?.TAURI === 'true';
  console.log('[IPC] TAURI env:', tauriEnv);
  
  if (tauriEnv) {
    _cachedTauri = true;
    return true;
  }

  // Try dynamic import as last resort
  import('@tauri-apps/api/core')
    .then(() => {
      console.log('[IPC] @tauri-apps/api/core loaded successfully');
      _cachedTauri = true;
    })
    .catch(err => {
      console.log('[IPC] @tauri-apps/api/core failed:', err?.message);
      _cachedTauri = false;
    });

  // Default to Tauri mode if in a browser environment (Tauri runs in webview)
  _cachedTauri = typeof window !== 'undefined';
  console.log('[IPC] Defaulting Tauri to:', _cachedTauri, '(window exists)');
  
  return !!_cachedTauri;
}

/**
 * Generic invoke wrapper with error handling and logging
 */
async function tauriInvoke<T>(command: string, _args?: unknown): Promise<T> {
  const usingTauri = syncIsTauri();
  
  if (usingTauri) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const argsObj = typeof _args === 'object' && _args !== null ? _args as Record<string, unknown> : undefined;
      return await invoke<T>(command, argsObj);
    } catch (err) {
      // Tauri call failed — fall through to mock
      console.warn(`[IPC] Tauri invoke failed for "${command}":`, err);
      return mockInvoke(command, _args) as Promise<T>;
    }
  }
  
  // Mock implementation for development outside Tauri
  return mockInvoke(command, _args) as Promise<T>;
}

async function mockInvoke(_command: string, _args?: unknown): Promise<unknown> {
  await new Promise(resolve => setTimeout(resolve, 150));

  const mockEntities: EntitySnapshot[] = [
    { id: 1, global_id: '3jT0DjvX7GqL5nM0rZ8w', ifc_type: 'IfcProject', name: 'Mock Building Project', description: 'IFC test model', attributes: [['GlobalId', '3jT0DjvX7GqL5nM0rZ8w'], ['Name', 'Mock Building Project'], ['ObjectType', 'Building']] },
    { id: 2, global_id: '1mK9pL2xR4vN6wQ8yB0c', ifc_type: 'IfcSite', name: 'Default Site', attributes: [['GlobalId', '1mK9pL2xR4vN6wQ8yB0c'], ['Name', 'Default Site']] },
    { id: 3, global_id: '2aB3cD4eF5gH6iJ7kL8m', ifc_type: 'IfcBuilding', name: 'Mock Building', attributes: [['GlobalId', '2aB3cD4eF5gH6iJ7kL8m'], ['Name', 'Mock Building']] },
    { id: 4, global_id: '4nP9qR8sT7uV6wX5yZ4a', ifc_type: 'IfcBuildingStorey', name: 'Ground Floor', attributes: [['GlobalId', '4nP9qR8sT7uV6wX5yZ4a'], ['Name', 'Ground Floor']] },
    { id: 5, global_id: '5bC6dE7fG8hI9jK0lM1n', ifc_type: 'IfcWall', name: 'Wall - Outer', description: 'Outer wall', attributes: [['GlobalId', '5bC6dE7fG8hI9jK0lM1n'], ['Name', 'Wall - Outer'], ['Tag', 'W-001']] },
    { id: 6, global_id: '6oP7qR8sT9uV0wX1yZ2a', ifc_type: 'IfcSlab', name: 'Slab - Ground', description: 'Ground slab', attributes: [['GlobalId', '6oP7qR8sT9uV0wX1yZ2a'], ['Name', 'Slab - Ground'], ['Tag', 'SLAB-001']] },
    { id: 7, global_id: '7bC8dE9fG0hI1jK2lM3n', ifc_type: 'IfcColumn', name: 'Column - C1', attributes: [['GlobalId', '7bC8dE9fG0hI1jK2lM3n'], ['Name', 'Column - C1'], ['Tag', 'COL-001']] },
    { id: 8, global_id: '8oP9qR0sT1uV2wX3yZ4a', ifc_type: 'IfcDoor', name: 'Door - D001', attributes: [['GlobalId', '8oP9qR0sT1uV2wX3yZ4a'], ['Name', 'Door - D001'], ['Tag', 'DOOR-001']] },
    { id: 9, global_id: '9bC0dE1fG2hI3jK4lM5n', ifc_type: 'IfcWindow', name: 'Window - W001', attributes: [['GlobalId', '9bC0dE1fG2hI3jK4lM5n'], ['Name', 'Window - W001'], ['Tag', 'WIN-001']] },
    { id: 10, global_id: '10oP1qR2sT3uV4wX5yZ6a', ifc_type: 'IfcBeam', name: 'Beam - B001', attributes: [['GlobalId', '10oP1qR2sT3uV4wX5yZ6a'], ['Name', 'Beam - B001'], ['Tag', 'BEAM-001']] },
  ];

  const args = _args as Record<string, unknown> | undefined;

  switch (_command) {
    case 'get_all_entities':
      console.log('[MOCK] get_all_entities — returning', mockEntities.length, 'entities');
      return mockEntities;
    case 'get_entity_by_id':
      if (args && typeof args.entity_id === 'number') {
        return mockEntities.find(e => e.id === args.entity_id) || null;
      }
      return null;
    case 'open_file':
      return { path: '/mock/example.ifc', entity_count: mockEntities.length, project_name: 'Mock Building' };
    case 'open_ifc_file':
      return { path: '/mock/example.ifc', entity_count: mockEntities.length, project_name: 'Mock Building' };
    case 'open_ifc_file_from_dialog':
      console.log('[MOCK] open_ifc_file_from_dialog — returning mock data');
      return { path: '/mock/example.ifc', entity_count: mockEntities.length, project_name: 'Mock Building' };

    case 'open_ifc_file_from_dialog_v2':
      console.log('[MOCK] open_ifc_file_from_dialog_v2 returning mock data');
      return { path: '/mock/example2.ifc', entity_count: 5, project_name: 'Test Project' };
    case 'get_hierarchy_root':
      return { id: 1, name: 'Mock Building Project', ifc_type: 'IfcProject', children: [
        { id: 2, name: 'Default Site', ifc_type: 'IfcSite', children: [
          { id: 3, name: 'Mock Building', ifc_type: 'IfcBuilding', children: [
            { id: 4, name: 'Ground Floor', ifc_type: 'IfcBuildingStorey', children: [
              { id: 5, name: 'Wall - Outer', ifc_type: 'IfcWall', children: [] },
              { id: 6, name: 'Slab - Ground', ifc_type: 'IfcSlab', children: [] },
              { id: 7, name: 'Column - C1', ifc_type: 'IfcColumn', children: [] },
              { id: 8, name: 'Door - D001', ifc_type: 'IfcDoor', children: [] },
              { id: 9, name: 'Window - W001', ifc_type: 'IfcWindow', children: [] },
              { id: 10, name: 'Beam - B001', ifc_type: 'IfcBeam', children: [] },
            ]},
          ]},
        ]},
      ], expanded: false, has_geometry: true };
    case 'get_entity_property_sets': {
      const id = args && typeof args.entity_id === 'number' ? args.entity_id : null;
      const ifcType = mockEntities.find(e => e.id === id);
      const typeStr = ifcType
        ? (typeof ifcType.ifc_type === 'string' ? ifcType.ifc_type : (ifcType.ifc_type as any).name || 'Unknown')
        : 'IfcElement';
      // Wall gets a realistic Pset_WallCommon; other elements get a generic pset + Qto.
      if (typeStr === 'IfcWall') {
        return [
          {
            name: 'Pset_WallCommon',
            definition_type: 'IfcPropertySet',
            properties: [
              { name: 'LoadBearing', value: 'True' },
              { name: 'FireRating', value: '60' },
              { name: 'Compartment', value: 'Yes' },
              { name: 'BasicWallType', value: 'Standard' },
              { name: 'WallType', value: 'Load-bearing' },
              { name: 'WallThickness', value: '0.3' },
              { name: 'Reference', value: 'W-001' },
            ],
          },
          {
            name: 'Wall_Qto',
            definition_type: 'IfcElementQuantity',
            properties: [
              { name: 'Net_Gross_Weight', value: '1875.42' },
              { name: 'Gross_Weight', value: '1875.42' },
              { name: 'Volume', value: '0.73' },
              { name: 'Area', value: '12.5' },
              { name: 'Length', value: '12.5' },
              { name: 'Height', value: '3.0' },
              { name: 'Width', value: '0.3' },
            ],
          },
        ];
      }
      return [
        {
          name: `Pset_${typeStr.replace('Ifc', '')}Common`,
          definition_type: 'IfcPropertySet',
          properties: [
            { name: 'Reference', value: `${typeStr.slice(-4)}-001` },
            { name: 'FireRating', value: '60' },
          ],
        },
        {
          name: `${typeStr.replace('Ifc', '')}_Qto`,
          definition_type: 'IfcElementQuantity',
          properties: [
            { name: 'Volume', value: '1.25' },
            { name: 'Area', value: '2.4' },
          ],
        },
      ];
    }
    case 'search_entities':
      return mockEntities.map(e => e.id);
    case 'get_all_entity_ids':
      return mockEntities.map(e => e.id);
    case 'get_all_entities':
      return mockEntities;
    case 'get_entity_by_id':
      return null;
    case 'get_entity_types': {
      const typeMap = new Map<string, number>();
      mockEntities.forEach(e => {
        const typeStr = typeof e.ifc_type === 'string' ? e.ifc_type : (e.ifc_type as any).name || 'Unknown';
        typeMap.set(typeStr, (typeMap.get(typeStr) || 0) + 1);
      });
      return Array.from(typeMap.entries());
    }
    case 'load_geometry_batch':
      return mockEntities.map(e => {
        const ifcTypeStr = typeof e.ifc_type === 'string' ? e.ifc_type : (e.ifc_type as any).name || 'Unknown';
        return {
          entity_id: e.id,
          vertices: [
            -1, -1, -1, 1, -1, -1, 1, 1, -1, -1, 1, -1,
            -1, -1, 1, 1, -1, 1, 1, 1, 1, -1, 1, 1,
            -1, -1, -1, -1, -1, 1, -1, 1, 1, -1, 1, -1,
            1, -1, -1, 1, -1, 1, 1, 1, 1, 1, 1, -1,
            -1, -1, -1, -1, 1, -1, 1, 1, -1, 1, -1, -1,
            -1, -1, 1, -1, 1, 1, 1, 1, 1, 1, -1, 1,
          ],
          indices: [0,1,2,0,2,3, 4,5,6,4,6,7, 8,9,10,8,10,11, 12,13,14,12,14,15, 16,17,18,16,18,19, 20,21,22,20,22,23],
          normals: [],
          material: { name: `${ifcTypeStr}_material`, color_diffuse: getMockColor(ifcTypeStr), opacity: 0.9, roughness: 0.7, metalness: 0.1 },
          bbox: { min: [-1, -1, -1] as [number, number, number], max: [1, e.id * 3 + 2, 1] as [number, number, number] },
        };
      });
    default:
      return null;
  }
}

function getMockColor(ifcType: string): [number, number, number] {
  const colors: Record<string, [number, number, number]> = {
    'IfcWall': [0.67, 0.67, 0.67],
    'IfcSlab': [0.53, 0.53, 0.53],
    'IfcColumn': [0.4, 0.4, 0.4],
    'IfcBeam': [0.47, 0.47, 0.47],
    'IfcDoor': [0.55, 0.37, 0.24],
    'IfcWindow': [0.42, 0.64, 0.85],
    'IfcProject': [0.3, 0.5, 0.3],
    'IfcSite': [0.35, 0.5, 0.3],
    'IfcBuilding': [0.5, 0.4, 0.3],
    'IfcBuildingStorey': [0.4, 0.4, 0.5],
  };
  return colors[ifcType] || [0.5, 0.5, 0.5];
}

export async function fetchAllEntities(): Promise<EntitySnapshot[]> {
  return await tauriInvoke<EntitySnapshot[]>('get_all_entities');
}

async function invokeWithRetry<T>(command: string, args?: unknown, retries = 2): Promise<T | null> {
  const result = await tauriInvoke<T | null>(command, args);
  if (result !== null) return result;
  
  for (let i = 0; i < retries; i++) {
    await new Promise(r => setTimeout(r, 200));
    const retryResult = await tauriInvoke<T | null>(command, args);
    if (retryResult !== null) {
      console.log(`[IPC] Command "${command}" succeeded on retry ${i + 1}`);
      return retryResult;
    }
  }
  return null;
}

export function useTauriIPC() {
  return {
    openFile: () => invokeWithRetry<ModelInfo>('open_ifc_file_from_dialog'),
    closeModel: () => tauriInvoke<void>('close_model'),
    getEntityCount: () => tauriInvoke<number>('get_entity_count'),
    getEntityById: (id: number) => tauriInvoke<EntitySnapshot>('get_entity_by_id', { entity_id: id }),
    getEntityPropertySets: (id: number) => tauriInvoke<PropertySetData[]>('get_entity_property_sets', { entity_id: id }),
    getHierarchyRoot: () => tauriInvoke<TreeNodeData | null>('get_hierarchy_root'),
    getChildren: (parentId: number) => tauriInvoke<TreeNodeData[]>('get_children', { entity_id: parentId }),
    searchEntities: (query: string) => tauriInvoke<number[]>('search_entities', { query }),
    getAllEntityIds: () => tauriInvoke<number[]>('get_all_entity_ids'),
    getAllEntities: () => tauriInvoke<EntitySnapshot[]>('get_all_entities'),
    getEntityTypes: () => tauriInvoke<Array<[string, number]>>('get_entity_types'),
    loadGeometryBatch: (ids: number[]) => tauriInvoke<MeshData[]>('load_geometry_batch', { entityIds: ids }),
    releaseGeometry: (ids: number[]) => tauriInvoke<number>('release_geometry', { entityIds: ids }),
  };
}
