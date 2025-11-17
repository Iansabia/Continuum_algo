import { useEffect, useRef } from 'react';

interface FloatingLine {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  speed: number;
  offset: number;
  opacity: number;
  length: number;
}

export default function AnimatedBackground() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const floatingLinesRef = useRef<FloatingLine[]>([]);
  const animationTimeRef = useRef(0);
  const animationFrameRef = useRef<number>();

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const updateCanvasSize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;

      // Reinitialize lines on resize
      const lines: FloatingLine[] = [];
      const lineCount = Math.floor((canvas.width * canvas.height) / 30000); // Density based on screen size

      for (let i = 0; i < lineCount; i++) {
        const length = 50 + Math.random() * 150;
        const angle = Math.random() * Math.PI * 2;
        const x1 = Math.random() * canvas.width;
        const y1 = Math.random() * canvas.height;

        lines.push({
          x1,
          y1,
          x2: x1 + Math.cos(angle) * length,
          y2: y1 + Math.sin(angle) * length,
          speed: 0.2 + Math.random() * 0.5,
          offset: Math.random() * Math.PI * 2,
          opacity: 0.08 + Math.random() * 0.12,
          length,
        });
      }
      floatingLinesRef.current = lines;
    };

    updateCanvasSize();
    window.addEventListener('resize', updateCanvasSize);

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const animate = () => {
      animationTimeRef.current += 0.016; // ~60fps
      const time = animationTimeRef.current;

      // Clear canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Draw floating lines
      floatingLinesRef.current.forEach((line) => {
        // Animate line position with sine wave motion
        const offsetX = Math.sin(time * line.speed + line.offset) * 40;
        const offsetY = Math.cos(time * line.speed * 0.7 + line.offset) * 40;

        // Calculate animated opacity (pulse effect)
        const pulseOpacity = line.opacity * (0.5 + Math.sin(time * line.speed * 2 + line.offset) * 0.5);

        // Bright purple with animated opacity
        ctx.strokeStyle = `rgba(96, 76, 156, ${pulseOpacity})`;
        ctx.lineWidth = 1 + Math.sin(time * line.speed + line.offset) * 0.5;
        ctx.beginPath();
        ctx.moveTo(line.x1 + offsetX, line.y1 + offsetY);
        ctx.lineTo(line.x2 + offsetX, line.y2 + offsetY);
        ctx.stroke();

        // Add subtle glow effect for some lines
        if (pulseOpacity > 0.15) {
          ctx.strokeStyle = `rgba(158, 140, 180, ${pulseOpacity * 0.3})`; // Lavender glow
          ctx.lineWidth = 3;
          ctx.beginPath();
          ctx.moveTo(line.x1 + offsetX, line.y1 + offsetY);
          ctx.lineTo(line.x2 + offsetX, line.y2 + offsetY);
          ctx.stroke();
        }
      });

      animationFrameRef.current = requestAnimationFrame(animate);
    };

    animate();

    return () => {
      window.removeEventListener('resize', updateCanvasSize);
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      className="fixed inset-0 pointer-events-none z-0"
      style={{ opacity: 0.6 }}
    />
  );
}
