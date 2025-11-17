import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, ReferenceLine } from 'recharts';
import { motion } from 'framer-motion';

interface PayoutCurveChartProps {
  pmax: number;
  breakevenRadius: number;
  targetRadius: number; // d_max in yards
  k: number; // Curve steepness
  lastShotDistance?: number;
  lastShotMultiplier?: number;
}

export default function PayoutCurveChart({
  pmax,
  breakevenRadius,
  targetRadius,
  k,
  lastShotDistance,
  lastShotMultiplier,
}: PayoutCurveChartProps) {
  // Validate P_max before rendering
  const isValidPmax = pmax > 0 && !isNaN(pmax) && isFinite(pmax);

  if (!isValidPmax) {
    return (
      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-2xl border border-white/10 p-4 h-full flex flex-col"
      >
        <h3 className="text-xs font-semibold text-white/70 uppercase tracking-wider mb-3">Payout Curve</h3>
        <div className="flex-1 flex items-center justify-center text-white/70 text-sm">
          <p>⚠️ Take shots to see curve</p>
        </div>
      </motion.div>
    );
  }

  // Payout formula: P_max * (1 - d/d_max)^k
  // This matches the Rust implementation
  const calculatePayoutMultiplier = (distanceYards: number) => {
    const dMaxFt = targetRadius * 3; // Convert target radius from yards to feet
    const distanceFt = distanceYards * 3; // Convert distance to feet

    // If beyond target radius, no payout
    if (distanceFt >= dMaxFt) {
      return 0;
    }

    // Rust formula: P_max * (1 - d/d_max)^k
    const payoutFactor = Math.pow(1 - distanceFt / dMaxFt, k);
    const multiplier = pmax * payoutFactor;

    return Math.max(0, multiplier);
  };

  // Generate payout curve data
  const generateCurveData = () => {
    const data = [];
    const steps = 100;

    for (let i = 0; i <= steps; i++) {
      const distanceYards = (i / steps) * targetRadius;
      const multiplier = calculatePayoutMultiplier(distanceYards);
      data.push({
        distance: distanceYards,
        multiplier: multiplier,
        breakeven: 1.0,
        isLastShot: false,
      });
    }

    // Add last shot point to the curve data if available
    if (lastShotDistance !== undefined && lastShotMultiplier !== undefined) {
      // Find the closest data point and mark it (or insert it)
      const closestIndex = Math.round((lastShotDistance / targetRadius) * steps);

      // Insert the actual shot data at the correct position
      if (closestIndex >= 0 && closestIndex <= steps) {
        data.splice(closestIndex, 0, {
          distance: lastShotDistance,
          multiplier: lastShotMultiplier,
          breakeven: 1.0,
          isLastShot: true,
        });
      }
    }

    return data;
  };

  const data = generateCurveData();

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-2xl border border-white/10 p-4 h-full flex flex-col"
    >
      <h3 className="text-xs font-semibold text-white/70 uppercase tracking-wider mb-3">Payout Curve</h3>
      <div className="flex-1 min-h-0">
        <ResponsiveContainer width="100%" height="100%">
        <LineChart data={data} margin={{ top: 5, right: 10, left: 0, bottom: 5 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#493b7c" opacity={0.1} />
          <XAxis
            dataKey="distance"
            stroke="#493b7c"
            tick={{ fontSize: 10, fill: '#dfc9ad' }}
            label={{ value: 'Distance (y)', position: 'insideBottom', offset: -3, fill: '#dfc9ad', fontSize: 10 }}
          />
          <YAxis
            stroke="#493b7c"
            tick={{ fontSize: 10, fill: '#dfc9ad' }}
            label={{ value: 'Mult', angle: -90, position: 'insideLeft', fill: '#dfc9ad', fontSize: 10 }}
            domain={[0, 'auto']}
            tickCount={6}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: 'rgba(0, 0, 0, 0.9)',
              border: '1px solid rgba(255, 255, 255, 0.1)',
              borderRadius: '12px',
              color: '#ffffff',
              padding: '8px 12px',
            }}
            formatter={(value: number, name: string) => {
              if (name === 'multiplier') return [`${value.toFixed(2)}x`, 'Payout'];
              if (name === 'breakeven') return ['1.00x', 'Breakeven'];
              return [value, name];
            }}
          />

          {/* Breakeven line */}
          <ReferenceLine
            y={1.0}
            stroke="var(--brand-tan)"
            strokeDasharray="5 5"
            strokeWidth={1.5}
            label={{ value: 'Breakeven', position: 'right', fill: 'var(--brand-tan)', fontSize: 10 }}
          />

          {/* Breakeven radius vertical line */}
          <ReferenceLine
            x={breakevenRadius}
            stroke="#493b7c"
            strokeDasharray="3 3"
            strokeWidth={1}
            opacity={0.4}
            label={{ value: `${breakevenRadius.toFixed(1)}y`, position: 'top', fill: '#dfc9ad', fontSize: 10 }}
          />

          {/* Payout curve */}
          <Line
            type="monotone"
            dataKey="multiplier"
            stroke="var(--brand-bright-purple)"
            strokeWidth={2.5}
            dot={false}
            activeDot={{ r: 5, fill: 'var(--brand-bright-purple)', strokeWidth: 2, stroke: 'var(--brand-lavender)' }}
          />
        </LineChart>
      </ResponsiveContainer>
      </div>
    </motion.div>
  );
}
