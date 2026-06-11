import { useEffect, useState } from 'react';
import { useModelStore } from './store/modelStore';
import { useTauriIPC, fetchAllEntities, ModelInfo, MeshData, EntitySnapshot } from './hooks/useTauriIPC';
import Layout from './components/Layout';
import IfcTree from './components/tree/IfcTree';
import Viewer3D from './components/viewer/Viewer3D';
import PropertiesPanel from './components/properties/PropertiesPanel';

function App() {
  const [selectedEntityId, setSelectedEntityId] = useState<number | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [ifcTypeFilter, setIfcTypeFilter] = useState<string>('');
  const [isModelLoaded, setIsModelLoaded] = useState(false);

  const ipc = useTauriIPC();
  const store = useModelStore();

  useEffect(() => {
    if (store.loadedFilePath) {
      setIsModelLoaded(true);
    } else {
      setIsModelLoaded(false);
    }
  }, [store.loadedFilePath]);

  useEffect(() => {
    if (selectedEntityId !== null && store.selectedEntityId !== selectedEntityId) {
      store.selectEntity(selectedEntityId);
    }
  }, [selectedEntityId, store.selectedEntityId, store.selectEntity]);

  const handleLoadFile = async () => {
    try {
      store.setLoading(true);
      setIsModelLoaded(false);

      console.log('[App] Step 1: opening file dialog...');
      const modelInfo: ModelInfo | null = await ipc.openFile();
      console.log('[App] Step 1 result:', modelInfo);
      if (!modelInfo) {
        console.log('[App] No model info, aborting');
        store.setLoading(false);
        return;
      }

      console.log('[App] Step 2: getting hierarchy...');
      const hierarchyRoot = await ipc.getHierarchyRoot();
      console.log('[App] Step 2 result:', JSON.stringify(hierarchyRoot, null, 2));
      
      if (hierarchyRoot) {
        store.setHierarchy([hierarchyRoot]);
        console.log('[App] Set hierarchy, treeNodes count:', [hierarchyRoot].length);
      } else {
        store.setHierarchy([]);
        console.log('[App] No hierarchy root');
      }

      console.log('[App] Step 3: fetching entities...');
      const entities: EntitySnapshot[] = await fetchAllEntities();
      console.log('[App] Step 3 result:', entities.length, 'entities');
      
      // Log first entity for debugging
      if (entities.length > 0) {
        console.log('[App] First entity:', JSON.stringify(entities[0], null, 2));
      }

      const entityMap = new Map<number, EntitySnapshot>();
      entities.forEach(e => entityMap.set(e.id, e));
      store.setEntities(entityMap as Map<number, any>);

      console.log('[App] Step 4: loading geometry in batches...');
      
      const allEntityIds = await ipc.getAllEntityIds();
      console.log('[App] Step 4:', allEntityIds.length, 'entity IDs total');
      
      if (allEntityIds.length > 0) {
        store.setRemainingIds(allEntityIds);
        
        // Load geometry in batches - show progress bar
        const BATCH_SIZE = 200;
        const totalBatches = Math.ceil(allEntityIds.length / BATCH_SIZE);
        const loadingBar = document.createElement('div');
        loadingBar.style.cssText = 'position:fixed;top:50%;left:50%;transform:translate(-50%,-50%);background:#1e2746;padding:24px 48px;border-radius:8px;z-index:9999;color:#e0e0e0;font-size:14px;text-align:center;';
        loadingBar.textContent = `Loading geometry 0/${allEntityIds.length}...`;
        document.body.appendChild(loadingBar);

        let loadedCount = 0;
        for (let i = 0; i < allEntityIds.length; i += BATCH_SIZE) {
          const batchEnd = Math.min(i + BATCH_SIZE, allEntityIds.length);
          const batch = allEntityIds.slice(i, batchEnd);
          
          try {
            const meshes: MeshData[] = await ipc.loadGeometryBatch(batch.map(Number));
            console.log(`[App] Batch ${Math.floor(i / BATCH_SIZE) + 1}/${totalBatches}: ${meshes.length} meshes`);
            if (meshes.length > 0) {
              console.log('[App] Sample mesh:', JSON.stringify(meshes[0], null, 2));
            }
            meshes.forEach(mesh => {
              store.setMeshes(Number(mesh.entity_id), [mesh as any]);
            });
          } catch (err) {
            console.warn(`[App] Failed to load geometry batch:`, err);
          }
          
          loadedCount += (batchEnd - i);
          loadingBar.textContent = `Loading geometry ${loadedCount}/${allEntityIds.length}...`;
        }

        document.body.removeChild(loadingBar);
      }

      console.log('[App] Step 5: finalizing');
      store.setLoadedFilePath(modelInfo.path);
      setIsModelLoaded(true);
      console.log('[App] DONE - entities:', store.entities.size, 'meshes:', store.meshes.size, 'nodes:', store.treeNodes.length);
    } catch (error) {
      console.error('[App] Failed to load file:', error);
      store.setLoading(false);
    }
  };

  return (
    <Layout>
      <div className="app-sidebar">
        <div className="sidebar-header">
          <h1 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--color-text-primary)' }}>
            IFC Tauri
          </h1>
        </div>

        <div style={{ padding: '8px' }}>
          <button
            onClick={handleLoadFile}
            className="toolbar-button"
            style={{ width: '100%', textAlign: 'center', cursor: 'pointer' }}
          >
            Open IFC File
          </button>
        </div>

        <div className="sidebar-controls">
          <input
            type="text"
            placeholder="Search entities..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="search-input"
            disabled={!isModelLoaded}
          />
          <select
            value={ifcTypeFilter}
            onChange={(e) => setIfcTypeFilter(e.target.value)}
            className="filter-select"
            disabled={!isModelLoaded}
          >
            <option value="">All types</option>
            <option value="IfcWall">IfcWall</option>
            <option value="IfcSlab">IfcSlab</option>
            <option value="IfcColumn">IfcColumn</option>
            <option value="IfcBeam">IfcBeam</option>
            <option value="IfcDoor">IfcDoor</option>
            <option value="IfcWindow">IfcWindow</option>
            <option value="IfcSpace">IfcSpace</option>
          </select>
        </div>

        {isModelLoaded ? (
          <IfcTree
            treeData={store.treeNodes}
            selectedEntityId={selectedEntityId}
            onSelect={(id) => setSelectedEntityId(id)}
            searchQuery={searchQuery}
            ifcTypeFilter={ifcTypeFilter}
            expandedIds={store.expandedIds}
            toggleExpand={(id) => store.toggleExpand(id)}
          />
        ) : (
          <div style={{ padding: '12px', color: 'var(--color-text-muted)', textAlign: 'center', fontSize: '13px' }}>
            {store.isLoading ? 'Loading...' : 'Open an IFC file to get started'}
          </div>
        )}

        <div className="sidebar-status">
          {isModelLoaded && store.loadedFilePath ? (
            <span className="status-ready" style={{ fontSize: '10px', color: '#6b8e6b' }}>
              {store.entities.size} entities loaded
            </span>
          ) : store.isLoading ? (
            <span className="status-loading">Loading...</span>
          ) : (
            <span className="status-idle">Ready</span>
          )}
        </div>
      </div>

      <Viewer3D selectedEntityId={selectedEntityId} onOpenFile={handleLoadFile} />

      <PropertiesPanel
        selectedEntityId={selectedEntityId}
        onSelectChange={setSelectedEntityId}
        entities={store.entities as Map<number, EntitySnapshot>}
        meshes={store.meshes}
      />
    </Layout>
  );
}

export default App;
