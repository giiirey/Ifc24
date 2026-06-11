import { useState } from 'react';
import Layout from './components/Layout';
import IfcTree from './components/tree/IfcTree';
import Viewer3D from './components/viewer/Viewer3D';
import PropertiesPanel from './components/properties/PropertiesPanel';

function App() {
  const [selectedEntityId, setSelectedEntityId] = useState<number | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [ifcTypeFilter, setIfcTypeFilter] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);

  const handleSelectEntity = (id: number | null) => {
    setSelectedEntityId(id);
  };

  return (
    <Layout>
      <div className="app-sidebar">
        <div className="sidebar-header">
          <h1 style={{ fontSize: '14px', fontWeight: 600, color: 'var(--color-text-primary)' }}>
            IFC Tauri
          </h1>
        </div>
        
        <div className="sidebar-controls">
          <input
            type="text"
            placeholder="Search entities..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="search-input"
          />
          <select
            value={ifcTypeFilter}
            onChange={(e) => setIfcTypeFilter(e.target.value)}
            className="filter-select"
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

        <IfcTree 
          selectedEntityId={selectedEntityId}
          onSelect={handleSelectEntity}
          searchQuery={searchQuery}
          ifcTypeFilter={ifcTypeFilter}
        />

        <div className="sidebar-status">
          {isLoading ? (
            <span className="status-loading">Loading...</span>
          ) : (
            <span className="status-idle">Ready</span>
          )}
        </div>
      </div>

      <Viewer3D selectedEntityId={selectedEntityId} />

      <PropertiesPanel 
        selectedEntityId={selectedEntityId} 
        onSelectChange={handleSelectEntity}
      />
    </Layout>
  );
}

export default App;
