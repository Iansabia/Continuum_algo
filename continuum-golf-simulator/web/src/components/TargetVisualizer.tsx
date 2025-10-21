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
  shots: Shot[];
  currentShot?: Shot | null;
  width?: number;
  height?: number;
}

export default function TargetVisualizer({
  sigma,
  breakevenRadius,
  shots,
  currentShot,
  width = 400,
  height = 400,
}: TargetVisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [animationFrame, setAnimationFrame] = useState(0);

  // Convert yards to pixels (scale for visualization)
  const SCALE = 5; // 5 pixels per yard
  const centerX = width / 2;
  const centerY = height / 2;

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

    // Draw probability density rings (3σ, 2σ, 1σ)
    drawProbabilityRings(ctx);

    // Draw breakeven radius
    drawBreakevenRadius(ctx);

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
  }, [sigma, breakevenRadius, shots, currentShot, animationFrame]);

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
    ctx.strokeStyle = '#2a2a2a';
    ctx.lineWidth = 0.5;

    // Draw vertical lines every 10 yards
    for (let x = -40; x <= 40; x += 10) {
      const px = centerX + yardsToPixels(x);
      ctx.beginPath();
      ctx.moveTo(px, 0);
      ctx.lineTo(px, height);
      ctx.stroke();
    }

    // Draw horizontal lines every 10 yards
    for (let y = -40; y <= 40; y += 10) {
      const py = centerY + yardsToPixels(y);
      ctx.beginPath();
      ctx.moveTo(0, py);
      ctx.lineTo(width, py);
      ctx.stroke();
    }
  };

  const drawProbabilityRings = (ctx: CanvasRenderingContext2D) => {
    const rings = [
      { sigma: 3, color: '#493b7c', alpha: 0.1 },
      { sigma: 2, color: '#604c9c', alpha: 0.15 },
      { sigma: 1, color: '#9e8cb4', alpha: 0.25 },
    ];

    rings.forEach((ring) => {
      const radius = yardsToPixels(sigma * ring.sigma);

      // Fill
      ctx.fillStyle = ring.color + Math.floor(ring.alpha * 255).toString(16).padStart(2, '0');
      ctx.beginPath();
      ctx.arc(centerX, centerY, radius, 0, Math.PI * 2);
      ctx.fill();

      // Stroke
      ctx.strokeStyle = ring.color;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(centerX, centerY, radius, 0, Math.PI * 2);
      ctx.stroke();

      // Label
      ctx.fillStyle = '#dfc9ad';
      ctx.font = '12px Inter';
      ctx.fillText(`${ring.sigma}σ`, centerX + radius - 20, centerY - 5);
    });
  };

  const drawBreakevenRadius = (ctx: CanvasRenderingContext2D) => {
    const radius = yardsToPixels(breakevenRadius);

    ctx.strokeStyle = '#7e6649';
    ctx.lineWidth = 2;
    ctx.setLineDash([5, 5]);
    ctx.beginPath();
    ctx.arc(centerX, centerY, radius, 0, Math.PI * 2);
    ctx.stroke();
    ctx.setLineDash([]);

    // Label
    ctx.fillStyle = '#dfc9ad';
    ctx.font = '14px Inter';
    ctx.fillText('Breakeven', centerX + radius - 50, centerY + 15);
  };

  const drawCenterPin = (ctx: CanvasRenderingContext2D) => {
    // Pin base (circle)
    ctx.fillStyle = '#D4AF37';
    ctx.beginPath();
    ctx.arc(centerX, centerY, 5, 0, Math.PI * 2);
    ctx.fill();

    // Pin top (triangle)
    ctx.fillStyle = '#D4AF37';
    ctx.beginPath();
    ctx.moveTo(centerX, centerY - 5);
    ctx.lineTo(centerX - 4, centerY - 15);
    ctx.lineTo(centerX + 4, centerY - 15);
    ctx.closePath();
    ctx.fill();

    // Cross-hair
    ctx.strokeStyle = '#D4AF37';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(centerX - 10, centerY);
    ctx.lineTo(centerX + 10, centerY);
    ctx.moveTo(centerX, centerY - 10);
    ctx.lineTo(centerX, centerY + 10);
    ctx.stroke();
  };

  const drawShot = (ctx: CanvasRenderingContext2D, shot: Shot, isAnimated: boolean) => {
    // Calculate x, y from distance and angle
    const x = centerX + yardsToPixels(shot.distance * Math.cos(shot.angle));
    const y = centerY + yardsToPixels(shot.distance * Math.sin(shot.angle));

    // Color based on profit/loss
    const color = shot.profit >= 0 ? '#10B981' : '#EF4444';
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
      ctx.fillText(`${shot.distance.toFixed(1)}y`, x + 10, y - 10);
    }
  };

  return (
    <div className="flex flex-col items-center">
      <canvas
        ref={canvasRef}
        width={width}
        height={height}
        className="bg-gray-900 rounded-lg shadow-lg border-2 border-brand-deep-purple"
      />
      <div className="mt-4 text-center text-sm text-gray-400">
        <p>σ = {sigma.toFixed(2)} yards | Breakeven = {breakevenRadius.toFixed(2)} yards</p>
      </div>
    </div>
  );
}
