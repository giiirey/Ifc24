import { create } from 'zustand';

export interface EntityInfo {
  id: number;
  global_id: string;
  ifc_type: string;
  name?: string;
  description?: string;
}

export interface MeshItem {
  entity_id: number;
  vertices: number[];
  indices: number[];
  material: { name: string; color_diffuse: [number, number, number] };
}

interface ModelStore {
  // Model data
  entities: Map<number, EntityInfo>;
  treeNodes: TreeNodeData[];
  meshes: Map<number, MeshItem[]>;
  
  // UI state
  selectedEntityId: number | null;
  expandedIds: Set<number>;
  
  // Loading state
  isLoading: boolean;
  loadProgress: number;
  
  // Actions
  setEntities: (entities: Map<number, EntityInfo>) => void;
  selectEntity: (id: number | null) => void;
  toggleExpand: (id: number) => void;
  setMeshes: (entityId: number, meshes: MeshItem[]) => void;
  setLoading: (loading: boolean, progress?: number) => void;
  
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
  selectedEntityId: null,
  expandedIds: new Set(),
  isLoading: false,
  loadProgress: 0,

  setEntities: (entities) => set({ entities }),
  
  selectEntity: (id) => {
    set({ selectedEntityId: id });
    if (id !== null && get().treeNodes.length > 0) {
      // Expand tree to the selected node
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

  setMeshes: (entityId, meshes) => {
    const current = new Map(get().meshes);
    current.set(entityId, meshes);
    set({ meshes: current });
  },

  setLoading: (loading, progress = 0) => set({ isLoading: loading, loadProgress: progress }),
  
  clearModel: () => set({ 
    entities: new Map(), 
    treeNodes: [], 
    meshes: new Map(), 
    selectedEntityId: null, 
    expandedIds: new Set(),
    isLoading: false,
    loadProgress: 0,
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
