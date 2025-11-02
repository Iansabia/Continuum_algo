import React, { useRef, useState, useEffect } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import BVNHeatmap3D from './BVNHeatmap3D';

interface ShotPoint {
  x: number;
  y: number;
  distance: number;
  angle: number;
  wager: number;
  payout: number;
  profit: number;
}

interface MCMCState {
  shotNum: number;
  muX: number;
  muY: number;
  sigmaX: number;
  sigmaY: number;
  confidence: number;
  pMax: number;
  rtp: number;
}

const PatternDrawingDemo: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isDrawing, setIsDrawing] = useState(false);
  const [drawnPoints, setDrawnPoints] = useState<Array<{x: number, y: number}>>([]);
  const [shotData, setShotData] = useState<ShotPoint[]>([]);
  const [mcmcHistory, setMcmcHistory] = useState<MCMCState[]>([]);
  const [isSimulating, setIsSimulating] = useState(false);
  const [selectedPattern, setSelectedPattern] = useState<'custom' | 'circle' | 'oval' | 'cluster' | 'scatter'>('custom');

  // Canvas dimensions
  const CANVAS_SIZE = 400;
  const CENTER = CANVAS_SIZE / 2;

  // Helper function to get CSS variable color values
  const getCSSColor = (varName: string): string => {
    if (typeof window === 'undefined') return '#ffffff';
    return getComputedStyle(document.documentElement).getPropertyValue(varName).trim() || '#ffffff';
  };

  useEffect(() => {
    if (canvasRef.current) {
      const canvas = canvasRef.current;
      const ctx = canvas.getContext('2d');
      if (ctx) {
        // Get CSS colors
        const deepPurple = getCSSColor('--brand-deep-purple');
        const brightPurple = getCSSColor('--brand-bright-purple');
        const tan = getCSSColor('--brand-tan');
        const lavender = getCSSColor('--brand-lavender');

        // Helper to convert hex to rgba
        const hexToRgba = (hex: string, alpha: number) => {
          const r = parseInt(hex.slice(1, 3), 16);
          const g = parseInt(hex.slice(3, 5), 16);
          const b = parseInt(hex.slice(5, 7), 16);
          return `rgba(${r}, ${g}, ${b}, ${alpha})`;
        };

        // Clear canvas with deep purple background
        ctx.fillStyle = deepPurple;
        ctx.fillRect(0, 0, CANVAS_SIZE, CANVAS_SIZE);

        // Draw grid with lavender at low opacity
        ctx.strokeStyle = hexToRgba(lavender, 0.1);
        ctx.lineWidth = 1;
        for (let i = 0; i <= CANVAS_SIZE; i += 50) {
          ctx.beginPath();
          ctx.moveTo(i, 0);
          ctx.lineTo(i, CANVAS_SIZE);
          ctx.stroke();
          ctx.beginPath();
          ctx.moveTo(0, i);
          ctx.lineTo(CANVAS_SIZE, i);
          ctx.stroke();
        }

        // Draw center crosshair with tan
        ctx.strokeStyle = tan;
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(CENTER - 20, CENTER);
        ctx.lineTo(CENTER + 20, CENTER);
        ctx.stroke();
        ctx.beginPath();
        ctx.moveTo(CENTER, CENTER - 20);
        ctx.lineTo(CENTER, CENTER + 20);
        ctx.stroke();

        // Draw center circle (hole) with lavender
        ctx.strokeStyle = lavender;
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.arc(CENTER, CENTER, 10, 0, Math.PI * 2);
        ctx.stroke();

        // Draw distance circles with bright purple at 30% opacity
        ctx.strokeStyle = hexToRgba(brightPurple, 0.3);
        ctx.lineWidth = 1;
        for (let r = 50; r < CANVAS_SIZE; r += 50) {
          ctx.beginPath();
          ctx.arc(CENTER, CENTER, r, 0, Math.PI * 2);
          ctx.stroke();
        }

        // Draw the pattern outline with tan
        if (drawnPoints.length > 0) {
          ctx.strokeStyle = tan;
          ctx.lineWidth = 3;
          ctx.beginPath();
          ctx.moveTo(drawnPoints[0].x, drawnPoints[0].y);
          for (let i = 1; i < drawnPoints.length; i++) {
            ctx.lineTo(drawnPoints[i].x, drawnPoints[i].y);
          }
          ctx.stroke();
        }

        // Draw shot points with lavender fade
        shotData.forEach((shot, i) => {
          const alpha = Math.max(0.3, 1 - i / shotData.length);
          ctx.fillStyle = hexToRgba(lavender, alpha);
          ctx.beginPath();
          ctx.arc(shot.x, shot.y, 4, 0, Math.PI * 2);
          ctx.fill();
        });
      }
    }
  }, [drawnPoints, shotData]);

  const handleMouseDown = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (selectedPattern !== 'custom') return;
    setIsDrawing(true);
    const canvas = canvasRef.current;
    const rect = canvas?.getBoundingClientRect();
    if (rect && canvas) {
      // Scale coordinates from display size to canvas size
      const scaleX = canvas.width / rect.width;
      const scaleY = canvas.height / rect.height;
      const x = (e.clientX - rect.left) * scaleX;
      const y = (e.clientY - rect.top) * scaleY;
      setDrawnPoints([{x, y}]);
    }
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!isDrawing || selectedPattern !== 'custom') return;
    const canvas = canvasRef.current;
    const rect = canvas?.getBoundingClientRect();
    if (rect && canvas) {
      // Scale coordinates from display size to canvas size
      const scaleX = canvas.width / rect.width;
      const scaleY = canvas.height / rect.height;
      const x = (e.clientX - rect.left) * scaleX;
      const y = (e.clientY - rect.top) * scaleY;
      setDrawnPoints(prev => [...prev, {x, y}]);
    }
  };

  const handleMouseUp = () => {
    setIsDrawing(false);
  };

  const clearPattern = () => {
    setDrawnPoints([]);
    setShotData([]);
    setMcmcHistory([]);
  };

  const generatePredefinedPattern = (type: 'circle' | 'oval' | 'cluster' | 'scatter') => {
    // Generate pattern outline for visualization
    const points: Array<{x: number, y: number}> = [];
    const numPoints = 100;

    switch (type) {
      case 'circle':
        for (let i = 0; i <= numPoints; i++) {
          const angle = (i / numPoints) * Math.PI * 2;
          const r = 60; // 30ft radius in canvas units
          points.push({
            x: CENTER + r * Math.cos(angle),
            y: CENTER + r * Math.sin(angle)
          });
        }
        break;
      case 'oval':
        for (let i = 0; i <= numPoints; i++) {
          const angle = (i / numPoints) * Math.PI * 2;
          const rx = 80; // 40ft horizontal
          const ry = 40; // 20ft vertical
          points.push({
            x: CENTER + rx * Math.cos(angle),
            y: CENTER + ry * Math.sin(angle)
          });
        }
        break;
      case 'cluster':
        // Inner circle with occasional outliers
        for (let i = 0; i <= numPoints; i++) {
          const angle = (i / numPoints) * Math.PI * 2;
          const r = Math.random() < 0.02 ? 180 : 60; // 2% outliers at 90ft, 98% at 30ft
          points.push({
            x: CENTER + r * Math.cos(angle),
            y: CENTER + r * Math.sin(angle)
          });
        }
        break;
      case 'scatter':
        // Wide random scatter
        for (let i = 0; i <= numPoints; i++) {
          const angle = (i / numPoints) * Math.PI * 2;
          const r = 90 + (Math.random() - 0.5) * 30; // 45ft ± 15ft noise
          points.push({
            x: CENTER + r * Math.cos(angle),
            y: CENTER + r * Math.sin(angle)
          });
        }
        break;
    }

    setDrawnPoints(points);
  };

  // Helper function to get bounding box of polygon
  const getBoundingBox = (polygon: Array<{x: number, y: number}>) => {
    const xs = polygon.map(p => p.x);
    const ys = polygon.map(p => p.y);
    return {
      minX: Math.min(...xs),
      maxX: Math.max(...xs),
      minY: Math.min(...ys),
      maxY: Math.max(...ys)
    };
  };
  const runSimulation = async () => {
    if (drawnPoints.length < 10) {
      alert('Please draw a larger pattern or select a predefined pattern!');
      return;
    }

    setIsSimulating(true);
    setShotData([]);
    setMcmcHistory([]);

    // Simulate shots based on the pattern
    const NUM_SHOTS = 100;
    const shots: ShotPoint[] = [];
    const mcmc: MCMCState[] = [];

    // Calculate pattern bounds to determine true BVN parameters
    const bounds = getBoundingBox(drawnPoints);
    const widthPixels = bounds.maxX - bounds.minX;
    const heightPixels = bounds.maxY - bounds.minY;

    // For custom drawn patterns, we need to sample from the actual drawn shape
    // For predefined patterns, we can use analytical BVN parameters
    const isCustomPattern = selectedPattern === 'custom';

    // TRUE BVN parameters (fixed for entire simulation)
    // For BVN: ~95% of shots fall within 2σ radius
    // So if we want shots within the drawn pattern radius R: σ = R / 2
    // This ensures the drawn pattern boundary contains ~95% of shots
    const trueMuX = 0; // No bias for predefined patterns
    const trueMuY = 0;
    const radiusX_feet = (widthPixels / 2) / 2; // width/2 in pixels, /2 for feet
    const radiusY_feet = (heightPixels / 2) / 2;
    const trueSigmaX = radiusX_feet / 2; // σ = R/2 so 2σ = R (95% within)
    const trueSigmaY = radiusY_feet / 2;

    // Estimate true correlation from drawn points
    let trueRho = 0;
    if (isCustomPattern && drawnPoints.length > 2) {
      const pointsInFeet = drawnPoints.map(p => ({
        x: (p.x - CENTER) / 2,
        y: (p.y - CENTER) / 2
      }));
      const meanX = pointsInFeet.reduce((sum, p) => sum + p.x, 0) / pointsInFeet.length;
      const meanY = pointsInFeet.reduce((sum, p) => sum + p.y, 0) / pointsInFeet.length;
      const varX = pointsInFeet.reduce((sum, p) => sum + Math.pow(p.x - meanX, 2), 0) / pointsInFeet.length;
      const varY = pointsInFeet.reduce((sum, p) => sum + Math.pow(p.y - meanY, 2), 0) / pointsInFeet.length;
      const covXY = pointsInFeet.reduce((sum, p) => sum + (p.x - meanX) * (p.y - meanY), 0) / pointsInFeet.length;
      const sigmaX = Math.sqrt(varX);
      const sigmaY = Math.sqrt(varY);
      trueRho = sigmaX > 0 && sigmaY > 0 ? covXY / (sigmaX * sigmaY) : 0;
      // Clamp to valid range
      trueRho = Math.max(-0.99, Math.min(0.99, trueRho));
    }

    // MCMC estimation state (what we're learning)
    let estimatedMuX = 0;
    let estimatedMuY = 0;
    let estimatedSigmaX = 20; // Initial guesses
    let estimatedSigmaY = 20;
    let totalWagered = 0;
    let totalWon = 0;

    // Fat-tail parameters (matches Rust implementation)
    const FAT_TAIL_PROB = 0.02;
    const FAT_TAIL_MULT = 3.0;

    for (let i = 0; i < NUM_SHOTS; i++) {
      // Yield to UI every 10 shots to prevent freezing
      if (i % 10 === 0) {
        await new Promise(resolve => setTimeout(resolve, 0));
      }

      let x: number, y: number, shotX_feet: number, shotY_feet: number;

      // Use BVN with Box-Muller transform and Cholesky decomposition for correlation
      // Matches production implementation in distributions.rs

      // Box-Muller to generate two independent N(0,1) samples
      const u1 = Math.random();
      const u2 = Math.random();
      const z0 = Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
      const z1 = Math.sqrt(-2 * Math.log(u1)) * Math.sin(2 * Math.PI * u2);

      // Determine if this is a fat-tail shot (2% probability)
      const isFatTail = Math.random() < FAT_TAIL_PROB;
      const sigma_x_effective = isFatTail ? trueSigmaX * FAT_TAIL_MULT : trueSigmaX;
      const sigma_y_effective = isFatTail ? trueSigmaY * FAT_TAIL_MULT : trueSigmaY;

      // Apply BVN transformation with correlation using Cholesky decomposition
      // Cholesky factor L: [[σ_x, 0], [ρσ_y, σ_y√(1-ρ²)]]
      // X = μ_x + σ_x × Z₀
      // Y = μ_y + σ_y × (ρ × Z₀ + √(1-ρ²) × Z₁)
      shotX_feet = trueMuX + sigma_x_effective * z0;
      shotY_feet = trueMuY + sigma_y_effective * (trueRho * z0 + Math.sqrt(1 - trueRho * trueRho) * z1);

      // Convert to canvas coordinates (2 pixels = 1 foot)
      x = CENTER + shotX_feet * 2;
      y = CENTER + shotY_feet * 2;

      // Calculate radial distance and angle for payout
      const distance = Math.sqrt(shotX_feet * shotX_feet + shotY_feet * shotY_feet);
      const angle = Math.atan2(shotY_feet, shotX_feet);

      // Simulate wager and payout (will be calculated below)
      const wager = 5 + Math.random() * 5;
      let multiplier = 0;
      let payout = 0;
      let profit = 0;

      shots.push({ x, y, distance, angle, wager, payout, profit });

      // Update MCMC estimation (simplified BVN parameter learning)
      if (i >= 5) {
        const recentShots = shots.slice(Math.max(0, i - 20), i + 1);

        // Estimate BVN parameters from recent shots (x, y coordinates in feet)
        const shotsInFeet = recentShots.map(s => ({
          x: (s.x - CENTER) / 2,  // Convert canvas pixels to feet
          y: (s.y - CENTER) / 2
        }));

        // Mean (bias)
        estimatedMuX = shotsInFeet.reduce((sum, s) => sum + s.x, 0) / shotsInFeet.length;
        estimatedMuY = shotsInFeet.reduce((sum, s) => sum + s.y, 0) / shotsInFeet.length;

        // Standard deviation (dispersion)
        const varX = shotsInFeet.reduce((sum, s) => sum + Math.pow(s.x - estimatedMuX, 2), 0) / shotsInFeet.length;
        const varY = shotsInFeet.reduce((sum, s) => sum + Math.pow(s.y - estimatedMuY, 2), 0) / shotsInFeet.length;
        estimatedSigmaX = Math.sqrt(varX);
        estimatedSigmaY = Math.sqrt(varY);

        // Covariance and correlation
        const covXY = shotsInFeet.reduce((sum, s) => sum + (s.x - estimatedMuX) * (s.y - estimatedMuY), 0) / shotsInFeet.length;
        const rho = covXY / (estimatedSigmaX * estimatedSigmaY);

        // Use FIXED hole configuration (matches Rust implementation)
        // Hole 4: Mid Iron (150 yds) - works for sigma 20-40ft
        // This provides stable RTP across different pattern sizes
        const dMax = 47.58;
        const k = 6.0;

        // Calculate P_max using 2D grid integration over BVN (matches Rust implementation)
        // P_max = target_RTP / expected_payout
        // expected_payout = ∬ payout(r) × BVN_PDF(x, y) dx dy

        // BVN PDF function with correlation
        const bvnPDF = (x: number, y: number, mu_x: number, mu_y: number, sigma_x: number, sigma_y: number, rho_param: number) => {
          // Clamp rho to valid range to prevent numerical issues
          const rho_clamped = Math.max(-0.9999, Math.min(0.9999, rho_param));

          const dx = (x - mu_x) / sigma_x;
          const dy = (y - mu_y) / sigma_y;

          // Quadratic form with correlation term
          const z = dx * dx - 2.0 * rho_clamped * dx * dy + dy * dy;

          // Compute exponent with correlation adjustment
          const rho_sq = rho_clamped * rho_clamped;
          const expTerm = Math.exp(-z / (2.0 * (1.0 - rho_sq)));

          // Normalization constant includes correlation factor
          const norm = 1.0 / (2.0 * Math.PI * sigma_x * sigma_y * Math.sqrt(1.0 - rho_sq));

          return norm * expTerm;
        };

        // 2D grid integration (100x100 grid for browser performance)
        const gridSize = 100;
        const xMin = estimatedMuX - 4 * estimatedSigmaX;
        const xMax = estimatedMuX + 4 * estimatedSigmaX;
        const yMin = estimatedMuY - 4 * estimatedSigmaY;
        const yMax = estimatedMuY + 4 * estimatedSigmaY;
        const dx = (xMax - xMin) / gridSize;
        const dy = (yMax - yMin) / gridSize;

        let expectedPayoutNormal = 0;

        for (let ix = 0; ix < gridSize; ix++) {
          for (let iy = 0; iy < gridSize; iy++) {
            const x = xMin + (ix + 0.5) * dx;
            const y = yMin + (iy + 0.5) * dy;

            // Radial distance from pin
            const r = Math.sqrt(x * x + y * y);

            // Payout function
            const payout = r > dMax ? 0 : Math.pow(1 - r / dMax, k);

            // BVN probability density with correlation
            const prob = bvnPDF(x, y, estimatedMuX, estimatedMuY, estimatedSigmaX, estimatedSigmaY, rho);

            // Accumulate (Riemann sum)
            expectedPayoutNormal += payout * prob * dx * dy;
          }
        }

        // Also calculate for fat-tail shots (2% probability, 3× dispersion)
        const sigma_x_fat = estimatedSigmaX * FAT_TAIL_MULT;
        const sigma_y_fat = estimatedSigmaY * FAT_TAIL_MULT;
        const xMinFat = estimatedMuX - 4 * sigma_x_fat;
        const xMaxFat = estimatedMuX + 4 * sigma_x_fat;
        const yMinFat = estimatedMuY - 4 * sigma_y_fat;
        const yMaxFat = estimatedMuY + 4 * sigma_y_fat;
        const dxFat = (xMaxFat - xMinFat) / gridSize;
        const dyFat = (yMaxFat - yMinFat) / gridSize;

        let expectedPayoutFat = 0;

        for (let ix = 0; ix < gridSize; ix++) {
          for (let iy = 0; iy < gridSize; iy++) {
            const x = xMinFat + (ix + 0.5) * dxFat;
            const y = yMinFat + (iy + 0.5) * dyFat;
            const r = Math.sqrt(x * x + y * y);
            const payout = r > dMax ? 0 : Math.pow(1 - r / dMax, k);
            const prob = bvnPDF(x, y, estimatedMuX, estimatedMuY, sigma_x_fat, sigma_y_fat, rho);
            expectedPayoutFat += payout * prob * dxFat * dyFat;
          }
        }

        // Weighted average (98% normal + 2% fat-tail)
        const expectedPayout = (1 - FAT_TAIL_PROB) * expectedPayoutNormal + FAT_TAIL_PROB * expectedPayoutFat;

        // P_max = target_RTP / expected_payout
        const targetRTP = 0.85;
        const pMax = targetRTP / Math.max(0.01, expectedPayout);

        // Calculate payout using the actual Continuum formula
        // P(d) = P_max * (1 - d/d_max)^k
        if (distance <= dMax) {
          const normalized = 1.0 - (distance / dMax);
          multiplier = pMax * Math.pow(normalized, k);
        }
        payout = wager * multiplier;
        profit = payout - wager;

        // Update the shot object with payout info
        shots[i].payout = payout;
        shots[i].profit = profit;

        totalWagered += wager;
        totalWon += payout;

        // Calculate confidence based on MCMC implementation (src/math/mcmc.rs:415-435)
        // Matches Rust: 70% from observation count + 30% from credible interval width
        const observationCount = i + 1;
        const countConfidence = Math.min(1.0, observationCount / 30.0);

        // Approximate MCMC credible interval using recent estimate variability
        // Rust uses: interval_confidence = (1.0 - relative_width / 2.0)
        // where relative_width = ci_width / estimate
        let intervalConfidence = 0.5; // Default moderate confidence

        if (mcmc.length >= 5) {
          // Use last 10 estimates to approximate posterior distribution
          const windowSize = Math.min(10, mcmc.length);
          const recentEstimates = mcmc.slice(-windowSize);

          // Calculate mean and std for both sigmaX and sigmaY
          const avgSigmaX = recentEstimates.reduce((sum, m) => sum + m.sigmaX, 0) / windowSize;
          const avgSigmaY = recentEstimates.reduce((sum, m) => sum + m.sigmaY, 0) / windowSize;

          const stdSigmaX = Math.sqrt(
            recentEstimates.reduce((sum, m) => sum + Math.pow(m.sigmaX - avgSigmaX, 2), 0) / windowSize
          );
          const stdSigmaY = Math.sqrt(
            recentEstimates.reduce((sum, m) => sum + Math.pow(m.sigmaY - avgSigmaY, 2), 0) / windowSize
          );

          // Approximate 95% credible interval: [mean - 1.96*std, mean + 1.96*std]
          // Relative width = 2 * 1.96 * std / mean = 3.92 * std / mean
          const relativeWidthX = (3.92 * stdSigmaX) / Math.max(avgSigmaX, 1.0);
          const relativeWidthY = (3.92 * stdSigmaY) / Math.max(avgSigmaY, 1.0);
          const avgRelativeWidth = (relativeWidthX + relativeWidthY) / 2;

          // Match Rust formula: (1.0 - relative_width / 2.0).max(0.0).min(1.0)
          intervalConfidence = Math.max(0.0, Math.min(1.0, 1.0 - avgRelativeWidth / 2.0));
        }

        // Combine: 70% from count, 30% from interval (matches Rust MCMC)
        const confidence = (countConfidence * 0.7 + intervalConfidence * 0.3) * 100;

        mcmc.push({
          shotNum: i + 1,
          muX: estimatedMuX,
          muY: estimatedMuY,
          sigmaX: estimatedSigmaX,
          sigmaY: estimatedSigmaY,
          confidence,
          pMax,
          rtp: (totalWon / totalWagered) * 100
        });
      }

      // Animate the simulation
      await new Promise(resolve => setTimeout(resolve, 50));
      setShotData([...shots]);
      if (mcmc.length > 0) {
        setMcmcHistory([...mcmc]);
      }
    }

    setIsSimulating(false);
  };

  return (
    <div className="h-full w-full bg-gradient-to-br from-[var(--brand-deep-purple)]/30 to-[#000000] rounded-2xl p-4 backdrop-blur-3xl overflow-hidden flex flex-col">
      {/* Glass morphism overlay */}
      <div className="absolute inset-0 bg-gradient-to-br from-[var(--brand-bright-purple)]/[0.15] via-transparent to-[var(--brand-dark-gold)]/[0.05] pointer-events-none rounded-2xl" />

      <div className="relative mb-3">
        <h2 className="text-2xl font-bold text-[var(--brand-tan)] mb-1">Interactive Pattern Drawing Demo</h2>
        <p className="text-[var(--brand-lavender)] text-xs">
          Draw a custom shot pattern or select a predefined one to see how MCMC adapts in real-time
        </p>
      </div>

      <div className="relative flex-1 grid grid-cols-1 lg:grid-cols-2 gap-3 min-h-0">
        {/* Left Column: Canvas + 3D View */}
        <div className="relative flex flex-col gap-2 min-h-0">
          {/* Canvas Section */}
          <div className="flex-shrink-0">
            <div className="flex justify-between items-center mb-2">
              <h3 className="text-sm font-semibold text-[var(--brand-tan)]">Shot Pattern Canvas</h3>
              <div className="flex gap-2">
                <button
                  onClick={clearPattern}
                  className="px-2 py-1 bg-[var(--brand-bright-purple)]/20 hover:bg-[var(--brand-bright-purple)]/40 rounded-lg text-[var(--brand-lavender)] text-xs border border-[var(--brand-lavender)]/30 transition-all"
                >
                  Clear
                </button>
                <button
                  onClick={runSimulation}
                  disabled={isSimulating || drawnPoints.length < 10}
                  className="px-3 py-1 bg-[var(--brand-bright-purple)] hover:bg-[var(--brand-deep-purple)] disabled:bg-[var(--brand-bright-purple)]/20 disabled:cursor-not-allowed rounded-lg text-[var(--brand-tan)] text-xs font-medium transition-all"
                >
                  {isSimulating ? 'Simulating...' : 'Run Simulation'}
                </button>
              </div>
            </div>

            {/* Pattern Selection */}
            <div className="mb-2 flex flex-wrap gap-1">
              {(['custom', 'circle', 'oval', 'cluster', 'scatter'] as const).map(pattern => (
                <button
                  key={pattern}
                  onClick={() => {
                    setSelectedPattern(pattern);
                    if (pattern !== 'custom') {
                      generatePredefinedPattern(pattern);
                    }
                  }}
                  className={`px-2 py-0.5 rounded-lg text-xs transition-all ${
                    selectedPattern === pattern
                      ? 'bg-[var(--brand-bright-purple)] text-[var(--brand-tan)] border-2 border-[var(--brand-lavender)]'
                      : 'bg-[var(--brand-bright-purple)]/20 text-[var(--brand-lavender)] border border-[var(--brand-lavender)]/30 hover:bg-[var(--brand-bright-purple)]/40'
                  }`}
                >
                  {pattern.charAt(0).toUpperCase() + pattern.slice(1)}
                </button>
              ))}
            </div>

            <canvas
              ref={canvasRef}
              width={CANVAS_SIZE}
              height={CANVAS_SIZE}
              onMouseDown={handleMouseDown}
              onMouseMove={handleMouseMove}
              onMouseUp={handleMouseUp}
              onMouseLeave={handleMouseUp}
              className="rounded-lg cursor-crosshair w-full max-w-[350px] mx-auto"
              style={{ imageRendering: 'crisp-edges' }}
            />

            <div className="mt-1 text-[10px] text-[var(--brand-lavender)]/70 text-center">
              {selectedPattern === 'custom'
                ? 'Draw your custom pattern by clicking and dragging on the canvas'
                : `Selected: ${selectedPattern.charAt(0).toUpperCase() + selectedPattern.slice(1)} pattern`
              }
            </div>
          </div>

          {/* BVN 3D View Section */}
          {mcmcHistory.length > 0 && (
            <div className="flex-1 min-h-0 flex flex-col">
              <h3 className="text-sm font-semibold text-[var(--brand-tan)] mb-1">3D Probability Distribution</h3>
              <div className="flex-1 min-h-0">
                <BVNHeatmap3D
                  sigmaX={mcmcHistory[mcmcHistory.length - 1].sigmaX}
                  sigmaY={mcmcHistory[mcmcHistory.length - 1].sigmaY}
                  currentPmax={mcmcHistory[mcmcHistory.length - 1].pMax}
                  shots={shotData}
                />
              </div>
            </div>
          )}
        </div>

        {/* MCMC Metrics */}
        <div className="relative flex flex-col gap-2 min-h-0">
          {mcmcHistory.length > 0 && (
            <>
              {/* Current Stats - Compact */}
              <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl p-2 border border-[var(--brand-tan)]/20 flex-shrink-0">
                <h3 className="text-sm font-semibold text-[var(--brand-tan)] mb-2">Current BVN State</h3>
                <div className="grid grid-cols-3 gap-2">
                  <div className="bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                    <div className="text-[10px] text-[var(--brand-lavender)] mb-0.5">σ_x</div>
                    <div className="text-base font-bold text-[var(--brand-tan)]">
                      {mcmcHistory[mcmcHistory.length - 1].sigmaX.toFixed(1)}ft
                    </div>
                  </div>
                  <div className="bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                    <div className="text-[10px] text-[var(--brand-lavender)] mb-0.5">σ_y</div>
                    <div className="text-base font-bold text-[var(--brand-tan)]">
                      {mcmcHistory[mcmcHistory.length - 1].sigmaY.toFixed(1)}ft
                    </div>
                  </div>
                  <div className="bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                    <div className="text-[10px] text-[var(--brand-lavender)] mb-0.5">Confidence</div>
                    <div className="text-base font-bold text-[var(--brand-tan)]">
                      {mcmcHistory[mcmcHistory.length - 1].confidence.toFixed(1)}%
                    </div>
                  </div>
                  <div className="bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                    <div className="text-[10px] text-[var(--brand-lavender)] mb-0.5">μ_x</div>
                    <div className="text-base font-bold text-[var(--brand-tan)]">
                      {mcmcHistory[mcmcHistory.length - 1].muX.toFixed(1)}ft
                    </div>
                  </div>
                  <div className="bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                    <div className="text-[10px] text-[var(--brand-lavender)] mb-0.5">μ_y</div>
                    <div className="text-base font-bold text-[var(--brand-tan)]">
                      {mcmcHistory[mcmcHistory.length - 1].muY.toFixed(1)}ft
                    </div>
                  </div>
                  <div className="bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                    <div className="text-[10px] text-[var(--brand-lavender)] mb-0.5">P_max</div>
                    <div className="text-base font-bold text-[var(--brand-tan)]">
                      {mcmcHistory[mcmcHistory.length - 1].pMax.toFixed(2)}x
                    </div>
                  </div>
                  <div className="col-span-3 bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                    <div className="text-[10px] text-[var(--brand-lavender)] mb-0.5">RTP</div>
                    <div className="text-xl font-bold text-[var(--brand-tan)]">
                      {mcmcHistory[mcmcHistory.length - 1].rtp.toFixed(1)}%
                    </div>
                  </div>
                </div>
              </div>

              {/* Sigma Evolution Chart */}
              <div className="flex-1 min-h-0 bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl p-3 border border-[var(--brand-tan)]/20 flex flex-col">
                <h3 className="text-sm font-semibold text-[var(--brand-tan)] mb-2">σ Evolution</h3>
                <div className="flex-1 min-h-0">
                  <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={mcmcHistory}>
                    <CartesianGrid strokeDasharray="3 3" stroke="var(--brand-lavender)" opacity={0.1} />
                    <XAxis
                      dataKey="shotNum"
                      stroke="var(--brand-lavender)"
                      tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                      label={{ value: 'Shot #', position: 'insideBottom', offset: -5, fill: 'var(--brand-lavender)' }}
                    />
                    <YAxis
                      stroke="var(--brand-lavender)"
                      tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                      label={{ value: 'σ (ft)', angle: -90, position: 'insideLeft', fill: 'var(--brand-lavender)' }}
                    />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: 'var(--brand-deep-purple)',
                        border: '1px solid var(--brand-tan)',
                        borderRadius: '12px',
                        color: 'var(--brand-tan)',
                        padding: '8px 12px'
                      }}
                      labelStyle={{ color: 'var(--brand-tan)' }}
                    />
                    <Legend />
                    <Line type="monotone" dataKey="sigmaX" stroke="var(--brand-bright-purple)" strokeWidth={2.5} dot={false} name="σ_x" />
                    <Line type="monotone" dataKey="sigmaY" stroke="var(--brand-lavender)" strokeWidth={2.5} strokeDasharray="5 5" dot={false} name="σ_y" />
                  </LineChart>
                  </ResponsiveContainer>
                </div>
              </div>

              {/* P_max Evolution Chart */}
              <div className="flex-1 min-h-0 bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl p-3 border border-[var(--brand-tan)]/20 flex flex-col">
                <h3 className="text-sm font-semibold text-[var(--brand-tan)] mb-2">P_max Adaptation</h3>
                <div className="flex-1 min-h-0">
                  <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={mcmcHistory}>
                    <CartesianGrid strokeDasharray="3 3" stroke="var(--brand-lavender)" opacity={0.1} />
                    <XAxis
                      dataKey="shotNum"
                      stroke="var(--brand-lavender)"
                      tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                      label={{ value: 'Shot #', position: 'insideBottom', offset: -5, fill: 'var(--brand-lavender)' }}
                    />
                    <YAxis
                      stroke="var(--brand-lavender)"
                      tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                      label={{ value: 'P_max (multiplier)', angle: -90, position: 'insideLeft', fill: 'var(--brand-lavender)' }}
                    />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: 'var(--brand-deep-purple)',
                        border: '1px solid var(--brand-tan)',
                        borderRadius: '12px',
                        color: 'var(--brand-tan)',
                        padding: '8px 12px'
                      }}
                      labelStyle={{ color: 'var(--brand-tan)' }}
                    />
                    <Legend />
                    <Line type="monotone" dataKey="pMax" stroke="var(--brand-dark-gold)" strokeWidth={2.5} dot={false} name="P_max" />
                  </LineChart>
                  </ResponsiveContainer>
                </div>
              </div>

              {/* RTP Evolution Chart */}
              <div className="flex-1 min-h-0 bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl p-3 border border-[var(--brand-tan)]/20 flex flex-col">
                <h3 className="text-sm font-semibold text-[var(--brand-tan)] mb-2">RTP Convergence</h3>
                <div className="flex-1 min-h-0">
                  <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={mcmcHistory}>
                    <CartesianGrid strokeDasharray="3 3" stroke="var(--brand-lavender)" opacity={0.1} />
                    <XAxis
                      dataKey="shotNum"
                      stroke="var(--brand-lavender)"
                      tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                      label={{ value: 'Shot #', position: 'insideBottom', offset: -5, fill: 'var(--brand-lavender)' }}
                    />
                    <YAxis
                      stroke="var(--brand-lavender)"
                      tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                      domain={[0, 150]}
                      label={{ value: 'RTP %', angle: -90, position: 'insideLeft', fill: 'var(--brand-lavender)' }}
                    />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: 'var(--brand-deep-purple)',
                        border: '1px solid var(--brand-tan)',
                        borderRadius: '12px',
                        color: 'var(--brand-tan)',
                        padding: '8px 12px'
                      }}
                      labelStyle={{ color: 'var(--brand-tan)' }}
                    />
                    <Line type="monotone" dataKey="rtp" stroke="var(--brand-bright-purple)" strokeWidth={2.5} dot={false} name="RTP" />
                    {/* Target RTP line */}
                    <Line type="monotone" dataKey={() => 85} stroke="var(--brand-lavender)" strokeWidth={1.5} strokeDasharray="5 5" dot={false} name="Target (85%)" />
                  </LineChart>
                  </ResponsiveContainer>
                </div>
              </div>
            </>
          )}

          {mcmcHistory.length === 0 && (
            <div className="flex-1 bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl p-6 border border-[var(--brand-tan)]/20 flex flex-col items-center justify-center text-center">
              <div className="text-[var(--brand-lavender)] mb-3">
                <svg className="w-12 h-12 mx-auto opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
              </div>
              <p className="text-[var(--brand-lavender)] text-sm">Draw a pattern and run the simulation to see MCMC adaptation in real-time</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default PatternDrawingDemo;
