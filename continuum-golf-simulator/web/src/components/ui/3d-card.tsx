import React from 'react'
import { motion } from 'framer-motion'

interface Card3DProps {
  children: React.ReactNode
  className?: string
  icon?: string
}

export const Card3D: React.FC<Card3DProps> = ({ children, className = '', icon }) => {
  return (
    <motion.div
      className={`relative rounded-3xl p-10 ${className}`}
      style={{
        background: `
          linear-gradient(135deg,
            #fcfcfd 0%,
            #f8f8fa 15%,
            #f3f4f6 30%,
            #eeeff2 45%,
            #e9eaed 60%,
            #e4e5e8 75%,
            #dee0e3 90%,
            #e2e3e6 100%
          )
        `,
        boxShadow: `
          0 3px 6px rgba(0, 0, 0, 0.12),
          0 8px 16px rgba(0, 0, 0, 0.10),
          0 16px 32px rgba(0, 0, 0, 0.08),
          0 1px 2px rgba(0, 0, 0, 0.12),
          inset 0 2px 1px rgba(255, 255, 255, 0.7),
          inset 0 -2px 6px rgba(0, 0, 0, 0.10),
          inset 2px 2px 8px rgba(0, 0, 0, 0.08),
          inset -2px 2px 8px rgba(0, 0, 0, 0.07),
          inset 0 0 1px rgba(0, 0, 0, 0.15)
        `,
        transition: 'all 0.3s ease',
      }}
      whileHover={{
        y: -8,
        boxShadow: `
          0 4px 8px rgba(0, 0, 0, 0.14),
          0 10px 20px rgba(0, 0, 0, 0.12),
          0 20px 40px rgba(0, 0, 0, 0.10),
          0 30px 60px rgba(0, 0, 0, 0.08),
          inset 0 2px 2px rgba(255, 255, 255, 0.8),
          inset 0 -3px 8px rgba(0, 0, 0, 0.12),
          inset 3px 3px 8px rgba(0, 0, 0, 0.10),
          inset -3px 3px 8px rgba(0, 0, 0, 0.09)
        `,
      }}
    >
      {/* Primary top edge ridge */}
      <div
        className="absolute inset-x-0 top-0 rounded-t-3xl pointer-events-none"
        style={{
          height: '2px',
          background:
            'linear-gradient(90deg, rgba(255, 255, 255, 0) 0%, rgba(255, 255, 255, 0.95) 5%, rgba(255, 255, 255, 1) 15%, rgba(255, 255, 255, 1) 85%, rgba(255, 255, 255, 0.95) 95%, rgba(255, 255, 255, 0) 100%)',
          filter: 'blur(0.3px)',
        }}
      />

      {/* Top hemisphere light catch */}
      <div
        className="absolute inset-x-0 top-0 rounded-3xl pointer-events-none"
        style={{
          height: '55%',
          background:
            'linear-gradient(180deg, rgba(255, 255, 255, 0.45) 0%, rgba(255, 255, 255, 0.25) 30%, rgba(255, 255, 255, 0.10) 60%, rgba(255, 255, 255, 0) 100%)',
        }}
      />

      {/* Directional light - top left */}
      <div
        className="absolute inset-0 rounded-3xl pointer-events-none"
        style={{
          background:
            'linear-gradient(135deg, rgba(255, 255, 255, 0.40) 0%, rgba(255, 255, 255, 0.20) 20%, rgba(255, 255, 255, 0.08) 40%, rgba(255, 255, 255, 0) 65%)',
        }}
      />

      {/* Premium gloss reflection */}
      <div
        className="absolute rounded-full pointer-events-none"
        style={{
          left: '15%',
          top: '12%',
          width: '140px',
          height: '18px',
          background:
            'radial-gradient(ellipse at center, rgba(255, 255, 255, 0.70) 0%, rgba(255, 255, 255, 0.35) 40%, rgba(255, 255, 255, 0.10) 70%, rgba(255, 255, 255, 0) 100%)',
          filter: 'blur(5px)',
          transform: 'rotate(-8deg)',
        }}
      />

      {/* Bottom curvature shadow */}
      <div
        className="absolute inset-x-0 bottom-0 rounded-b-3xl pointer-events-none"
        style={{
          height: '50%',
          background:
            'linear-gradient(0deg, rgba(0, 0, 0, 0.14) 0%, rgba(0, 0, 0, 0.08) 25%, rgba(0, 0, 0, 0.03) 50%, rgba(0, 0, 0, 0) 100%)',
        }}
      />

      {/* Bottom edge contact shadow */}
      <div
        className="absolute inset-x-0 bottom-0 rounded-b-3xl pointer-events-none"
        style={{
          height: '20%',
          background: 'linear-gradient(0deg, rgba(0, 0, 0, 0.20) 0%, rgba(0, 0, 0, 0) 100%)',
          filter: 'blur(3px)',
        }}
      />

      {/* Inner diffuse glow */}
      <div
        className="absolute inset-0 rounded-3xl pointer-events-none"
        style={{
          boxShadow: 'inset 0 0 40px rgba(255, 255, 255, 0.22)',
          opacity: 0.7,
        }}
      />

      {/* Micro edge definition */}
      <div
        className="absolute inset-0 rounded-3xl pointer-events-none"
        style={{
          boxShadow: 'inset 0 0 0 0.5px rgba(0, 0, 0, 0.10)',
        }}
      />

      {/* Icon badge if provided */}
      {icon && (
        <div
          className="relative z-10 w-16 h-16 rounded-2xl flex items-center justify-center mb-6 text-3xl"
          style={{
            background: `
              linear-gradient(135deg,
                rgba(252, 252, 253, 0.9) 0%,
                rgba(238, 239, 242, 0.9) 100%
              )
            `,
            boxShadow: `
              0 2px 4px rgba(0, 0, 0, 0.08),
              0 4px 8px rgba(0, 0, 0, 0.06),
              inset 0 1px 1px rgba(255, 255, 255, 0.8),
              inset 0 -1px 3px rgba(0, 0, 0, 0.08)
            `,
          }}
        >
          {icon}
        </div>
      )}

      {/* Content */}
      <div className="relative z-10">{children}</div>
    </motion.div>
  )
}
