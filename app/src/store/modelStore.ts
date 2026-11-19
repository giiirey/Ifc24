import { create } from 'zustand';

// Maximum transparency applied to dimmed (non-selected) elements.
// Each element keeps its own opacity, clamped to at most this value.
export const DEFAULT_DIM_OPACITY = 0.35;

export interface EntityInfo {
  id: number;
  global_id: string;
  ifc_type: string | { name?: string };
  name?: string;
  description?: string;
}

export interface MeshItem {
  entity_id: number;
  vertices: number[];
  indices: number[];
  normals?: number[];
  material: { name: string; color_diffuse: [number, number, number]; opacity?: number; roughness?: number; metalness?: number };
  bbox: { min: [number, number, number]; max: [number, number, number] } | null;
}

interface ModelStore {
  // Model data
  entities: Map<number, EntityInfo>;
  treeNodes: TreeNodeData[];
  meshes: Map<number, MeshItem[]>;
  loadedFilePath: string | null;
  remainingIds: number[];
  
  // UI state
  selectedEntityId: number | null;
  expandedIds: Set<number>;

  // Viewport state
  dimOpacity: number;    // ceiling for dimmed elements; each element keeps its own opacity below it
  viewRevision: number;  // bumped by "Update view" so the viewport rebuilds its render data
  
  // Loading state
  isLoading: boolean;
  loadProgress: number;
  batchLoadedCount: number;
  totalBatchCount: number;
  
  // Actions
  setEntities: (entities: Map<number, EntityInfo>) => void;
  selectEntity: (id: number | null) => void;
  toggleExpand: (id: number) => void;
  setMeshes: (entityId: number | Map<number, MeshItem[]> | MeshItem[], meshes?: MeshItem[]) => void;
  setLoading: (loading: boolean, progress?: number) => void;
  setHierarchy: (nodes: TreeNodeData[]) => void;
  setLoadedFilePath: (path: string | null) => void;
  setRemainingIds: (ids: number[]) => void;
  updateBatchProgress: (loaded: number, total: number) => void;

  // Viewport actions
  updateView: () => void;
  
  // Clear
  clearModel: () => void;
}

interface TreeNodeData {
  id: number;
  name: string;
  ifc_type: string;
  children: TreeNodeData[];
}

export const useModelStore = create<ModelStore>((set, get) => ({
  entities: new Map(),
  treeNodes: [],
  meshes: new Map(),
  loadedFilePath: null,
  remainingIds: [],
  selectedEntityId: null,
  expandedIds: new Set(),
  dimOpacity: DEFAULT_DIM_OPACITY,
  viewRevision: 0,
  isLoading: false,
  loadProgress: 0,
  batchLoadedCount: 0,
  totalBatchCount: 0,

  setEntities: (entities) => set({ entities }),
  
  selectEntity: (id) => {
    set({ selectedEntityId: id });
    if (id !== null && get().treeNodes.length > 0) {
      const expanded = new Set(get().expandedIds);
      expandToNode(get().treeNodes, id, expanded);
      set({ expandedIds: expanded });
    }
  },

  toggleExpand: (id) => {
    const current = get().expandedIds;
    const next = new Set(current);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    set({ expandedIds: next });
  },

  setMeshes: (entityId: number | Map<number, MeshItem[]> | MeshItem[], meshes?: MeshItem[]) => {
    if (Array.isArray(entityId)) {
      const meshMap = new Map<number, MeshItem[]>();
      for (const mesh of entityId) {
        const id = Number(mesh.entity_id);
        if (!meshMap.has(id)) {
          meshMap.set(id, []);
        }
        meshMap.get(id)!.push(mesh);
      }
      set({ meshes: meshMap });
    } else if (entityId instanceof Map) {
      set({ meshes: entityId });
    } else {
      const current = new Map(get().meshes);
      current.set(entityId, meshes!);
      set({ meshes: current });
    }
  },

  setLoading: (loading, progress = 0) => set({ isLoading: loading, loadProgress: progress }),
  
  setHierarchy: (nodes) => set({ treeNodes: nodes }),
  
  setLoadedFilePath: (path) => set({ loadedFilePath: path }),
  
  setRemainingIds: (ids) => set({ remainingIds: ids, batchLoadedCount: 0, totalBatchCount: Math.ceil(ids.length / 200) }),
  
  updateBatchProgress: (loaded, total) => set({ batchLoadedCount: loaded, totalBatchCount: total }),

  // "Update view": clear the selection and put every element back to its own
  // opacity, dimmed only up to the default ceiling.
  updateView: () => set({
    selectedEntityId: null,
    dimOpacity: DEFAULT_DIM_OPACITY,
    viewRevision: get().viewRevision + 1,
  }),
  
  clearModel: () => set({ 
    entities: new Map(), 
    treeNodes: [], 
    meshes: new Map(), 
    selectedEntityId: null, 
    expandedIds: new Set(),
    dimOpacity: DEFAULT_DIM_OPACITY,
    viewRevision: get().viewRevision + 1,
    isLoading: false,
    loadProgress: 0,
    loadedFilePath: null,
    remainingIds: [],
    batchLoadedCount: 0,
    totalBatchCount: 0,
  }),
}));

function expandToNode(nodes: TreeNodeData[], targetId: number, expanded: Set<number>): boolean {
  for (const node of nodes) {
    if (node.id === targetId) {
      return true;
    }
    if (expandToNode(node.children, targetId, expanded)) {
      expanded.add(node.id);
      return true;
    }
  }
  return false;
}
