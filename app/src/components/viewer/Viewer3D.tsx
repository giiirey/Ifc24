import { Canvas } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import { Suspense, useMemo, useState, useEffect } from 'react';
import { useModelStore } from '../../store/modelStore';
import * as THREE from 'three';

interface Viewer3DProps {
  selectedEntityId: number | null;
}

function Viewer3D({ selectedEntityId }: Viewer3DProps) {
  const meshes = useModelStore(state => state.meshes);

  const meshRenderData = useMemo(() => {
    const data: Array<{
      entity_id: number;
      bbox: { min: [number, number, number]; max: [number, number, number] } | null;
      color: [number, number, number];
      opacity: number;
      vertices: number[];
      indices: number[];
      normals: number[];
    }> = [];

    console.log('[Viewer3D] meshes.size =', meshes.size);
    
    meshes.forEach((items, key) => {
      console.log('[Viewer3D] Processing mesh key:', key, 'items:', items.length);
      for (const item of items) {
        const colorDiffuse = (item.material.color_diffuse as unknown as [number, number, number]) || [0.5, 0.5, 0.5];
        data.push({
          entity_id: item.entity_id,
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
  }, [meshes]);

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
      </div>

      <div className="viewer-status">
        {hasGeometry ? (
          <span style={{ fontSize: '12px', color: '#6b8e6b' }}>
            {meshes.size} entities rendered
          </span>
        ) : <span style={{ fontSize: '12px', color: 'var(--color-text-muted)' }}>No geometry data</span>}
      </div>

      <Canvas
        style={{ flex: 1 }}
        camera={{ position: cameraTarget, fov: 50 }}
        dpr={[1, 2]}
      >
        <color attach="background" args={['#1a1a2e']} />

        <ambientLight intensity={0.4} />
        <directionalLight position={[10, 10, 5]} intensity={0.8} />
        <directionalLight position={[-10, -5, -10]} intensity={0.2} />

        {hasGeometry ? (
          <>
            <group rotation={[-Math.PI / 2, 0, 0]}>
              <Suspense fallback={null}>
                {meshRenderData.map((mesh) => (
                  mesh.vertices.length > 0 && mesh.indices.length > 0 ? (
                    <MeshGeometry
                      key={mesh.entity_id}
                      vertices={mesh.vertices}
                      indices={mesh.indices}
                      normals={mesh.normals}
                      color={mesh.color}
                      opacity={mesh.opacity}
                    />
                  ) : (
                    <BoxMesh
                      key={mesh.entity_id}
                      selectedEntityId={selectedEntityId}
                      bbox={mesh.bbox}
                      color={mesh.color}
                      opacity={mesh.opacity}
                    />
                  )
                ))}
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
  vertices,
  indices,
  normals,
  color,
  opacity,
}: {
  vertices: number[];
  indices: number[];
  normals: number[];
  color: [number, number, number];
  opacity: number;
}) {
  const geometry = useMemo(() => {
    const geom = new THREE.BufferGeometry();

    const posArray = new Float32Array(vertices);
    geom.setAttribute('position', new THREE.BufferAttribute(posArray, 3));

    if (indices.length > 0) {
      const indexArray = new Uint32Array(indices);
      geom.setIndex(new THREE.BufferAttribute(indexArray, 1));
    }

    if (normals.length > 0 && normals.length === vertices.length) {
      const normalArray = new Float32Array(normals);
      geom.setAttribute('normal', new THREE.BufferAttribute(normalArray, 3));
    } else {
      geom.computeVertexNormals();
    }

    return geom;
  }, [vertices, indices, normals]);

  return (
    <mesh geometry={geometry}>
      <meshStandardMaterial
        color={colorToHex(color)}
        emissive={colorToHex(color)}
        emissiveIntensity={0.1}
        transparent={opacity < 1}
        opacity={opacity}
        roughness={0.7}
        metalness={0.1}
        side={THREE.DoubleSide}
      />
    </mesh>
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
