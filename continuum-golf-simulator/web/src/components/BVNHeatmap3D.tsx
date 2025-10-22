import { useRef, useMemo } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls, Text, Grid } from '@react-three/drei';
import * as THREE from 'three';

interface Shot {
  distance: number;
  angle: number;
  wager: number;
  payout: number;
  profit: number;
  x?: number;
  y?: number;
}

interface BVNHeatmap3DProps {
  sigmaX: number;
  sigmaY: number;
  currentPmax: number;
  shots: Shot[];
  width?: number;
  height?: number;
}

// Generate BVN probability density surface
function generateBVNSurface(sigmaX: number, sigmaY: number): {
  vertices: Float32Array;
  colors: Float32Array;
  indices: Uint16Array;
} {
  const resolution = 60; // Higher resolution for smoother surface
  const range = 3; // Show ±3σ range
  const maxX = sigmaX * range;
  const maxY = sigmaY * range;

  const vertices: number[] = [];
  const colors: number[] = [];
  const indices: number[] = [];

  // Bivariate Normal PDF
  const bvnPDF = (x: number, y: number): number => {
    const exponent = -0.5 * ((x * x) / (sigmaX * sigmaX) + (y * y) / (sigmaY * sigmaY));
    return Math.exp(exponent) / (2 * Math.PI * sigmaX * sigmaY);
  };

  let maxDensity = bvnPDF(0, 0);

  // Generate grid points
  for (let i = 0; i <= resolution; i++) {
    for (let j = 0; j <= resolution; j++) {
      // Position in yards
      const x = -maxX + (2 * maxX * i) / resolution;
      const y = -maxY + (2 * maxY * j) / resolution;

      const density = bvnPDF(x, y);
      const normalizedDensity = density / maxDensity;

      // Scale for 3D visualization
      const xScaled = (x / maxX) * 5;
      const zScaled = (y / maxY) * 5;
      const yScaled = normalizedDensity * 3; // Height represents probability density

      vertices.push(xScaled, yScaled, zScaled);

      // Color based on density (blue to red heatmap)
      const [r, g, b] = densityToColor(normalizedDensity);
      colors.push(r, g, b);

      // Generate triangle indices
      if (i < resolution && j < resolution) {
        const topLeft = i * (resolution + 1) + j;
        const topRight = topLeft + 1;
        const bottomLeft = (i + 1) * (resolution + 1) + j;
        const bottomRight = bottomLeft + 1;

        indices.push(topLeft, bottomLeft, topRight);
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

// Color mapping for probability density
function densityToColor(normalized: number): [number, number, number] {
  // Blue (low) -> Cyan -> Green -> Yellow -> Red (high)
  if (normalized < 0.25) {
    const t = normalized * 4;
    return [0, t, 1]; // Blue to Cyan
  } else if (normalized < 0.5) {
    const t = (normalized - 0.25) * 4;
    return [0, 1, 1 - t]; // Cyan to Green
  } else if (normalized < 0.75) {
    const t = (normalized - 0.5) * 4;
    return [t, 1, 0]; // Green to Yellow
  } else {
    const t = (normalized - 0.75) * 4;
    return [1, 1 - t, 0]; // Yellow to Red
  }
}

// BVN Density Surface Component
function BVNSurface({ sigmaX, sigmaY }: { sigmaX: number; sigmaY: number }) {
  const meshRef = useRef<THREE.Mesh>(null);

  const { vertices, colors, indices } = useMemo(
    () => generateBVNSurface(sigmaX, sigmaY),
    [sigmaX, sigmaY]
  );

  return (
    <mesh ref={meshRef}>
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
      <meshStandardMaterial vertexColors side={THREE.DoubleSide} transparent opacity={0.8} />
    </mesh>
  );
}

// Scatter plot of actual shots projected onto the ground plane
function ShotScatterPlot({ shots, sigmaX, sigmaY }: { shots: Shot[]; sigmaX: number; sigmaY: number }) {
  const range = 3;
  const maxX = sigmaX * range;
  const maxY = sigmaY * range;

  return (
    <group>
      {shots.map((shot, idx) => {
        // Calculate x, y position from distance and angle
        const x = shot.distance * Math.cos(shot.angle);
        const y = shot.distance * Math.sin(shot.angle);

        // Scale to 3D space
        const xScaled = (x / maxX) * 5;
        const zScaled = (y / maxY) * 5;

        // Clamp to visible range
        if (Math.abs(xScaled) > 5 || Math.abs(zScaled) > 5) return null;

        // Color based on profit/loss
        const color = shot.profit >= 0 ? '#10B981' : '#EF4444';

        return (
          <group key={idx}>
            {/* Dot on ground plane */}
            <mesh position={[xScaled, 0, zScaled]}>
              <sphereGeometry args={[0.08, 8, 8]} />
              <meshStandardMaterial color={color} emissive={color} emissiveIntensity={0.5} />
            </mesh>
            {/* Vertical line from ground to density surface */}
            <mesh position={[xScaled, 0.5, zScaled]}>
              <cylinderGeometry args={[0.02, 0.02, 1, 8]} />
              <meshStandardMaterial color={color} transparent opacity={0.3} />
            </mesh>
          </group>
        );
      })}
    </group>
  );
}

// Contour circles on ground plane showing 1σ, 2σ, 3σ
function GroundContours({ sigmaX }: { sigmaX: number; sigmaY: number }) {
  const range = 3;
  const maxX = sigmaX * range;

  const sigmaLevels = [
    { sigma: 1, color: '#604c9c', opacity: 0.6 },
    { sigma: 2, color: '#493b7c', opacity: 0.4 },
    { sigma: 3, color: '#3a2f5f', opacity: 0.3 },
  ];

  return (
    <group>
      {sigmaLevels.map(({ sigma, color, opacity }) => {
        const radius = (sigmaX * sigma / maxX) * 5;

        return (
          <mesh key={sigma} position={[0, 0.01, 0]} rotation={[-Math.PI / 2, 0, 0]}>
            <ringGeometry args={[radius - 0.05, radius + 0.05, 64]} />
            <meshBasicMaterial color={color} transparent opacity={opacity} />
          </mesh>
        );
      })}
    </group>
  );
}

// Axis labels for the 3D view
function AxisLabels({ sigmaX, sigmaY }: { sigmaX: number; sigmaY: number }) {
  return (
    <>
      {/* X-axis label */}
      <Text
        position={[6, 0, 0]}
        fontSize={0.4}
        color="#dfc9ad"
        anchorX="center"
        anchorY="middle"
      >
        X (yards)
      </Text>

      {/* Y-axis label (height = probability density) */}
      <Text
        position={[0, 4, 0]}
        fontSize={0.4}
        color="#dfc9ad"
        anchorX="center"
        anchorY="middle"
      >
        Probability Density
      </Text>

      {/* Z-axis label */}
      <Text
        position={[0, 0, 6]}
        fontSize={0.4}
        color="#dfc9ad"
        anchorX="center"
        anchorY="middle"
      >
        Y (yards)
      </Text>

      {/* Sigma annotations */}
      <Text
        position={[0, 3.5, 0]}
        fontSize={0.3}
        color="#9e8cb4"
        anchorX="center"
        anchorY="middle"
      >
        σ_x = {sigmaX.toFixed(1)}y, σ_y = {sigmaY.toFixed(1)}y
      </Text>
    </>
  );
}

// Center marker (target hole)
function CenterMarker() {
  const markerRef = useRef<THREE.Mesh>(null);

  useFrame((state) => {
    if (markerRef.current) {
      markerRef.current.position.y = 0.2 + Math.sin(state.clock.elapsedTime * 2) * 0.1;
    }
  });

  return (
    <mesh ref={markerRef} position={[0, 0.2, 0]}>
      <cylinderGeometry args={[0.1, 0.05, 0.4, 16]} />
      <meshStandardMaterial color="#D4AF37" emissive="#D4AF37" emissiveIntensity={0.8} />
    </mesh>
  );
}

export default function BVNHeatmap3D({
  sigmaX,
  sigmaY,
  shots,
}: BVNHeatmap3DProps) {
  return (
    <div className="w-full h-full flex flex-col">
      <div className="bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/30 shadow-lg flex-1 flex flex-col">
        <h3 className="text-xs font-medium text-[#9e8cb4] mb-2">
          3D Distribution
        </h3>

        <div className="flex-1 min-h-0">
          <Canvas camera={{ position: [8, 5, 8], fov: 50 }}>
            {/* Lighting */}
            <ambientLight intensity={0.6} />
            <directionalLight position={[10, 10, 5]} intensity={0.8} />
            <pointLight position={[-10, 5, -5]} intensity={0.4} />

            {/* BVN Probability Density Surface */}
            <BVNSurface sigmaX={sigmaX} sigmaY={sigmaY} />

            {/* Ground contours (1σ, 2σ, 3σ circles) */}
            <GroundContours sigmaX={sigmaX} sigmaY={sigmaY} />

            {/* Shot scatter plot on ground */}
            <ShotScatterPlot shots={shots} sigmaX={sigmaX} sigmaY={sigmaY} />

            {/* Center marker (target hole) */}
            <CenterMarker />

            {/* Grid on base plane */}
            <Grid
              args={[10, 10]}
              cellSize={0.5}
              cellThickness={0.5}
              cellColor="#2a2a4a"
              sectionSize={2.5}
              sectionThickness={1}
              sectionColor="#3a3a6a"
              fadeDistance={30}
              fadeStrength={1}
              followCamera={false}
              position={[0, 0, 0]}
            />

            {/* Axis labels */}
            <AxisLabels sigmaX={sigmaX} sigmaY={sigmaY} />

            {/* Interactive controls */}
            <OrbitControls
              enablePan={true}
              enableZoom={true}
              enableRotate={true}
              minDistance={6}
              maxDistance={20}
            />
          </Canvas>
        </div>

        {/* Legend */}
        <div className="mt-2 flex flex-wrap justify-center gap-2 text-[10px]">
          <div className="flex items-center gap-1">
            <div className="w-2 h-2 bg-blue-500 rounded-sm"></div>
            <span className="text-[#9e8cb4]/70">Low</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-2 h-2 bg-yellow-500 rounded-sm"></div>
            <span className="text-[#9e8cb4]/70">Med</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-2 h-2 bg-red-500 rounded-sm"></div>
            <span className="text-[#9e8cb4]/70">High</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-2 h-2 rounded-full bg-green-500"></div>
            <span className="text-[#9e8cb4]/70">Win</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-2 h-2 rounded-full bg-red-500"></div>
            <span className="text-[#9e8cb4]/70">Loss</span>
          </div>
        </div>

        {/* Info text */}
        <div className="mt-1.5 text-[10px] text-[#9e8cb4]/50 text-center">
          <p>Drag to rotate • Scroll to zoom • Right-click to pan</p>
        </div>
      </div>
    </div>
  );
}
