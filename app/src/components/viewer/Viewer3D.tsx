import { Canvas } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import { Suspense } from 'react';

interface Viewer3DProps {
  selectedEntityId: number | null;
}

function Viewer3D({ selectedEntityId }: Viewer3DProps) {
  return (
    <div className="viewer-container">
      <div className="viewer-toolbar">
        <span className="toolbar-label">3D Viewport</span>
        {selectedEntityId !== null && (
          <span className="toolbar-selection">
            Selected: Entity #{selectedEntityId}
          </span>
        )}
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
        
        <GridHelper />
        
        <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, -0.01, 0]}>
          <planeGeometry args={[100, 100]} />
          <meshStandardMaterial 
            color="#1e2746" 
            roughness={1} 
            metalness={0} 
          />
        </mesh>

        {selectedEntityId && <SelectedBox id={selectedEntityId} />}
        
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

function GridHelper() {
  return (
    <gridHelper args={[50, 50, '#2a3a6a', '#1e2a4a']} position={[0, 0.01, 0]} />
  );
}

function SelectedBox({ id }: { id: number }) {
  return (
    <mesh position={[0, 1, 0]}>
      <boxGeometry args={[1, 2, 1]} />
      <meshStandardMaterial 
        color="#ffd700" 
        emissive="#ffd700" 
        emissiveIntensity={0.3}
        transparent
        opacity={0.6}
      />
    </mesh>
  );
}

export default Viewer3D;
