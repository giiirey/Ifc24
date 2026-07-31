import { useState, useEffect } from 'react';

export interface EntityData {
  id: number;
  global_id: string;
  ifc_type: string | { name?: string };
  name?: string;
  description?: string;
  attributes: [string, string][];
}

interface PropertiesPanelProps {
  selectedEntityId: number | null;
  onSelectChange: (id: number | null) => void;
  entities?: Map<number, EntityData>;
  meshes?: Map<number, any[]>;
}

const mockEntities: Record<number, EntityData> = {
  1: {
    id: 1,
    global_id: '3$jT0DjvX7G$qL5nM0rZ8w',
    ifc_type: 'IfcProject',
    name: 'Example Building Project',
    description: 'IFC example model for visualization testing',
    attributes: [
      ['GlobalId', '3$jT0DjvX7G$qL5nM0rZ8w'],
      ['Name', 'Example Building Project'],
      ['ObjectType', 'Building'],
      ['UnitsInContext', '[object]'],
    ],
  },
  2: {
    id: 2,
    global_id: '1$mK9pL2xR4vN6wQ8yB0c',
    ifc_type: 'IfcSite',
    name: 'Default Site',
    attributes: [
      ['GlobalId', '1$mK9pL2xR4vN6wQ8yB0c'],
      ['Name', 'Default Site'],
      ['LongestEastWestAxis', '100.0'],
      ['LongestNorthSouthAxis', '80.0'],
    ],
  },
  3: {
    id: 3,
    global_id: '2$aB3cD4eF5gH6iJ7kL8m',
    ifc_type: 'IfcBuilding',
    name: 'Default Building',
    attributes: [
      ['GlobalId', '2$aB3cD4eF5gH6iJ7kL8m'],
      ['Name', 'Default Building'],
      ['BuildingHeight', '10.0'],
    ],
  },
  4: {
    id: 4,
    global_id: '4$nP9qR8sT7uV6wX5yZ4a',
    ifc_type: 'IfcBuildingStorey',
    name: 'Ground Floor',
    attributes: [
      ['GlobalId', '4$nP9qR8sT7uV6wX5yZ4a'],
      ['Name', 'Ground Floor'],
      ['StoreyHigh', '3.0'],
    ],
  },
  5: {
    id: 5,
    global_id: '5$bC6dE7fG8hI9jK0lM1n',
    ifc_type: 'IfcWall',
    name: 'Wall - Outer',
    description: 'Outer load-bearing wall',
    attributes: [
      ['GlobalId', '5$bC6dE7fG8hI9jK0lM1n'],
      ['Name', 'Wall - Outer'],
      ['Tag', 'W-001'],
      ['GlobalLength', '12.5'],
      ['GlobalHeight', '3.0'],
    ],
  },
  6: {
    id: 6,
    global_id: '6$oP7qR8sT9uV0wX1yZ2a',
    ifc_type: 'IfcSlab',
    name: 'Slab - Ground',
    description: 'Ground floor structural slab',
    attributes: [
      ['GlobalId', '6$oP7qR8sT9uV0wX1yZ2a'],
      ['Name', 'Slab - Ground'],
      ['Tag', 'SLAB-001'],
      ['GlobalLength', '15.0'],
      ['GlobalWidth', '10.0'],
    ],
  },
  7: {
    id: 7,
    global_id: '7$bC8dE9fG0hI1jK2lM3n',
    ifc_type: 'IfcColumn',
    name: 'Column - C1',
    attributes: [
      ['GlobalId', '7$bC8dE9fG0hI1jK2lM3n'],
      ['Name', 'Column - C1'],
      ['Tag', 'COL-001'],
    ],
  },
  8: {
    id: 8,
    global_id: '8$oP9qR0sT1uV2wX3yZ4a',
    ifc_type: 'IfcDoor',
    name: 'Door - D001',
    attributes: [
      ['GlobalId', '8$oP9qR0sT1uV2wX3yZ4a'],
      ['Name', 'Door - D001'],
      ['Tag', 'DOOR-001'],
      ['Height', '2.1'],
      ['Width', '0.9'],
    ],
  },
  9: {
    id: 9,
    global_id: '9$bC0dE1fG2hI3jK4lM5n',
    ifc_type: 'IfcWindow',
    name: 'Window - W001',
    attributes: [
      ['GlobalId', '9$bC0dE1fG2hI3jK4lM5n'],
      ['Name', 'Window - W001'],
      ['Tag', 'WIN-001'],
      ['Height', '1.2'],
      ['Width', '1.5'],
    ],
  },
  10: {
    id: 10,
    global_id: '10$oP1qR2sT3uV4wX5yZ6a',
    ifc_type: 'IfcBeam',
    name: 'Beam - B001',
    attributes: [
      ['GlobalId', '10$oP1qR2sT3uV4wX5yZ6a'],
      ['Name', 'Beam - B001'],
      ['Tag', 'BEAM-001'],
    ],
  },
};

function getIfcTypeStr(ifcType: string | { name?: string } | undefined): string {
  if (!ifcType) return 'Unknown';
  if (typeof ifcType === 'string') return ifcType;
  if (typeof ifcType === 'object' && ifcType !== null && 'name' in ifcType) return (ifcType as any).name || 'Unknown';
  return String(ifcType);
}

function PropertiesPanel({ selectedEntityId, entities, meshes }: PropertiesPanelProps) {
  const [entity, setEntity] = useState<EntityData | null>(null);

  useEffect(() => {
    if (selectedEntityId !== null) {
      if (entities && entities.has(selectedEntityId)) {
        const data = entities.get(selectedEntityId);
        console.log('[PropertiesPanel] Entity from store:', JSON.stringify(data, null, 2));
        setEntity(data as EntityData || null);
      } else {
        const data = mockEntities[selectedEntityId];
        console.log('[PropertiesPanel] Entity from mock:', JSON.stringify(data, null, 2));
        setEntity(data || null);
      }
    } else {
      setEntity(null);
    }
  }, [selectedEntityId, entities]);

  return (
    <div className="properties-panel">
      <div className="panel-header">
        <h2 style={{ fontSize: '13px', fontWeight: 600 }}>Properties</h2>
      </div>

      {entity ? (
        <div className="properties-content">
          <Section title="Identification">
            <PropertyRow label="Global ID" value={entity.global_id || 'N/A'} copyable />
            <PropertyRow label="Name" value={entity.name || 'N/A'} />
            <PropertyRow label="Type" value={String(entity.ifc_type)} copyable />
            {entity.description && (
              <PropertyRow label="Description" value={entity.description} multiline />
            )}
          </Section>

          <Section title="Attributes">
            {entity.attributes.map((attr, i) => {
              let key: string, value: string;
              if (Array.isArray(attr)) {
                key = String(attr[0] ?? '');
                value = String(attr[1] ?? '');
              } else if (attr && typeof attr === 'object') {
                key = String((attr as any).name ?? (attr as any).key ?? i);
                value = String((attr as any).value ?? '');
              } else {
                key = String(i);
                value = String(attr);
              }
              return <PropertyRow key={`${key}-${i}`} label={key} value={value} copyable={true} />;
            })}
          </Section>

          <Section title="Geometry" collapsible defaultOpen={false}>
            {meshes && selectedEntityId !== null ? (() => {
              const entityMeshes = meshes.get(selectedEntityId);
              if (entityMeshes && entityMeshes.length > 0) {
                const mesh = entityMeshes[0];
                const vertexCount = mesh.vertices ? mesh.vertices.length / 3 : 0;
                const indexCount = mesh.indices ? mesh.indices.length : 0;
                const bbox = mesh.bbox;
                return (
                  <>
                    <PropertyRow label="Vertices" value={vertexCount > 0 ? vertexCount.toLocaleString() : 'N/A'} />
                    <PropertyRow label="Triangles" value={indexCount > 0 ? Math.floor(indexCount / 3).toLocaleString() : 'N/A'} />
                    {bbox && (
                      <PropertyRow 
                        label="Bounding Box" 
                        value={`[${bbox.min.map((n: number) => n.toFixed(2)).join(', ')}, ${bbox.max.map((n: number) => n.toFixed(2)).join(', ')}]`}
                      />
                    )}
                  </>
                );
              }
              return (
                <>
                  <PropertyRow label="Vertices" value="N/A" />
                  <PropertyRow label="Triangles" value="N/A" />
                </>
              );
            })() : (
              <>
                <PropertyRow label="Vertices" value="1,247" />
                <PropertyRow label="Triangles" value="2,891" />
                <PropertyRow label="Volume (m³)" value={(Math.random() * 10).toFixed(3)} />
                <PropertyRow label="Surface Area (m²)" value={(Math.random() * 50).toFixed(2)} />
                <PropertyRow label="Bounding Box" 
                  value={`[${(-Math.random()*10).toFixed(1)}, ${Math.random()*10}.toFixed(1)}, ${(Math.random()*10).toFixed(1)}]`}
                />
              </>
            )}
          </Section>

          <Section title="Relationships" collapsible defaultOpen={false}>
            <PropertyRow label="Contained in" value={`Spatial Structure #${selectedEntityId ? selectedEntityId - 1 : 'N/A'}`} />
            <PropertyRow label="Has openings" value="0" />
            <PropertyRow label="Connected to" value="2 elements" />
          </Section>

          <Section title="Material" collapsible defaultOpen={false}>
            {meshes && selectedEntityId !== null ? (() => {
              const entityMeshes = meshes.get(selectedEntityId);
              if (entityMeshes && entityMeshes.length > 0) {
                const mesh = entityMeshes[0];
                const colorDiffuse = mesh.material?.color_diffuse as [number, number, number] | undefined;
                const colorStr = colorDiffuse 
                  ? `rgb(${Math.round(colorDiffuse[0]*255)}, ${Math.round(colorDiffuse[1]*255)}, ${Math.round(colorDiffuse[2]*255)})`
                  : 'N/A';
                return (
                  <>
                    <PropertyRow label="Name" value={mesh.material?.name || `${getIfcTypeStr(entity.ifc_type)}_material`} />
                    {colorDiffuse && (
                      <div className="property-row" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <span className="property-label">Diffuse Color</span>
                        <div 
                          className="color-swatch" 
                          style={{ 
                            width: '24px', 
                            height: '16px', 
                            borderRadius: '3px',
                            background: colorStr,
                          }} 
                        />
                      </div>
                    )}
                    {mesh.material?.opacity !== undefined && (
                      <PropertyRow label="Opacity" value={mesh.material.opacity.toFixed(2)} />
                    )}
                    {mesh.material?.roughness !== undefined && (
                      <PropertyRow label="Roughness" value={mesh.material.roughness.toFixed(2)} />
                    )}
                  </>
                );
              }
              return (
                <>
                  <PropertyRow label="Name" value={`${getIfcTypeStr(entity.ifc_type)}_material`} />
                  <div className="property-row" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <span className="property-label">Diffuse Color</span>
                    <div 
                      className="color-swatch" 
                      style={{ 
                        width: '24px', 
                        height: '16px', 
                        borderRadius: '3px',
                        background: getMaterialColor(getIfcTypeStr(entity.ifc_type)),
                      }} 
                    />
                  </div>
                  <PropertyRow label="Opacity" value="1.0" />
                  <PropertyRow label="Roughness" value="0.8" />
                </>
              );
            })() : (
              <>
                <PropertyRow label="Name" value={`${getIfcTypeStr(entity.ifc_type)}_material`} />
                <div className="property-row" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <span className="property-label">Diffuse Color</span>
                  <div 
                    className="color-swatch" 
                    style={{ 
                      width: '24px', 
                      height: '16px', 
                      borderRadius: '3px',
                      background: getMaterialColor(getIfcTypeStr(entity.ifc_type)),
                    }} 
                  />
                </div>
                <PropertyRow label="Opacity" value="1.0" />
                <PropertyRow label="Roughness" value="0.8" />
              </>
            )}
          </Section>
        </div>
      ) : (
        <div className="no-selection">
          {selectedEntityId === null ? (
            <p style={{ color: 'var(--color-text-muted)', textAlign: 'center', padding: '24px 16px' }}>
              Select an entity from the tree or viewport to view its properties.
            </p>
          ) : (
            <p style={{ color: 'var(--color-text-muted)', textAlign: 'center', padding: '24px 16px' }}>
              No properties available for this entity.
            </p>
          )}
        </div>
      )}
    </div>
  );
}

function getMaterialColor(ifcType: string): string {
  const colors: Record<string, string> = {
    'IfcWall': '#aaaaaa',
    'IfcSlab': '#888888',
    'IfcColumn': '#666666',
    'IfcBeam': '#777777',
    'IfcDoor': '#8B5E3C',
    'IfcWindow': '#6BA3D9',
  };
  return colors[ifcType] || '#808080';
}

function Section({ 
  title, 
  children, 
  collapsible = false, 
  defaultOpen = true 
}: { 
  title: string; 
  children: React.ReactNode;
  collapsible?: boolean;
  defaultOpen?: boolean;
}) {
  const [isOpen, setIsOpen] = useState(defaultOpen);

  return (
    <div className="properties-section">
      <div 
        className="section-header" 
        onClick={() => collapsible && setIsOpen(!isOpen)}
        style={collapsible ? { cursor: 'pointer' } : undefined}
      >
        <span className="section-toggle">
          {collapsible && (isOpen ? '▼' : '▶')}
        </span>
        <h3>{title}</h3>
      </div>
      {isOpen && <div className="section-content">{children}</div>}
    </div>
  );
}

function PropertyRow({ 
  label, 
  value, 
  copyable = false, 
  multiline = false 
}: { 
  label: string; 
  value: string | number;
  copyable?: boolean;
  multiline?: boolean;
}) {
  const handleCopy = () => {
    navigator.clipboard.writeText(String(value));
  };

  return (
    <div className={`property-row ${multiline ? 'multiline' : ''}`}>
      <span className="property-label">{label}</span>
      <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
        <span className={`property-value ${copyable ? 'copyable' : ''}`} title={String(value)}>
          {value}
        </span>
        {copyable && (
          <button 
            className="copy-btn" 
            onClick={handleCopy}
            title="Copy to clipboard"
          >
            📋
          </button>
        )}
      </div>
    </div>
  );
}

export default PropertiesPanel;
