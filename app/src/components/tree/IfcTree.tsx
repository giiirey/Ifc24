import { useMemo } from 'react';

export interface TreeNodeData {
  id: number;
  name: string;
  ifc_type: string;
  children: TreeNodeData[];
}

interface IfcTreeProps {
  treeData: TreeNodeData[];
  selectedEntityId: number | null;
  onSelect: (id: number | null) => void;
  searchQuery: string;
  ifcTypeFilter: string;
  expandedIds: Set<number>;
  toggleExpand: (id: number) => void;
}

function IfcTree({ treeData, selectedEntityId, onSelect, searchQuery, ifcTypeFilter, expandedIds, toggleExpand }: IfcTreeProps) {
  const filteredTree = useMemo(() => {
    if (!treeData || treeData.length === 0) return [];
    
    return treeData
      .map(node => filterNodeRecursively(node, searchQuery, ifcTypeFilter, expandedIds))
      .filter(Boolean) as TreeNodeData[];
  }, [treeData, searchQuery, ifcTypeFilter, expandedIds]);

  if (!treeData || treeData.length === 0) {
    return (
      <div className="tree-container">
        <div style={{ padding: '12px', color: 'var(--color-text-muted)', textAlign: 'center' }}>
          No hierarchy data available
        </div>
      </div>
    );
  }

  if (filteredTree.length === 0 && (searchQuery || ifcTypeFilter)) {
    return (
      <div className="tree-container">
        <div style={{ padding: '12px', color: 'var(--color-text-muted)', textAlign: 'center' }}>
          {searchQuery ? `No results for "${searchQuery}"` : 'No matching entities'}
        </div>
      </div>
    );
  }

  return (
    <div className="tree-container">
      {filteredTree.map(node => (
        <TreeNode
          key={node.id}
          node={node}
          depth={0}
          selectedEntityId={selectedEntityId}
          onSelect={onSelect}
          toggleExpand={toggleExpand}
          expandedIds={expandedIds}
        />
      ))}
    </div>
  );
}

function filterNodeRecursively(node: TreeNodeData, search: string, typeFilter: string, expanded: Set<number>): TreeNodeData | null {
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
    const childMatches = node.children.some(c => c.ifc_type === typeFilter);
    if (!childMatches) {
      matches = false;
    }
  }

  const filteredChildren = node.children
    .map(child => filterNodeRecursively(child, search, typeFilter, expanded))
    .filter(Boolean) as TreeNodeData[];

  if (matches || filteredChildren.length > 0) {
    return {
      ...node,
      children: search || typeFilter ? filteredChildren : node.children,
    };
  }

  return null;
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
      >
        <span 
          className="expand-toggle"
          onClick={(e) => {
            e.stopPropagation();
            if (hasChildren) toggleExpand(node.id);
          }}
          style={{ minWidth: '16px', display: 'inline-block', color: 'var(--color-text-muted)', fontSize: '10px', cursor: hasChildren ? 'pointer' : 'default' }}
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
    'IfcProject': '\uD83C\uDFE2',
    'IfcSite': '\uD83C\uDF0D',
    'IfcBuilding': '\uD83C\uDFDA',
    'IfcBuildingStorey': '\uD83D\uDCC0',
    'IfcWall': '\uD83E\uDDF1',
    'IfcSlab': '\u25AC',
    'IfcColumn': '|',
    'IfcBeam': '--',
    'IfcDoor': '\uD83D\uDEAA',
    'IfcWindow': '\uD83E\ude9F',
    'IfcSpace': '\uD83D\uDCE6',
  };
  return icons[ifcType] || '\u2022';
}

function truncateName(name: string, maxLength: number): string {
  if (name.length <= maxLength) return name;
  return name.slice(0, maxLength - 3) + '...';
}

export default IfcTree;
