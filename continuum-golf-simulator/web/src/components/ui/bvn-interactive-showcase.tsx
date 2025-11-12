'use client'

import { Card } from "@/components/ui/card"
import { SpotlightAceternity } from "@/components/ui/spotlight-aceternity"
import { Canvas } from '@react-three/fiber'
import { OrbitControls, PerspectiveCamera } from '@react-three/drei'
import { useRef, useMemo } from 'react'
import * as THREE from 'three'

// Simplified BVN visualization for the landing page
function BVNVisualization() {
  const meshRef = useRef<THREE.Mesh>(null)

  // Create the BVN distribution surface
  const geometry = useMemo(() => {
    const size = 50
    const segments = 50
    const geo = new THREE.PlaneGeometry(size, size, segments, segments)

    const positions = geo.attributes.position
    for (let i = 0; i < positions.count; i++) {
      const x = positions.getX(i) / 10
      const y = positions.getY(i) / 10

      // Bivariate normal distribution formula
      const sigmaX = 1.5
      const sigmaY = 1.5
      const rho = 0.3

      const z1 = x / sigmaX
      const z2 = y / sigmaY
      const exp = -0.5 * (z1 * z1 + z2 * z2 - 2 * rho * z1 * z2) / (1 - rho * rho)
      const z = 8 * Math.exp(exp)

      positions.setZ(i, z)
    }

    geo.computeVertexNormals()
    return geo
  }, [])

  return (
    <>
      <PerspectiveCamera makeDefault position={[30, 25, 30]} />
      <OrbitControls
        enablePan={false}
        minDistance={20}
        maxDistance={60}
        minPolarAngle={Math.PI / 6}
        maxPolarAngle={Math.PI / 2.5}
        autoRotate
        autoRotateSpeed={1}
      />

      <ambientLight intensity={0.5} />
      <directionalLight position={[10, 10, 5]} intensity={1} />
      <pointLight position={[-10, -10, -5]} intensity={0.5} color="#604c9c" />

      <mesh ref={meshRef} geometry={geometry} rotation={[-Math.PI / 2, 0, 0]}>
        <meshStandardMaterial
          color="#604c9c"
          wireframe={false}
          transparent
          opacity={0.85}
          side={THREE.DoubleSide}
          metalness={0.3}
          roughness={0.4}
        />
      </mesh>

      {/* Grid helper */}
      <gridHelper args={[50, 20, '#493b7c', '#9e8cb4']} position={[0, 0, 0]} />
    </>
  )
}

export function BVNInteractiveShowcase() {
  return (
    <Card className="w-full min-h-[600px] md:h-[600px] bg-black/[0.96] relative overflow-hidden border-brand-lavender/20">
      <SpotlightAceternity
        className="-top-40 left-0 md:left-60 md:-top-20"
        fill="#9e8cb4"
      />

      <div className="flex flex-col md:flex-row min-h-[600px]">
        {/* Left content */}
        <div className="flex-1 p-8 relative z-10 flex flex-col justify-center">
          <h2 className="text-4xl md:text-5xl font-bold bg-clip-text text-transparent bg-gradient-to-b from-brand-tan to-brand-dark-gold mb-4">
            Interactive BVN Model
          </h2>
          <p className="text-brand-lavender text-lg mb-6 max-w-lg leading-relaxed">
            Experience our proprietary Bivariate Normal Distribution algorithm in real-time.
            This 3D visualization demonstrates how we calculate skill-based payouts with
            mathematical precision.
          </p>
          <div className="space-y-4 max-w-lg">
            <div className="flex items-start gap-3">
              <div className="w-2 h-2 rounded-full bg-brand-bright-purple mt-2"></div>
              <div>
                <h3 className="text-brand-tan font-semibold mb-1">Personalized Curves</h3>
                <p className="text-brand-lavender/80 text-sm">
                  Each player's skill profile generates a unique payout distribution
                </p>
              </div>
            </div>
            <div className="flex items-start gap-3">
              <div className="w-2 h-2 rounded-full bg-brand-bright-purple mt-2"></div>
              <div>
                <h3 className="text-brand-tan font-semibold mb-1">Real-Time Adaptation</h3>
                <p className="text-brand-lavender/80 text-sm">
                  The model updates as players improve, maintaining fairness
                </p>
              </div>
            </div>
            <div className="flex items-start gap-3">
              <div className="w-2 h-2 rounded-full bg-brand-bright-purple mt-2"></div>
              <div>
                <h3 className="text-brand-tan font-semibold mb-1">Mathematical Integrity</h3>
                <p className="text-brand-lavender/80 text-sm">
                  Proven statistical methods ensure transparent, verifiable results
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Right content - 3D visualization */}
        <div className="flex-1 relative min-h-[400px]">
          <div className="absolute inset-0 bg-gradient-to-br from-brand-deep-purple/10 to-transparent"></div>
          <Canvas className="w-full h-full">
            <BVNVisualization />
          </Canvas>
          <div className="absolute bottom-4 right-4 text-xs text-brand-lavender/60 bg-black/50 px-3 py-2 rounded backdrop-blur-sm">
            Drag to rotate • Scroll to zoom
          </div>
        </div>
      </div>
    </Card>
  )
}
