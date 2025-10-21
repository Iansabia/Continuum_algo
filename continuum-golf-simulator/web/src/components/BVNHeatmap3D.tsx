import { useRef, useMemo, useState } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls, Text, Grid } from '@react-three/drei';
import * as THREE from 'three';

interface BVNHeatmap3DProps {
  sigmaX: number;
  sigmaY: number;
  currentPmax: number;
  width?: number;
  height?: number;
}

// Generate P_max surface data over (σ_x, σ_y) space
function generatePmaxSurface(centerX: number, centerY: number): {
  vertices: Float32Array;
  colors: Float32Array;
  indices: Uint16Array;
} {
  const resolution = 50; // 50x50 grid
  const rangeX = [Math.max(1, centerX - 10), centerX + 10]; // ±10 yards from center
  const rangeY = [Math.max(1, centerY - 10), centerY + 10];

  const vertices: number[] = [];
  const colors: number[] = [];
  const indices: number[] = [];

  // Calculate P_max for each (σ_x, σ_y) point
  const calculatePmax = (sigX: number, sigY: number): number => {
    // P_max formula: exp(0.5) ≈ 1.649 for optimal pricing
    // But we can show variation based on dispersion
    const avgSigma = Math.sqrt(sigX * sigX + sigY * sigY) / Math.SQRT2;
    const baselineDispersion = 10; // yards
    const ratio = baselineDispersion / avgSigma;
    return Math.exp(0.5) * Math.max(0.5, Math.min(2.0, ratio));
  };

  // Color mapping: P_max to RGB (red → yellow → green)
  const pmaxToColor = (pmax: number): [number, number, number] => {
    // Normalize P_max to 0-1 range (assuming P_max is between 0.5 and 3.0)
    const normalized = (pmax - 0.5) / 2.5;

    if (normalized < 0.5) {
      // Red to Yellow
      const t = normalized * 2;
      return [1, t, 0];
    } else {
      // Yellow to Green
      const t = (normalized - 0.5) * 2;
      return [1 - t, 1, 0];
    }
  };

  // Generate grid points
  for (let i = 0; i <= resolution; i++) {
    for (let j = 0; j <= resolution; j++) {
      const sigX = rangeX[0] + (rangeX[1] - rangeX[0]) * (i / resolution);
      const sigY = rangeY[0] + (rangeY[1] - rangeY[0]) * (j / resolution);
      const pmax = calculatePmax(sigX, sigY);

      // Vertex position (normalized to -5 to +5 range for better visualization)
      const x = (i / resolution - 0.5) * 10;
      const z = (j / resolution - 0.5) * 10;
      const y = (pmax - 1.0) * 5; // Scale P_max for visibility

      vertices.push(x, y, z);

      // Color based on P_max
      const [r, g, b] = pmaxToColor(pmax);
      colors.push(r, g, b);

      // Generate triangle indices (two triangles per quad)
      if (i < resolution && j < resolution) {
        const topLeft = i * (resolution + 1) + j;
        const topRight = topLeft + 1;
        const bottomLeft = (i + 1) * (resolution + 1) + j;
        const bottomRight = bottomLeft + 1;

        // First triangle
        indices.push(topLeft, bottomLeft, topRight);
        // Second triangle
        indices.push(topRight, bottomLeft, bottomRight);
      }
    }
  }

  return {
    vertices: new Float32Array(vertices),
    colors: new Float32Array(colors),
    indices: new Uint16Array(indices),
  };
}

function Surface({ sigmaX, sigmaY }: { sigmaX: number; sigmaY: number }) {
  const meshRef = useRef<THREE.Mesh>(null);
  const [hovered, setHovered] = useState(false);

  const { vertices, colors, indices } = useMemo(
    () => generatePmaxSurface(sigmaX, sigmaY),
    [sigmaX, sigmaY]
  );

  // Gentle rotation animation
  useFrame((state) => {
    if (meshRef.current && !hovered) {
      meshRef.current.rotation.y = Math.sin(state.clock.elapsedTime * 0.2) * 0.1;
    }
  });

  return (
    <mesh
      ref={meshRef}
      onPointerEnter={() => setHovered(true)}
      onPointerLeave={() => setHovered(false)}
    >
      <bufferGeometry>
        <bufferAttribute
          attach="attributes-position"
          count={vertices.length / 3}
          array={vertices}
          itemSize={3}
        />
        <bufferAttribute
          attach="attributes-color"
          count={colors.length / 3}
          array={colors}
          itemSize={3}
        />
        <bufferAttribute attach="index" count={indices.length} array={indices} itemSize={1} />
      </bufferGeometry>
      <meshStandardMaterial vertexColors side={THREE.DoubleSide} />
    </mesh>
  );
}

function AxisLabels() {
  return (
    <>
      {/* X-axis label (σ_x) */}
      <Text
        position={[6, -3, 0]}
        fontSize={0.5}
        color="#dfc9ad"
        anchorX="center"
        anchorY="middle"
      >
        σ_x (yards)
      </Text>

      {/* Y-axis label (P_max) */}
      <Text
        position={[0, 5, 0]}
        fontSize={0.5}
        color="#dfc9ad"
        anchorX="center"
        anchorY="middle"
      >
        P_max
      </Text>

      {/* Z-axis label (σ_y) */}
      <Text
        position={[0, -3, 6]}
        fontSize={0.5}
        color="#dfc9ad"
        anchorX="center"
        anchorY="middle"
      >
        σ_y (yards)
      </Text>
    </>
  );
}

function CurrentPositionMarker({ pmax }: { sigmaX: number; sigmaY: number; pmax: number }) {
  const markerRef = useRef<THREE.Mesh>(null);

  useFrame((state) => {
    if (markerRef.current) {
      markerRef.current.position.y = (pmax - 1.0) * 5 + 0.3 + Math.sin(state.clock.elapsedTime * 2) * 0.1;
    }
  });

  return (
    <mesh ref={markerRef} position={[0, (pmax - 1.0) * 5 + 0.3, 0]}>
      <sphereGeometry args={[0.2, 16, 16]} />
      <meshStandardMaterial color="#D4AF37" emissive="#D4AF37" emissiveIntensity={0.5} />
    </mesh>
  );
}

export default function BVNHeatmap3D({
  sigmaX,
  sigmaY,
  currentPmax,
}: BVNHeatmap3DProps) {
  return (
    <div className="bg-gray-800 p-4 rounded-lg border-2 border-brand-deep-purple">
      <h3 className="text-lg font-semibold text-brand-tan mb-4">
        3D P_max Surface (BVN Distribution)
      </h3>

      <div className="w-full aspect-[6/5] max-w-[600px] mx-auto">
        <Canvas camera={{ position: [8, 6, 8], fov: 50 }}>
          {/* Lighting */}
          <ambientLight intensity={0.5} />
          <directionalLight position={[10, 10, 5]} intensity={1} />
          <pointLight position={[-10, -10, -5]} intensity={0.5} />

          {/* 3D Surface */}
          <Surface sigmaX={sigmaX} sigmaY={sigmaY} />

          {/* Current position marker */}
          <CurrentPositionMarker sigmaX={sigmaX} sigmaY={sigmaY} pmax={currentPmax} />

          {/* Grid on base plane */}
          <Grid
            args={[10, 10]}
            cellSize={1}
            cellThickness={0.5}
            cellColor="#604c9c"
            sectionSize={5}
            sectionThickness={1}
            sectionColor="#493b7c"
            fadeDistance={30}
            fadeStrength={1}
            followCamera={false}
            position={[0, -3, 0]}
          />

          {/* Axis labels */}
          <AxisLabels />

          {/* Interactive controls */}
          <OrbitControls
            enablePan={true}
            enableZoom={true}
            enableRotate={true}
            minDistance={5}
            maxDistance={20}
          />
        </Canvas>
      </div>

      {/* Legend */}
      <div className="mt-4 flex justify-center gap-6 text-sm">
        <div className="flex items-center gap-2">
          <div className="w-4 h-4 bg-red-500"></div>
          <span className="text-gray-400">Low P_max</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-4 h-4 bg-yellow-500"></div>
          <span className="text-gray-400">Medium P_max</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-4 h-4 bg-green-500"></div>
          <span className="text-gray-400">High P_max</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-golf-gold"></div>
          <span className="text-gray-400">Current Position</span>
        </div>
      </div>

      {/* Info text */}
      <div className="mt-4 text-xs text-gray-500 text-center">
        <p>Interactive 3D visualization of P_max across (σ_x, σ_y) space</p>
        <p>Drag to rotate • Scroll to zoom • Right-click to pan</p>
      </div>
    </div>
  );
}
