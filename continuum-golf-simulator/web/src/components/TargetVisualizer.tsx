import { useEffect, useRef, useState } from 'react';

interface Shot {
  distance: number;
  angle: number;
  wager: number;
  payout: number;
  profit: number;
  x?: number;
  y?: number;
}

interface TargetVisualizerProps {
  sigma: number;
  breakevenRadius: number;
  targetRadius: number; // Current hole's target radius in yards (d_max)
  shots: Shot[];
  currentShot?: Shot | null;
  width?: number;
  height?: number;
}

export default function TargetVisualizer({
  sigma,
  breakevenRadius,
  targetRadius,
  shots,
  currentShot,
  width = 400,
  height = 400,
}: TargetVisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [animationFrame, setAnimationFrame] = useState(0);

  const centerX = width / 2;
  const centerY = height / 2;

  // Fixed scale based ONLY on target radius to keep circles stationary
  // We show 1.3x the target radius to make the target larger on screen
  const viewportRadius = targetRadius * 1.3;

  const availableRadius = Math.min(width, height) / 2 - 20; // Canvas padding
  const SCALE = availableRadius / viewportRadius; // pixels per yard (fixed scale)

  const yardsToPixels = (yards: number) => yards * SCALE;

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Draw grid (optional, for scale reference)
    drawGrid(ctx);

    // Draw breakeven radius
    drawBreakevenRadius(ctx);

    // Draw fixed target boundary (absolute maximum)
    drawFixedTargetBoundary(ctx);

    // Draw center pin
    drawCenterPin(ctx);

    // Draw all previous shots
    shots.forEach((shot) => {
      drawShot(ctx, shot, false);
    });

    // Draw current shot with animation
    if (currentShot) {
      drawShot(ctx, currentShot, true);
    }
  }, [sigma, breakevenRadius, targetRadius, shots, currentShot, animationFrame, viewportRadius]);

  // Animate current shot
  useEffect(() => {
    if (!currentShot) return;

    let frame = 0;
    const maxFrames = 30;
    const interval = setInterval(() => {
      frame++;
      setAnimationFrame(frame);
      if (frame >= maxFrames) {
        clearInterval(interval);
      }
    }, 30);

    return () => clearInterval(interval);
  }, [currentShot]);

  const drawGrid = (ctx: CanvasRenderingContext2D) => {
    // Calculate distance from center for each grid line to fade at edges
    const maxDistance = Math.sqrt(centerX * centerX + centerY * centerY);

    // Dynamic grid spacing based on target radius
    // Smaller holes (5-10y) = 2y grid, Medium (10-20y) = 5y grid, Large (20y+) = 10y grid
    const gridSpacing = targetRadius < 10 ? 2 : targetRadius < 20 ? 5 : 10;
    const maxRange = Math.ceil(viewportRadius / gridSpacing) * gridSpacing;

    // Draw vertical lines with opacity based on distance from center
    for (let x = -maxRange; x <= maxRange; x += gridSpacing) {
      const px = centerX + yardsToPixels(x);
      if (px < 0 || px > width) continue; // Skip lines outside canvas

      const distanceFromCenter = Math.abs(px - centerX);
      const opacity = Math.max(0, 1 - (distanceFromCenter / maxDistance) * 1.5);

      ctx.strokeStyle = `rgba(73, 59, 124, ${opacity * 0.15})`; // Purple grid
      ctx.lineWidth = 0.5;
      ctx.beginPath();
      ctx.moveTo(px, 0);
      ctx.lineTo(px, height);
      ctx.stroke();
    }

    // Draw horizontal lines with opacity based on distance from center
    for (let y = -maxRange; y <= maxRange; y += gridSpacing) {
      const py = centerY + yardsToPixels(y);
      if (py < 0 || py > height) continue; // Skip lines outside canvas

      const distanceFromCenter = Math.abs(py - centerY);
      const opacity = Math.max(0, 1 - (distanceFromCenter / maxDistance) * 1.5);

      ctx.strokeStyle = `rgba(73, 59, 124, ${opacity * 0.15})`; // Purple grid
      ctx.lineWidth = 0.5;
      ctx.beginPath();
      ctx.moveTo(0, py);
      ctx.lineTo(width, py);
      ctx.stroke();
    }
  };

  const drawFixedTargetBoundary = (ctx: CanvasRenderingContext2D) => {
    // Target boundary for current hole
    const radius = yardsToPixels(targetRadius);

    // Draw target boundary circle only
    ctx.strokeStyle = '#493b7c';
    ctx.lineWidth = 3;
    ctx.beginPath();
    ctx.arc(centerX, centerY, radius, 0, Math.PI * 2);
    ctx.stroke();

    // Simple label at the top
    ctx.fillStyle = '#493b7c';
    ctx.font = '12px Inter';
    ctx.fontWeight = '600';
    ctx.textAlign = 'center';
    ctx.fillText(`Target`, centerX, centerY - radius - 8);
    ctx.textAlign = 'left';
  };


  const drawBreakevenRadius = (ctx: CanvasRenderingContext2D) => {
    const radius = yardsToPixels(breakevenRadius);

    // Draw dashed circle for breakeven
    ctx.strokeStyle = '#604c9c';
    ctx.lineWidth = 2.5;
    ctx.setLineDash([8, 4]);
    ctx.beginPath();
    ctx.arc(centerX, centerY, radius, 0, Math.PI * 2);
    ctx.stroke();
    ctx.setLineDash([]);

    // Simple label at the bottom
    ctx.fillStyle = '#604c9c';
    ctx.font = '12px Inter';
    ctx.fontWeight = '600';
    ctx.textAlign = 'center';
    ctx.fillText('Breakeven', centerX, centerY + radius + 18);
    ctx.textAlign = 'left';
  };

  const drawCenterPin = (ctx: CanvasRenderingContext2D) => {
    // Simple center crosshair
    ctx.strokeStyle = '#493b7c';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(centerX - 8, centerY);
    ctx.lineTo(centerX + 8, centerY);
    ctx.moveTo(centerX, centerY - 8);
    ctx.lineTo(centerX, centerY + 8);
    ctx.stroke();

    // Center dot
    ctx.fillStyle = '#493b7c';
    ctx.beginPath();
    ctx.arc(centerX, centerY, 3, 0, Math.PI * 2);
    ctx.fill();
  };

  const drawShot = (ctx: CanvasRenderingContext2D, shot: Shot, isAnimated: boolean) => {
    // Calculate x, y from distance and angle
    const x = centerX + yardsToPixels(shot.distance * Math.cos(shot.angle));
    const y = centerY + yardsToPixels(shot.distance * Math.sin(shot.angle));

    // Color based on profit/loss
    const color = shot.profit >= 0 ? '#604c9c' : '#ac7c6c';
    const radius = isAnimated ? 6 + Math.sin(animationFrame / 5) * 2 : 4;
    const alpha = isAnimated ? 0.8 + Math.sin(animationFrame / 3) * 0.2 : 0.6;

    // Draw shot marker
    ctx.fillStyle = color + Math.floor(alpha * 255).toString(16).padStart(2, '0');
    ctx.beginPath();
    ctx.arc(x, y, radius, 0, Math.PI * 2);
    ctx.fill();

    // Draw stroke
    ctx.strokeStyle = color;
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(x, y, radius, 0, Math.PI * 2);
    ctx.stroke();

    // If animated, draw line from center to shot
    if (isAnimated) {
      ctx.strokeStyle = color + '40';
      ctx.lineWidth = 1;
      ctx.setLineDash([3, 3]);
      ctx.beginPath();
      ctx.moveTo(centerX, centerY);
      ctx.lineTo(x, y);
      ctx.stroke();
      ctx.setLineDash([]);

      // Draw distance label
      ctx.fillStyle = '#dfc9ad';
      ctx.font = '12px Inter';
      ctx.fontWeight = '600';
      ctx.fillText(`${shot.distance.toFixed(1)}y`, x + 10, y - 10);
    }
  };

  return (
    <canvas
      ref={canvasRef}
      width={width}
      height={height}
      className="rounded-lg"
    />
  );
}
