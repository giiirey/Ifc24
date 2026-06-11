import { useState, useEffect, useMemo } from 'react';

interface TreeNodeData {
  id: number;
  name: string;
  ifc_type: string;
  children: TreeNodeData[];
}

interface IfcTreeProps {
  selectedEntityId: number | null;
  onSelect: (id: number | null) => void;
  searchQuery: string;
  ifcTypeFilter: string;
}

const mockTreeData: TreeNodeData = {
  id: 1,
  name: 'IfcProject',
  ifc_type: 'IfcProject',
  children: [
    {
      id: 2,
      name: 'Default Site',
      ifc_type: 'IfcSite',
      children: [
        {
          id: 3,
          name: 'Default Building',
          ifc_type: 'IfcBuilding',
          children: [
            {
              id: 4,
              name: 'Ground Floor',
              ifc_type: 'IfcBuildingStorey',
              children: [
                { id: 5, name: 'Wall - Outer', ifc_type: 'IfcWall', children: [] },
                { id: 6, name: 'Slab - Ground', ifc_type: 'IfcSlab', children: [] },
                { id: 7, name: 'Column - C1', ifc_type: 'IfcColumn', children: [] },
                { id: 8, name: 'Door - D001', ifc_type: 'IfcDoor', children: [] },
                { id: 9, name: 'Window - W001', ifc_type: 'IfcWindow', children: [] },
                { id: 10, name: 'Beam - B001', ifc_type: 'IfcBeam', children: [] },
              ],
            },
            {
              id: 11,
              name: 'First Floor',
              ifc_type: 'IfcBuildingStorey',
              children: [
                { id: 12, name: 'Wall - Outer', ifc_type: 'IfcWall', children: [] },
                { id: 13, name: 'Slab - Upper', ifc_type: 'IfcSlab', children: [] },
                { id: 14, name: 'Column - C2', ifc_type: 'IfcColumn', children: [] },
              ],
            },
          ],
        },
      ],
    },
  ],
};

function IfcTree({ selectedEntityId, onSelect, searchQuery, ifcTypeFilter }: IfcTreeProps) {
  const [treeData] = useState<TreeNodeData>(mockTreeData);
  const [expandedIds, setExpandedIds] = useState<Set<number>>(new Set([1, 2, 3]));

  useEffect(() => {
    // Expand all when searching/filtering
    if (searchQuery || ifcTypeFilter) {
      setExpandedIds(new Set([1, 2, 3, 4, 11]));
    }
  }, [searchQuery, ifcTypeFilter]);

  const toggleExpand = (id: number) => {
    const next = new Set(expandedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    setExpandedIds(next);
  };

  const filteredTree = useMemo(() => {
    return filterNode(treeData, searchQuery, ifcTypeFilter, expandedIds);
  }, [treeData, searchQuery, ifcTypeFilter, expandedIds]);

  return (
    <div className="tree-container">
      <TreeNode node={filteredTree} depth={0} selectedEntityId={selectedEntityId} onSelect={onSelect} toggleExpand={toggleExpand} expandedIds={expandedIds} />
      {filteredTree.children.length === 0 && !searchQuery && ifcTypeFilter && (
        <div style={{ padding: '12px', color: 'var(--color-text-muted)', textAlign: 'center' }}>
          No matching entities
        </div>
      )}
      {filteredTree.children.length === 0 && searchQuery && (
        <div style={{ padding: '12px', color: 'var(--color-text-muted)', textAlign: 'center' }}>
          No results for "{searchQuery}"
        </div>
      )}
    </div>
  );
}

function filterNode(
  node: TreeNodeData,
  search: string,
  typeFilter: string,
  expanded: Set<number>
): TreeNodeData {
  let matches = true;
  
  if (search) {
    const q = search.toLowerCase();
    const nameMatch = node.name.toLowerCase().includes(q);
    const typeMatch = node.ifc_type.toLowerCase().includes(q);
    if (!nameMatch && !typeMatch) {
      matches = false;
    }
  }

  if (typeFilter && node.ifc_type !== typeFilter) {
    // Allow if any descendant matches
    const childMatches = node.children.some(c => c.ifc_type === typeFilter);
    if (!childMatches) {
      matches = false;
    }
  }

  const filteredChildren = node.children.map(child => filterNode(child, search, typeFilter, expanded));
  const childrenWithDepth = (node.ifc_type !== 'IfcProject' || search || typeFilter) 
    ? filteredChildren 
    : filteredChildren.filter(c => c.name !== 'Default Building');

  return {
    ...node,
    children: childrenWithDepth,
  };
}

function TreeNode({ 
  node, 
  depth, 
  selectedEntityId, 
  onSelect, 
  toggleExpand, 
  expandedIds 
}: { 
  node: TreeNodeData; 
  depth: number; 
  selectedEntityId: number | null;
  onSelect: (id: number | null) => void;
  toggleExpand: (id: number) => void;
  expandedIds: Set<number>;
}) {
  const isExpanded = expandedIds.has(node.id);
  const isSelected = selectedEntityId === node.id;
  const hasChildren = node.children.length > 0;

  return (
    <div>
      <div
        className={`tree-node ${isSelected ? 'selected' : ''}`}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
        onClick={() => onSelect(node.id)}
        onDoubleClick={() => hasChildren && toggleExpand(node.id)}
      >
        <span 
          className="expand-toggle"
          onClick={(e) => {
            e.stopPropagation();
            if (hasChildren) toggleExpand(node.id);
          }}
          style={{ minWidth: '16px', display: 'inline-block', color: 'var(--color-text-muted)', fontSize: '10px' }}
        >
          {hasChildren ? (isExpanded ? '▼' : '▶') : ' '}
        </span>
        <span className="tree-icon">{getTypeIcon(node.ifc_type)}</span>
        <span className={`type-badge type-${node.ifc_type.replace('Ifc', '')}`}>
          {node.ifc_type}
        </span>
        <span className="tree-name" title={node.name}>{truncateName(node.name, 28)}</span>
      </div>
      {isExpanded && hasChildren && node.children.map(child => (
        <TreeNode 
          key={child.id} 
          node={child} 
          depth={depth + 1} 
          selectedEntityId={selectedEntityId}
          onSelect={onSelect}
          toggleExpand={toggleExpand}
          expandedIds={expandedIds}
        />
      ))}
    </div>
  );
}

function getTypeIcon(ifcType: string): string {
  const icons: Record<string, string> = {
    'IfcProject': '🏢',
    'IfcSite': '🌍',
    'IfcBuilding': '🏗️',
    'IfcBuildingStorey': '📐',
    'IfcWall': '🧱',
    'IfcSlab': '▬',
    'IfcColumn': '|',
    'IfcBeam': '--',
    'IfcDoor': '🚪',
    'IfcWindow': '🪟',
    'IfcSpace': '📦',
  };
  return icons[ifcType] || '•';
}

function truncateName(name: string, maxLength: number): string {
  if (name.length <= maxLength) return name;
  return name.slice(0, maxLength - 3) + '...';
}

export default IfcTree;
