import { useRef, useMemo } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { OrbitControls, Text, Grid, Line as DreiLine } from '@react-three/drei';
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
}

// ============================================================================
// BIVARIATE NORMAL DISTRIBUTION & KERNEL DENSITY ESTIMATION
// ============================================================================

/**
 * Calculate the Bivariate Normal Distribution (BVN) PDF
 *
 * Formula: P(x,y) = 1/(2π·σx·σy·√(1-ρ²)) · exp(-z/(2(1-ρ²)))
 * where: z = (x-μx)²/σx² - 2ρ(x-μx)(y-μy)/(σx·σy) + (y-μy)²/σy²
 *
 * @param x - X coordinate
 * @param y - Y coordinate
 * @param muX - Mean in X direction (default: 0)
 * @param muY - Mean in Y direction (default: 0)
 * @param sigmaX - Standard deviation in X direction
 * @param sigmaY - Standard deviation in Y direction
 * @param rho - Correlation coefficient (default: 0 for circular)
 * @returns Probability density at (x, y)
 */
// @ts-ignore - Reserved for future use
// eslint-disable-next-line @typescript-eslint/no-unused-vars
function bivariateNormalPDF(
  x: number,
  y: number,
  muX: number = 0,
  muY: number = 0,
  sigmaX: number = 1,
  sigmaY: number = 1,
  rho: number = 0
): number {
  const xNorm = (x - muX) / sigmaX;
  const yNorm = (y - muY) / sigmaY;

  const rhoSquared = rho * rho;
  const denominator = 2 * Math.PI * sigmaX * sigmaY * Math.sqrt(1 - rhoSquared);

  const z = (xNorm * xNorm - 2 * rho * xNorm * yNorm + yNorm * yNorm) / (2 * (1 - rhoSquared));

  return Math.exp(-z) / denominator;
}

/**
 * Calculate kernel density estimation from actual shot coordinates
 * Uses Gaussian kernel to estimate probability density at a grid point
 *
 * FIXED: Proper normalization prevents "pillow effect" with many shots
 * - Normalizes by kernel area (2π * bandwidth²)
 * - Uses tighter bandwidth for sharper peaks
 */
function calculateDensityFromShots(
  shots: Shot[],
  gridX: number,
  gridY: number,
  bandwidth: number = 2.0
): number {
  if (shots.length === 0) return 0;

  let density = 0;

  shots.forEach(shot => {
    const shotX = shot.distance * Math.cos(shot.angle);
    const shotY = shot.distance * Math.sin(shot.angle);

    // Gaussian kernel
    const dx = gridX - shotX;
    const dy = gridY - shotY;
    const distSquared = (dx * dx + dy * dy) / (bandwidth * bandwidth);
    density += Math.exp(-0.5 * distSquared);
  });

  // Proper KDE normalization: divide by (n * 2π * h²)
  // This prevents density from growing linearly with number of shots
  const normalizationFactor = shots.length * 2 * Math.PI * bandwidth * bandwidth;
  return density / normalizationFactor;
}

// ============================================================================
// 3D SURFACE GENERATION
// ============================================================================

/**
 * Generate density surface mesh from actual shot coordinates
 * Creates a triangulated 3D surface where height represents probability density
 *
 * @param shots - Array of shot data with distance and angle
 * @param sigmaX - Standard deviation in X direction
 * @param sigmaY - Standard deviation in Y direction
 * @returns Vertices, colors, and indices for Three.js BufferGeometry
 */
function generateDensitySurfaceFromShots(
  shots: Shot[],
  sigmaX: number,
  sigmaY: number
): {
  vertices: Float32Array;
  colors: Float32Array;
  indices: Uint16Array;
} {
  const resolution = 60; // Higher resolution for smoother surface with visible triangular mesh
  // ADAPTIVE RANGE: Scale view to 3σ (covers ~99.7% of distribution)
  // This prevents "pillow effect" when sigma is large
  const maxSigma = Math.max(sigmaX, sigmaY);
  const rangeYards = Math.max(15, maxSigma * 3); // Minimum 15 yards, or 3× max sigma

  const vertices: number[] = [];
  const colors: number[] = [];
  const indices: number[] = [];

  // Only generate surface if there are shots
  if (shots.length > 0) {
    // Calculate densities from actual shots
    const densities: number[] = [];
    let maxDensity = 0;

    // Calculate bandwidth based on shot dispersion
    // FIXED: Reduced bandwidth for sharper, more localized peaks
    const bandwidth = Math.max(1.5, Math.sqrt(sigmaX * sigmaX + sigmaY * sigmaY) * 0.3);

    for (let i = 0; i <= resolution; i++) {
      for (let j = 0; j <= resolution; j++) {
        const x = -rangeYards + (2 * rangeYards * i) / resolution;
        const y = -rangeYards + (2 * rangeYards * j) / resolution;

        const density = calculateDensityFromShots(shots, x, y, bandwidth);
        densities.push(density);
        maxDensity = Math.max(maxDensity, density);
      }
    }

    // Generate vertices with heights based on density
    let densityIdx = 0;
    for (let i = 0; i <= resolution; i++) {
      for (let j = 0; j <= resolution; j++) {
        const x = -rangeYards + (2 * rangeYards * i) / resolution;
        const y = -rangeYards + (2 * rangeYards * j) / resolution;

        const density = densities[densityIdx];
        const normalizedDensity = maxDensity > 0 ? density / maxDensity : 0;

        const xScaled = (x / rangeYards) * 5;
        const zScaled = (y / rangeYards) * 5;
        // Apply exponential scaling to emphasize peaks and reduce "puffiness"
        // FIXED: Stronger power function creates sharper peaks with near-zero base
        const yScaled = Math.pow(normalizedDensity, 1.5) * 3.0; // Much sharper peaks

        vertices.push(xScaled, yScaled, zScaled);

        // Color based on density
        const [r, g, b] = densityToColor(normalizedDensity);
        colors.push(r, g, b);

        if (i < resolution && j < resolution) {
          const topLeft = i * (resolution + 1) + j;
          const topRight = topLeft + 1;
          const bottomLeft = (i + 1) * (resolution + 1) + j;
          const bottomRight = bottomLeft + 1;

          indices.push(topLeft, bottomLeft, topRight);
          indices.push(topRight, bottomLeft, bottomRight);
        }

        densityIdx++;
      }
    }
  }

  return {
    vertices: new Float32Array(vertices),
    colors: new Float32Array(colors),
    indices: new Uint16Array(indices),
  };
}

/**
 * Maps normalized density values to colors
 * Uses reversed gradient: White (low) → Bright Purple (high)
 *
 * @param normalized - Density value normalized to [0, 1]
 * @returns RGB color tuple [r, g, b] where each component is in [0, 1]
 */
function densityToColor(normalized: number): [number, number, number] {
  // White (255, 255, 255) -> Bright Purple #604c9c (96, 76, 156)
  // Linear interpolation from white to bright purple

  const t = normalized;
  return [
    (255 + t * (96 - 255)) / 255,
    (255 + t * (76 - 255)) / 255,
    (255 + t * (156 - 255)) / 255
  ];
}

// ============================================================================
// REACT THREE FIBER COMPONENTS
// ============================================================================

/**
 * 3D Density Surface Component
 * Renders the triangulated mesh that visualizes probability density
 * Smoothly animates when new shots are added
 *
 * Key features for triangular mesh visualization:
 * - Uses THREE.BufferGeometry with explicit vertex/face structure
 * - Dual rendering: solid colored surface + wireframe overlay
 * - FlatShading emphasizes individual triangular faces
 */
function DensitySurface({
  shots,
  sigmaX,
  sigmaY,
}: {
  shots: Shot[];
  sigmaX: number;
  sigmaY: number;
}) {
  // Regenerate surface data whenever shots change
  const { vertices, colors, indices } = useMemo(
    () => generateDensitySurfaceFromShots(shots, sigmaX, sigmaY),
    [shots, sigmaX, sigmaY]
  );

  // No animation - just use key to force re-render when shots change

  // Only render if there are vertices to display
  if (vertices.length === 0) {
    return null;
  }

  return (
    <group key={`surface-${shots.length}`}>
      {/* Main colored surface with flat shading to show triangular faces */}
      <mesh key={`mesh-${shots.length}`}>
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
        <meshStandardMaterial
          vertexColors
          side={THREE.DoubleSide}
          flatShading={true} // Enable flat shading for visible triangle faces
          polygonOffset={true}
          polygonOffsetFactor={1}
          polygonOffsetUnits={1}
        />
      </mesh>

      {/* Wireframe overlay to emphasize triangular mesh structure */}
      <lineSegments key={`wireframe-${shots.length}`}>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            count={vertices.length / 3}
            array={vertices}
            itemSize={3}
          />
          <bufferAttribute attach="index" count={indices.length} array={indices} itemSize={1} />
        </bufferGeometry>
        <lineBasicMaterial
          color="#493b7c"
          transparent
          opacity={0.3}
          linewidth={1}
        />
      </lineSegments>
    </group>
  );
}

/**
 * Scatter Plot Component
 * Displays actual shot positions on the ground plane with vertical lines
 * Colors indicate profit (green) or loss (red)
 */
function ShotScatterPlot({ shots, sigmaX, sigmaY }: { shots: Shot[]; sigmaX: number; sigmaY: number }) {
  // FIXED: Use same range calculation as density surface to match alignment
  const maxSigma = Math.max(sigmaX, sigmaY);
  const rangeYards = Math.max(15, maxSigma * 3);

  return (
    <group>
      {shots.map((shot, idx) => {
        // Calculate x, y position from distance and angle
        const x = shot.distance * Math.cos(shot.angle);
        const y = shot.distance * Math.sin(shot.angle);

        // Scale to 3D space - use same scaling as density surface
        const xScaled = (x / rangeYards) * 5;
        const zScaled = (y / rangeYards) * 5;

        // Clamp to visible range
        if (Math.abs(xScaled) > 5 || Math.abs(zScaled) > 5) return null;

        // Color based on profit/loss
        const color = shot.profit >= 0 ? '#10B981' : '#EF4444';

        return (
          <group key={idx}>
            {/* Dot on ground plane */}
            <mesh position={[xScaled, 0.05, zScaled]}>
              <sphereGeometry args={[0.12, 16, 16]} />
              <meshStandardMaterial color={color} emissive={color} emissiveIntensity={0.7} />
            </mesh>
          </group>
        );
      })}
    </group>
  );
}

/**
 * Ground Contours Component
 * Displays concentric circles at 1σ, 2σ, and 3σ levels on the ground plane
 * Provides visual reference for standard deviations
 */
function GroundContours({ sigmaX, sigmaY }: { sigmaX: number; sigmaY: number }) {
  // FIXED: Use same range calculation as density surface to match alignment
  const maxSigma = Math.max(sigmaX, sigmaY);
  const rangeYards = Math.max(15, maxSigma * 3);

  const sigmaLevels = [
    { sigma: 1, color: '#604c9c', opacity: 0.6 },
    { sigma: 2, color: '#493b7c', opacity: 0.4 },
    { sigma: 3, color: '#3a2f5f', opacity: 0.3 },
  ];

  return (
    <group>
      {sigmaLevels.map(({ sigma, color, opacity }) => {
        const radius = (sigmaX * sigma / rangeYards) * 5;

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

/**
 * Marginal Distribution Curves Component
 * Displays P(X) and P(Y) on vertical walls by numerically integrating the 3D density surface
 * - P(X): Integral of density over all Y values for each X
 * - P(Y): Integral of density over all X values for each Y
 */
function MarginalCurves({ shots, sigmaX, sigmaY }: { shots: Shot[]; sigmaX: number; sigmaY: number }) {
  // ADAPTIVE RANGE: Match the surface range
  const maxSigma = Math.max(sigmaX, sigmaY);
  const rangeYards = Math.max(15, maxSigma * 3);

  // Generate marginal P(X) by integrating density over Y
  const marginalXPoints = useMemo(() => {
    const points: THREE.Vector3[] = [];
    const samples = 100;

    if (shots.length === 0) {
      // Return empty when no shots
      return points;
    } else {
      const bandwidth = Math.max(1.5, Math.sqrt(sigmaX * sigmaX + sigmaY * sigmaY) * 0.3);
      const marginalDensities: number[] = [];
      let maxDensity = 0;

      // For each X position, integrate density over all Y values
      for (let i = 0; i <= samples; i++) {
        const x = -rangeYards + (2 * rangeYards * i) / samples;
        let densitySum = 0;

        // Integrate over Y axis (sum density at this X for all Y)
        const ySamples = 50;
        for (let j = 0; j <= ySamples; j++) {
          const y = -rangeYards + (2 * rangeYards * j) / ySamples;
          densitySum += calculateDensityFromShots(shots, x, y, bandwidth);
        }

        marginalDensities.push(densitySum);
        maxDensity = Math.max(maxDensity, densitySum);
      }

      // Generate curve from integrated densities
      for (let i = 0; i <= samples; i++) {
        const x = -rangeYards + (2 * rangeYards * i) / samples;
        const xScaled = (x / rangeYards) * 5;
        const normalized = maxDensity > 0 ? marginalDensities[i] / maxDensity : 0;
        const yScaled = Math.pow(normalized, 1.5) * 3.0; // Match surface scaling
        points.push(new THREE.Vector3(xScaled, yScaled, -5));
      }
    }
    return points;
  }, [shots, sigmaX, sigmaY]);

  // Generate marginal P(Y) by integrating density over X
  const marginalYPoints = useMemo(() => {
    const points: THREE.Vector3[] = [];
    const samples = 100;

    if (shots.length === 0) {
      // Return empty when no shots
      return points;
    } else {
      const bandwidth = Math.max(1.5, Math.sqrt(sigmaX * sigmaX + sigmaY * sigmaY) * 0.3);
      const marginalDensities: number[] = [];
      let maxDensity = 0;

      // For each Y position, integrate density over all X values
      for (let i = 0; i <= samples; i++) {
        const y = -rangeYards + (2 * rangeYards * i) / samples;
        let densitySum = 0;

        // Integrate over X axis (sum density at this Y for all X)
        const xSamples = 50;
        for (let j = 0; j <= xSamples; j++) {
          const x = -rangeYards + (2 * rangeYards * j) / xSamples;
          densitySum += calculateDensityFromShots(shots, x, y, bandwidth);
        }

        marginalDensities.push(densitySum);
        maxDensity = Math.max(maxDensity, densitySum);
      }

      // Generate curve from integrated densities
      for (let i = 0; i <= samples; i++) {
        const y = -rangeYards + (2 * rangeYards * i) / samples;
        const zScaled = (y / rangeYards) * 5;
        const normalized = maxDensity > 0 ? marginalDensities[i] / maxDensity : 0;
        const yScaled = Math.pow(normalized, 1.5) * 3.0; // Match surface scaling
        points.push(new THREE.Vector3(5, yScaled, zScaled));
      }
    }
    return points;
  }, [shots, sigmaX, sigmaY]);

  // Only render if there are points
  if (marginalXPoints.length === 0 || marginalYPoints.length === 0) {
    return null;
  }

  return (
    <group>
      {/* Marginal P(X) on back wall (XZ-plane) - gradient from white to purple */}
      <DreiLine
        points={marginalXPoints}
        color="#604c9c"
        lineWidth={3}
      />

      {/* Marginal P(Y) on side wall (YZ-plane) - gradient from white to purple */}
      <DreiLine
        points={marginalYPoints}
        color="#604c9c"
        lineWidth={3}
      />
    </group>
  );
}

// ============================================================================
// HELPER COMPONENTS
// ============================================================================

/**
 * Axis Labels Component
 * Displays X, Y, Z axis labels and sigma annotations in 3D space
 */
function AxisLabels({ sigmaX, sigmaY, shotCount }: { sigmaX: number; sigmaY: number; shotCount: number }) {
  return (
    <>
      {/* 3D X-axis (right wall) = Golf Y (distance, forward-backward) */}
      <Text
        position={[6, 0, 0]}
        fontSize={0.4}
        color="#dfc9ad"
        anchorX="center"
        anchorY="middle"
      >
        Y (distance)
      </Text>

      {/* 3D Y-axis = Probability density (height) */}
      <Text
        position={[0, 4, 0]}
        fontSize={0.4}
        color="#dfc9ad"
        anchorX="center"
        anchorY="middle"
      >
        Probability Density
      </Text>

      {/* 3D Z-axis (back wall) = Golf X (lateral, left-right) */}
      <Text
        position={[0, 0, 6]}
        fontSize={0.4}
        color="#dfc9ad"
        anchorX="center"
        anchorY="middle"
      >
        X (lateral)
      </Text>

      {/* Sigma annotations - only show when we have enough shots for meaningful statistics */}
      {shotCount >= 2 && (
        <Text
          position={[0, 3.5, 0]}
          fontSize={0.3}
          color="#9e8cb4"
          anchorX="center"
          anchorY="middle"
        >
          σ_x = {sigmaX.toFixed(1)}y, σ_y = {sigmaY.toFixed(1)}y
        </Text>
      )}
    </>
  );
}

/**
 * Center Marker Component
 * Displays an animated marker at the target hole position (origin)
 */
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

// ============================================================================
// MAIN COMPONENT
// ============================================================================

/**
 * BVNHeatmap3D - Main Component
 * 3D visualization of Bivariate Normal distribution built from actual shot data
 *
 * Features:
 * - Triangulated mesh surface showing probability density via KDE
 * - Marginal distributions P(X) and P(Y) integrated from 3D surface
 * - Shot scatter plot on ground plane
 * - Standard deviation contours (1σ, 2σ, 3σ)
 * - Interactive controls for rotation, zoom, and pan
 */
export default function BVNHeatmap3D({
  sigmaX,
  sigmaY,
  shots,
}: BVNHeatmap3DProps) {
  return (
    <div className="w-full h-full flex flex-col relative">
      <div className="flex-1 flex flex-col relative">
        <div className="flex-1 min-h-0">
          <Canvas camera={{ position: [8, 5, 8], fov: 50 }}>
            {/* Lighting - stronger for flat shading */}
            <ambientLight intensity={0.5} />
            <directionalLight position={[10, 10, 5]} intensity={1.0} />
            <directionalLight position={[-5, 5, -5]} intensity={0.4} />
            <pointLight position={[-10, 5, -5]} intensity={0.3} />

            {/* Density Surface built from actual shots */}
            <DensitySurface shots={shots} sigmaX={sigmaX} sigmaY={sigmaY} />

            {/* Ground contours (1σ, 2σ, 3σ circles) */}
            <GroundContours sigmaX={sigmaX} sigmaY={sigmaY} />

            {/* Marginal distribution curves - integrated from 3D density */}
            <MarginalCurves shots={shots} sigmaX={sigmaX} sigmaY={sigmaY} />

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
            <AxisLabels sigmaX={sigmaX} sigmaY={sigmaY} shotCount={shots.length} />

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

        {/* Legend - floating at bottom */}
        <div className="absolute bottom-2 left-1/2 transform -translate-x-1/2 flex flex-wrap justify-center gap-2 text-[10px]">
          <div className="flex items-center gap-1">
            <div className="w-2 h-2 rounded-sm" style={{ background: 'linear-gradient(to right, #ffffff, #604c9c)' }}></div>
            <span className="text-white/70">Low → High Density</span>
          </div>
          <div className="h-px w-2 bg-white/10"></div>
          <div className="flex items-center gap-1">
            <div className="w-2 h-2 rounded-full bg-[var(--brand-bright-purple)]"></div>
            <span className="text-white/70">Win</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-2 h-2 rounded-full bg-[var(--brand-rose-copper)]"></div>
            <span className="text-white/70">Loss</span>
          </div>
        </div>

        {/* Info text - floating at bottom */}
        <div className="absolute bottom-8 left-1/2 transform -translate-x-1/2 text-[10px] text-white/50 text-center">
          <p>Drag to rotate • Scroll to zoom • Right-click to pan</p>
        </div>
      </div>
    </div>
  );
}
