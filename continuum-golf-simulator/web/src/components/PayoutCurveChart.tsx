import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, ReferenceLine } from 'recharts';

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

  // Debug last shot values
  console.log('📍 PayoutCurveChart last shot:', {
    distance: lastShotDistance,
    multiplier: lastShotMultiplier,
    hasDistance: lastShotDistance !== undefined,
    hasMultiplier: lastShotMultiplier !== undefined,
  });

  if (!isValidPmax) {
    return (
      <div className="bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/30 shadow-lg h-full flex flex-col">
        <h3 className="text-xs font-medium text-[#9e8cb4] mb-2">Payout Curve</h3>
        <div className="flex-1 flex items-center justify-center text-[#9e8cb4]/60 text-xs">
          <p>⚠️ Take shots to see curve</p>
        </div>
      </div>
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
    <div className="bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/30 shadow-lg h-full flex flex-col">
      <h3 className="text-xs font-medium text-[#9e8cb4] mb-2">Payout Curve</h3>
      <div className="flex-1 min-h-0">
        <ResponsiveContainer width="100%" height="100%">
        <LineChart data={data} margin={{ top: 5, right: 10, left: 0, bottom: 5 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="rgba(158,140,180,0.1)" />
          <XAxis
            dataKey="distance"
            stroke="rgba(158,140,180,0.5)"
            tick={{ fontSize: 10, fill: 'rgba(158,140,180,0.7)' }}
            label={{ value: 'Distance (y)', position: 'insideBottom', offset: -3, fill: 'rgba(158,140,180,0.7)', fontSize: 10 }}
          />
          <YAxis
            stroke="rgba(158,140,180,0.5)"
            tick={{ fontSize: 10, fill: 'rgba(158,140,180,0.7)' }}
            label={{ value: 'Mult', angle: -90, position: 'insideLeft', fill: 'rgba(158,140,180,0.7)', fontSize: 10 }}
            domain={[0, 'auto']}
            tickCount={6}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: 'rgba(73,59,124,0.9)',
              border: '1px solid rgba(158,140,180,0.3)',
              borderRadius: '8px',
              color: 'rgba(223,201,173,1)',
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
            stroke="rgba(158,140,180,0.5)"
            strokeDasharray="5 5"
            strokeWidth={1.5}
            label={{ value: 'Breakeven', position: 'right', fill: 'rgba(158,140,180,0.7)' }}
          />

          {/* Breakeven radius vertical line */}
          <ReferenceLine
            x={breakevenRadius}
            stroke="rgba(158,140,180,0.3)"
            strokeDasharray="3 3"
            strokeWidth={1}
            label={{ value: `${breakevenRadius.toFixed(1)}y`, position: 'top', fill: 'rgba(158,140,180,0.7)' }}
          />

          {/* Payout curve */}
          <Line
            type="monotone"
            dataKey="multiplier"
            stroke="#604c9c"
            strokeWidth={2}
            dot={false}
            activeDot={{ r: 4, fill: '#604c9c' }}
          />
        </LineChart>
      </ResponsiveContainer>
      </div>
    </div>
  );
}
