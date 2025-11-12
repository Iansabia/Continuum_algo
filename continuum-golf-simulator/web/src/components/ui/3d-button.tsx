import React from 'react'
import { motion } from 'framer-motion'

interface Button3DProps {
  children: React.ReactNode
  onClick?: () => void
  href?: string
  variant?: 'primary' | 'secondary'
  className?: string
}

export const Button3D: React.FC<Button3DProps> = ({
  children,
  onClick,
  href,
  variant = 'primary',
  className = '',
}) => {
  const isPrimary = variant === 'primary'

  const baseStyle = {
    position: 'relative' as const,
    padding: '16px 40px',
    fontSize: '15.5px',
    fontWeight: 680,
    letterSpacing: '0.5px',
    border: 'none',
    cursor: 'pointer',
    textDecoration: 'none',
    display: 'inline-block',
    borderRadius: '12px',
    fontFamily: 'Inter, -apple-system, BlinkMacSystemFont, "SF Pro Display", sans-serif',
    WebkitFontSmoothing: 'antialiased' as const,
    MozOsxFontSmoothing: 'grayscale' as const,
    transition: 'all 0.2s ease',
  }

  const primaryStyle = {
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
    color: '#1a1a1a',
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
    textShadow: `
      0 1px 0 rgba(0, 0, 0, 0.35),
      0 -1px 0 rgba(255, 255, 255, 0.8),
      1px 1px 0 rgba(0, 0, 0, 0.18),
      -1px 1px 0 rgba(0, 0, 0, 0.15)
    `,
  }

  const secondaryStyle = {
    background: `
      linear-gradient(135deg,
        rgba(252, 252, 253, 0.4) 0%,
        rgba(248, 248, 250, 0.4) 50%,
        rgba(244, 244, 246, 0.4) 100%
      )
    `,
    color: '#1a1a1a',
    border: '2px solid rgba(255, 255, 255, 0.3)',
    backdropFilter: 'blur(10px)',
    boxShadow: `
      0 2px 4px rgba(0, 0, 0, 0.08),
      0 4px 8px rgba(0, 0, 0, 0.06),
      inset 0 1px 1px rgba(255, 255, 255, 0.6),
      inset 0 -1px 3px rgba(0, 0, 0, 0.06)
    `,
    textShadow: `
      0 1px 0 rgba(0, 0, 0, 0.25),
      0 -1px 0 rgba(255, 255, 255, 0.7)
    `,
  }

  const hoverStyle = isPrimary
    ? {
        transform: 'translateY(-2px)',
        boxShadow: `
          0 4px 8px rgba(0, 0, 0, 0.14),
          0 10px 20px rgba(0, 0, 0, 0.12),
          0 20px 40px rgba(0, 0, 0, 0.10),
          inset 0 2px 2px rgba(255, 255, 255, 0.8),
          inset 0 -3px 8px rgba(0, 0, 0, 0.12)
        `,
      }
    : {
        transform: 'translateY(-2px)',
        background: `
          linear-gradient(135deg,
            rgba(252, 252, 253, 0.6) 0%,
            rgba(248, 248, 250, 0.6) 50%,
            rgba(244, 244, 246, 0.6) 100%
          )
        `,
        borderColor: 'rgba(255, 255, 255, 0.5)',
      }

  const activeStyle = {
    transform: 'translateY(1px)',
    boxShadow: isPrimary
      ? `
        0 1px 2px rgba(0, 0, 0, 0.10),
        0 2px 4px rgba(0, 0, 0, 0.08),
        inset 0 2px 4px rgba(0, 0, 0, 0.12)
      `
      : `
        0 1px 2px rgba(0, 0, 0, 0.06),
        inset 0 1px 2px rgba(0, 0, 0, 0.08)
      `,
  }

  const combinedStyle = {
    ...baseStyle,
    ...(isPrimary ? primaryStyle : secondaryStyle),
  }

  const content = (
    <>
      {/* Top edge highlight */}
      <div
        className="absolute inset-x-0 top-0 rounded-t-xl pointer-events-none"
        style={{
          height: '2px',
          background:
            'linear-gradient(90deg, rgba(255, 255, 255, 0) 0%, rgba(255, 255, 255, 0.9) 20%, rgba(255, 255, 255, 1) 50%, rgba(255, 255, 255, 0.9) 80%, rgba(255, 255, 255, 0) 100%)',
          filter: 'blur(0.3px)',
        }}
      />

      {/* Top hemisphere light */}
      <div
        className="absolute inset-x-0 top-0 rounded-xl pointer-events-none"
        style={{
          height: '50%',
          background:
            'linear-gradient(180deg, rgba(255, 255, 255, 0.35) 0%, rgba(255, 255, 255, 0.15) 50%, rgba(255, 255, 255, 0) 100%)',
        }}
      />

      {/* Content */}
      <span className="relative z-10">{children}</span>
    </>
  )

  if (href) {
    return (
      <motion.a
        href={href}
        className={className}
        style={combinedStyle}
        whileHover={hoverStyle}
        whileTap={activeStyle}
      >
        {content}
      </motion.a>
    )
  }

  return (
    <motion.button
      onClick={onClick}
      className={className}
      style={combinedStyle}
      whileHover={hoverStyle}
      whileTap={activeStyle}
    >
      {content}
    </motion.button>
  )
}
