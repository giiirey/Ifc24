import { Canvas } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import { Suspense, useMemo, useState, useEffect } from 'react';
import { useModelStore } from '../../store/modelStore';
import * as THREE from 'three';

interface Viewer3DProps {
  selectedEntityId: number | null;
  onUpdateView: () => void;
  // Called when an element is clicked in the viewport; selecting through the
  // store also expands and highlights the node in the tree.
  onSelect: (id: number | null) => void;
}

// Accent colour for the selected element in the viewport.
const HIGHLIGHT_COLOR = '#ff7043';
const HIGHLIGHT_EMISSIVE = 0.5;

interface RenderMesh {
  entity_id: number;
  bbox: { min: [number, number, number]; max: [number, number, number] } | null;
  color: [number, number, number];
  opacity: number;
  vertices: number[];
  indices: number[];
  normals: number[];
}

function Viewer3D({ selectedEntityId, onUpdateView, onSelect }: Viewer3DProps) {
  const meshes = useModelStore(state => state.meshes);
  const entities = useModelStore(state => state.entities);
  const dimOpacity = useModelStore(state => state.dimOpacity);
  const viewRevision = useModelStore(state => state.viewRevision);

  const meshRenderData = useMemo(() => {
    const data: RenderMesh[] = [];

    console.log('[Viewer3D] meshes.size =', meshes.size);
    
    meshes.forEach((items, key) => {
      console.log('[Viewer3D] Processing mesh key:', key, 'items:', items.length);
      for (const item of items) {
        const colorDiffuse = (item.material.color_diffuse as unknown as [number, number, number]) || [0.5, 0.5, 0.5];
        data.push({
          entity_id: Number(item.entity_id),
          bbox: item.bbox ? { min: item.bbox.min as [number, number, number], max: item.bbox.max as [number, number, number] } : null,
          color: colorDiffuse,
          opacity: item.material.opacity ?? 0.9,
          vertices: item.vertices || [],
          indices: item.indices || [],
          normals: item.normals || [],
        });
      }
    });

    console.log('[Viewer3D] meshRenderData.length =', data.length);
    if (data.length > 0) {
      console.log('[Viewer3D] First mesh:', {
        entity_id: data[0].entity_id,
        vertices: data[0].vertices.length,
        indices: data[0].indices.length,
        bbox: data[0].bbox
      });
    }

    return data;
  }, [meshes, viewRevision]);

  const selectedMeshes = useMemo(
    () => meshRenderData.filter(mesh => mesh.entity_id === selectedEntityId),
    [meshRenderData, selectedEntityId]
  );

  const hasGeometry = meshes.size > 0;
  console.log('[Viewer3D] hasGeometry =', hasGeometry, 'meshes.size =', meshes.size);

  const [, setTick] = useState(0);
  useEffect(() => {
    const interval = setInterval(() => setTick(t => t + 1), 500);
    return () => clearInterval(interval);
  }, []);

  const { cameraTarget, cameraLookAt } = useMemo(() => {
    if (!hasGeometry) return { 
      cameraTarget: [5, 5, 5] as [number, number, number],
      cameraLookAt: [0, 0, 0] as [number, number, number]
    };

    let minX = Infinity, minY = Infinity, minZ = Infinity;
    let maxX = -Infinity, maxY = -Infinity, maxZ = -Infinity;

    for (const mesh of meshRenderData) {
      if (mesh.bbox) {
        minX = Math.min(minX, mesh.bbox.min[0]);
        minY = Math.min(minY, mesh.bbox.min[1]);
        minZ = Math.min(minZ, mesh.bbox.min[2]);
        maxX = Math.max(maxX, mesh.bbox.max[0]);
        maxY = Math.max(maxY, mesh.bbox.max[1]);
        maxZ = Math.max(maxZ, mesh.bbox.max[2]);
      }
    }

    if (minX === Infinity) return { 
      cameraTarget: [5, 5, 5] as [number, number, number],
      cameraLookAt: [0, 0, 0] as [number, number, number]
    };

    const cx = (minX + maxX) / 2;
    const cy = (minY + maxY) / 2;
    const cz = (minZ + maxZ) / 2;
    const size = Math.max(maxX - minX, maxY - minY, maxZ - minZ);
    const dist = size * 1.5;

    // Rotate coordinates by -PI/2 around X: (x, y, z) -> (x, z, -y)
    const rcx = cx, rcy = cz, rcz = -cy;
    const camPos: [number, number, number] = [rcx + dist, rcy + dist * 0.5, rcz + dist];
    const lookAt: [number, number, number] = [rcx, rcy, rcz];

    console.log('[Viewer3D] cameraTarget:', camPos, 'lookAt:', lookAt, 'size:', size);
    return { cameraTarget: camPos, cameraLookAt: lookAt };
  }, [meshRenderData, hasGeometry]);

  return (
    <div className="viewer-container">
      <div className="viewer-toolbar">
        <span className="toolbar-label">3D Viewport</span>
        <div className="toolbar-actions">
          <button
            type="button"
            className="toolbar-button"
            onClick={onUpdateView}
            title="Clear the selection and restore the original transparency"
          >
            Update view
          </button>
        </div>
      </div>

      <div className="viewer-status">
        {hasGeometry ? (
          <span style={{ fontSize: '12px', color: '#6b8e6b' }}>
            {meshes.size} entities rendered
          </span>
        ) : <span style={{ fontSize: '12px', color: 'var(--color-text-muted)' }}>No geometry data</span>}
        {selectedEntityId !== null && (
          <span style={{ fontSize: '12px', color: HIGHLIGHT_COLOR, marginLeft: '8px' }}>
            Selected: {getIfcType(entities.get(selectedEntityId)?.ifc_type)} —
            {entities.get(selectedEntityId)?.name || `#${selectedEntityId}`}
            {selectedMeshes.length === 0 ? ' (no geometry)' : ''}
          </span>
        )}
      </div>

      <Canvas
        style={{ flex: 1 }}
        camera={{ position: cameraTarget, fov: 50 }}
        dpr={[1, 2]}
        onPointerMissed={() => onSelect(null)}
      >
        <color attach="background" args={['#1a1a2e']} />

        <ambientLight intensity={0.4} />
        <directionalLight position={[10, 10, 5]} intensity={0.8} />
        <directionalLight position={[-10, -5, -10]} intensity={0.2} />

        {hasGeometry ? (
          <>
            <group rotation={[-Math.PI / 2, 0, 0]}>
              <Suspense fallback={null}>
                {meshRenderData.map((mesh, index) => {
                  const selected = mesh.entity_id === selectedEntityId;
                  const dim = selectedEntityId !== null && !selected;

                  if (mesh.vertices.length > 0 && mesh.indices.length > 0) {
                    return (
                      <MeshGeometry
                        key={`${mesh.entity_id}:${index}`}
                        entityId={mesh.entity_id}
                        onSelect={onSelect}
                        vertices={mesh.vertices}
                        indices={mesh.indices}
                        normals={mesh.normals}
                        color={mesh.color}
                        opacity={mesh.opacity}
                        selected={selected}
                        dim={dim}
                        dimOpacity={dimOpacity}
                      />
                    );
                  }

                  return (
                    <BoxMesh
                      key={`${mesh.entity_id}:${index}`}
                      entityId={mesh.entity_id}
                      onSelect={onSelect}
                      bbox={mesh.bbox}
                      color={mesh.color}
                      opacity={mesh.opacity}
                      selected={selected}
                      dim={dim}
                      dimOpacity={dimOpacity}
                    />
                  );
                })}

                {selectedMeshes.length > 0 && <SelectionOutline meshes={selectedMeshes} />}
              </Suspense>
            </group>
          </>
        ) : null}

        <OrbitControls
          enableDamping
          dampingFactor={0.05}
          minDistance={1}
          maxDistance={500}
          target={cameraLookAt}
        />
      </Canvas>
    </div>
  );
}

function MeshGeometry({
  entityId,
  onSelect,
  vertices,
  indices,
  normals,
  color,
  opacity,
  selected,
  dim,
  dimOpacity,
}: {
  entityId: number;
  onSelect: (id: number | null) => void;
  vertices: number[];
  indices: number[];
  normals: number[];
  color: [number, number, number];
  opacity: number;
  selected: boolean;
  dim: boolean;
  dimOpacity: number;
}) {
  // Normals from IfcOpenShell are per face corner. They can only be used as a
  // per-vertex attribute when they match the vertex count one to one; when
  // they are missing (welded meshes emit no normals) the fallback averages
  // normals across faces, so shading has to stay flat to keep faces flat.
  const { geometry, flatShading } = useMemo(() => {
    const geom = new THREE.BufferGeometry();

    const posArray = new Float32Array(vertices);
    geom.setAttribute('position', new THREE.BufferAttribute(posArray, 3));

    if (indices.length > 0) {
      const indexArray = new Uint32Array(indices);
      geom.setIndex(new THREE.BufferAttribute(indexArray, 1));
    }

    const hasUsableNormals = normals.length > 0 && normals.length === vertices.length;
    if (hasUsableNormals) {
      const normalArray = new Float32Array(normals);
      geom.setAttribute('normal', new THREE.BufferAttribute(normalArray, 3));
    } else {
      geom.computeVertexNormals();
    }

    return { geometry: geom, flatShading: !hasUsableNormals };
  }, [vertices, indices, normals]);

  useEffect(() => () => geometry.dispose(), [geometry]);

  const baseColor = colorToHex(color);
  const materialColor = selected ? HIGHLIGHT_COLOR : baseColor;

  return (
    <mesh
      geometry={geometry}
      onClick={(e) => {
        e.stopPropagation();
        onSelect(entityId);
      }}
    >
      <meshStandardMaterial
        color={materialColor}
        emissive={materialColor}
        emissiveIntensity={selected ? HIGHLIGHT_EMISSIVE : 0.1}
        transparent={opacity < 1 || dim}
        opacity={dim ? Math.min(opacity, dimOpacity) : opacity}
        roughness={0.7}
        metalness={0.1}
        flatShading={flatShading}
        side={THREE.DoubleSide}
      />
    </mesh>
  );
}

function BoxMesh({
  entityId,
  onSelect,
  bbox,
  color,
  opacity,
  selected,
  dim,
  dimOpacity,
}: {
  entityId: number;
  onSelect: (id: number | null) => void;
  bbox: { min: [number, number, number]; max: [number, number, number] } | null;
  color: [number, number, number];
  opacity: number;
  selected: boolean;
  dim: boolean;
  dimOpacity: number;
}) {
  const geometry = useMemo(() => {
    if (bbox && bbox.min && bbox.max) {
      const center = [(bbox.min[0] + bbox.max[0]) / 2, (bbox.min[1] + bbox.max[1]) / 2, (bbox.min[2] + bbox.max[2]) / 2];
      const size = [
        Math.abs(bbox.max[0] - bbox.min[0]),
        Math.abs(bbox.max[1] - bbox.min[1]),
        Math.abs(bbox.max[2] - bbox.min[2]),
      ];

      return { center: center as [number, number, number], size };
    }

    return { center: [0, 0.5, 0] as [number, number, number], size: [0.5, 1, 0.5] };
  }, [bbox]);

  const width = Math.max(geometry.size[0], 0.01);
  const height = Math.max(geometry.size[1], 0.01);
  const depth = Math.max(geometry.size[2], 0.01);

  const baseColor = colorToHex(color);
  const materialColor = selected ? HIGHLIGHT_COLOR : baseColor;

  return (
    <mesh
      position={geometry.center}
      onClick={(e) => {
        e.stopPropagation();
        onSelect(entityId);
      }}
    >
      <boxGeometry args={[width, height, depth]} />
      <meshStandardMaterial
        color={materialColor}
        emissive={materialColor}
        emissiveIntensity={selected ? HIGHLIGHT_EMISSIVE : 0.3}
        transparent={opacity < 1 || dim}
        opacity={dim ? Math.min(opacity, dimOpacity) : opacity}
        roughness={0.7}
        metalness={0.1}
      />
    </mesh>
  );
}

// Wireframe bounding box around the selected entity, drawn on top of the geometry.
function SelectionOutline({ meshes }: { meshes: RenderMesh[] }) {
  const boxes = meshes
    .filter(mesh => mesh.bbox && mesh.bbox.min && mesh.bbox.max)
    .map(mesh => ({
      center: [
        (mesh.bbox!.min[0] + mesh.bbox!.max[0]) / 2,
        (mesh.bbox!.min[1] + mesh.bbox!.max[1]) / 2,
        (mesh.bbox!.min[2] + mesh.bbox!.max[2]) / 2,
      ] as [number, number, number],
      size: [
        Math.max(Math.abs(mesh.bbox!.max[0] - mesh.bbox!.min[0]), 0.02),
        Math.max(Math.abs(mesh.bbox!.max[1] - mesh.bbox!.min[1]), 0.02),
        Math.max(Math.abs(mesh.bbox!.max[2] - mesh.bbox!.min[2]), 0.02),
      ] as [number, number, number],
    }));

  const geometry = useMemo(() => {
    const box = new THREE.BoxGeometry(1, 1, 1);
    const edges = new THREE.EdgesGeometry(box);
    box.dispose();
    return edges;
  }, []);

  useEffect(() => () => geometry.dispose(), [geometry]);

  if (boxes.length === 0) return null;

  return (
    <>
      {boxes.map((box, index) => (
        <lineSegments
          key={index}
          geometry={geometry}
          position={box.center}
          scale={box.size}
          renderOrder={999}
          raycast={() => null}
        >
          <lineBasicMaterial color={HIGHLIGHT_COLOR} depthTest={false} />
        </lineSegments>
      ))}
    </>
  );
}

function getIfcType(ifcType: string | { name?: string } | undefined): string {
  if (!ifcType) return 'Unknown';
  if (typeof ifcType === 'string') return ifcType;
  return ifcType.name || 'Unknown';
}

function colorToHex(rgb: [number, number, number]): string {
  const toHex = (n: number) => {
    const hex = Math.round(Math.max(0, Math.min(255, n * 255))).toString(16);
    return hex.length === 1 ? '0' + hex : hex;
  };
  return `#${toHex(rgb[0])}${toHex(rgb[1])}${toHex(rgb[2])}`;
}

export default Viewer3D;
