import { Canvas } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import { Suspense, useMemo, useState, useEffect } from 'react';
import { useModelStore } from '../../store/modelStore';

interface Viewer3DProps {
  selectedEntityId: number | null;
  onOpenFile?: () => void;
}

function Viewer3D({ selectedEntityId, onOpenFile }: Viewer3DProps) {
  const meshes = useModelStore(state => state.meshes);
  const loadedFilePath = useModelStore(state => state.loadedFilePath);

  const meshRenderData = useMemo(() => {
    const data: Array<{ entity_id: number; bbox: { min: [number, number, number]; max: [number, number, number] } | null; color: [number, number, number]; opacity: number }> = [];
    
    meshes.forEach((items) => {
      for (const item of items) {
        const colorDiffuse = (item.material.color_diffuse as unknown as [number, number, number]) || [0.5, 0.5, 0.5];
        data.push({
          entity_id: item.entity_id,
          bbox: item.bbox ? { min: item.bbox.min as [number, number, number], max: item.bbox.max as [number, number, number] } : null,
          color: colorDiffuse,
          opacity: item.material.opacity ?? 0.9,
        });
      }
    });

    return data;
  }, [meshes]);

  const hasGeometry = meshes.size > 0;

  // Debug: log mesh state on every render
  console.log(`[Viewer3D] RE_RENDER - meshes.size=${meshes.size}, loadedFilePath=${loadedFilePath ? 'true' : 'false'}, hasGeometry=${hasGeometry}`);
  if (meshes.size > 0) {
    const [firstKey, firstVal] = meshes.entries().next().value;
    console.log('[Viewer3D] Sample mesh key:', firstKey);
    console.log('[Viewer3D] Sample mesh val:', firstVal);
  }

  const [tick, setTick] = useState(0);
  useEffect(() => {
    // Force re-render loop to detect state changes
    const interval = setInterval(() => setTick(t => t + 1), 500);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="viewer-container">
      <div className="viewer-toolbar">
        <span className="toolbar-label">3D Viewport</span>
        {hasGeometry ? (
          <span style={{ fontSize: '12px', color: '#6b8e6b' }}>
            {meshes.size} entities rendered
          </span>
        ) : loadedFilePath ? (
          <span style={{ fontSize: '12px', color: 'var(--color-text-muted)' }}>No geometry data</span>
        ) : onOpenFile ? (
          <button
            className="toolbar-button"
            onClick={onOpenFile}
          >
            Open IFC File
          </button>
        ) : null}
      </div>

      <Canvas
        style={{ flex: 1 }}
        camera={{ position: [5, 5, 5], fov: 50 }}
        dpr={[1, 2]}
      >
        <color attach="background" args={['#1a1a2e']} />

        <ambientLight intensity={0.4} />
        <directionalLight position={[10, 10, 5]} intensity={0.8} />
        <directionalLight position={[-10, -5, -10]} intensity={0.2} />

        {hasGeometry ? (
          <>
            <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, -0.01, 0]}>
              <planeGeometry args={[100, 100]} />
              <meshStandardMaterial
                color="#1e2746"
                roughness={1}
                metalness={0}
              />
            </mesh>
            <Suspense fallback={null}>
              {meshRenderData.map((mesh) => (
                <BoxMesh
                  key={mesh.entity_id}
                  selectedEntityId={selectedEntityId}
                  bbox={mesh.bbox}
                  color={mesh.color}
                  opacity={mesh.opacity}
                />
              ))}
            </Suspense>
          </>
        ) : null}

        <OrbitControls
          enableDamping
          dampingFactor={0.05}
          minDistance={1}
          maxDistance={100}
        />
      </Canvas>
    </div>
  );
}

function BoxMesh({
  bbox,
  color,
  opacity
}: {
  selectedEntityId?: number | null;
  entity_id?: number;
  bbox: { min: [number, number, number]; max: [number, number, number] } | null;
  color: [number, number, number];
  opacity: number;
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

  return (
    <mesh position={geometry.center}>
      <boxGeometry args={[width, height, depth]} />
      <meshStandardMaterial
        color={colorToHex(color)}
        emissive={colorToHex(color)}
        emissiveIntensity={0.3}
        transparent={opacity < 1}
        opacity={opacity}
        roughness={0.7}
        metalness={0.1}
      />
    </mesh>
  );
}

function colorToHex(rgb: [number, number, number]): string {
  const toHex = (n: number) => {
    const hex = Math.round(Math.max(0, Math.min(255, n * 255))).toString(16);
    return hex.length === 1 ? '0' + hex : hex;
  };
  return `#${toHex(rgb[0])}${toHex(rgb[1])}${toHex(rgb[2])}`;
}

export default Viewer3D;
