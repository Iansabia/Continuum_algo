import { useEffect, useRef, useState } from 'react'

export function useAdaptiveTextColor() {
  const ref = useRef<HTMLElement>(null)
  const [colorClass, setColorClass] = useState('text-brand-tan')

  useEffect(() => {
    if (!ref.current) return

    let rafId: number

    const checkBackgroundColor = () => {
      if (!ref.current) return

      const rect = ref.current.getBoundingClientRect()
      const x = rect.left + rect.width / 2
      const y = rect.top + rect.height / 2

      // Create a small canvas to sample the pixel color
      const canvas = document.createElement('canvas')
      canvas.width = 1
      canvas.height = 1
      const ctx = canvas.getContext('2d', { willReadFrequently: true })

      if (!ctx) return

      // Get all elements at this point
      const elements = document.elementsFromPoint(x, y)

      // Find the gradient background element (skip the text element itself)
      for (const element of elements) {
        if (element === ref.current) continue

        const computedStyle = window.getComputedStyle(element)
        const bgColor = computedStyle.backgroundColor

        // Skip transparent backgrounds
        if (bgColor === 'rgba(0, 0, 0, 0)' || bgColor === 'transparent') continue

        // Parse the RGB values
        const rgbMatch = bgColor.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/)
        if (!rgbMatch) continue

        const r = parseInt(rgbMatch[1])
        const g = parseInt(rgbMatch[2])
        const b = parseInt(rgbMatch[3])

        // Calculate if the color is more purple-ish or tan-ish
        // Purple: high blue component, lower red/green
        // Tan: high red/green, lower blue
        const isPurplish = b > Math.max(r, g) * 0.8
        const isTannish = r > b * 1.2 && g > b * 1.2

        // Calculate luminance
        const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255

        // If background is purple or dark, use tan text
        // If background is tan or light, use purple text
        if (isPurplish || luminance < 0.4) {
          setColorClass('text-brand-tan')
        } else if (isTannish || luminance > 0.55) {
          setColorClass('text-brand-bright-purple')
        }

        break // Use the first non-transparent background
      }

      rafId = requestAnimationFrame(checkBackgroundColor)
    }

    rafId = requestAnimationFrame(checkBackgroundColor)

    return () => {
      if (rafId) cancelAnimationFrame(rafId)
    }
  }, [])

  return { ref, colorClass }
}
