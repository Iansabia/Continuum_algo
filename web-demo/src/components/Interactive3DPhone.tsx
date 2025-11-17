import { SplineScene } from '@/components/ui/splite'

export function Interactive3DPhone() {
  return (
    <div className="relative w-full h-[700px] flex items-center justify-center">
      {/* Spline 3D iPhone Scene - Using a proper iPhone model */}
      <SplineScene
        scene="https://prod.spline.design/bbZoK7d2K5OjTJjP/scene.splinecode"
        className="w-full h-full"
      />
    </div>
  )
}
