import { useAdaptiveTextColor } from '@/hooks/useAdaptiveTextColor'

interface AdaptiveTextProps {
  children: React.ReactNode
  className?: string
  as?: 'h1' | 'h2' | 'h3' | 'p' | 'div' | 'span'
}

export function AdaptiveText({ children, className = '', as = 'div' }: AdaptiveTextProps) {
  const { ref, colorClass } = useAdaptiveTextColor()
  const Component = as

  return (
    <Component
      ref={ref as any}
      className={`transition-colors duration-150 ${colorClass} ${className}`}
    >
      {children}
    </Component>
  )
}
