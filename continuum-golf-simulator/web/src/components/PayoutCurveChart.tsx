import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, ReferenceLine, ReferenceDot } from 'recharts';

interface PayoutCurveChartProps {
  pmax: number;
  breakevenRadius: number;
  lastShotDistance?: number;
  lastShotMultiplier?: number;
}

export default function PayoutCurveChart({
  pmax,
  breakevenRadius,
  lastShotDistance,
  lastShotMultiplier,
}: PayoutCurveChartProps) {
  // Validate P_max before rendering
  const isValidPmax = pmax > 0 && !isNaN(pmax) && isFinite(pmax);

  if (!isValidPmax) {
    return (
      <div className="bg-gray-800 p-4 rounded-lg border-2 border-brand-deep-purple">
        <h3 className="text-lg font-semibold text-brand-tan mb-4">Payout Curve</h3>
        <div className="h-64 flex items-center justify-center text-gray-500">
          <p>⚠️ Invalid P_max value. Take some shots to see the payout curve!</p>
        </div>
      </div>
    );
  }

  // Generate payout curve data
  const generateCurveData = () => {
    const data = [];
    const dMaxFt = 30; // Maximum target radius in feet
    const maxDistanceYards = dMaxFt / 3; // Convert to yards for display
    const steps = 100;

    for (let i = 0; i <= steps; i++) {
      const distanceYards = (i / steps) * maxDistanceYards;
      const multiplier = calculatePayoutMultiplier(distanceYards);
      data.push({
        distance: distanceYards,
        multiplier: multiplier,
        breakeven: 1.0,
      });
    }

    return data;
  };

  // Payout formula: P_max * (1 - d/d_max)^k
  // This matches the Rust implementation
  const calculatePayoutMultiplier = (distanceYards: number) => {
    const dMaxFt = 30; // Maximum target radius in feet
    const k = 5.0; // Curve steepness
    const distanceFt = distanceYards * 3; // Convert to feet

    // If beyond target radius, no payout
    if (distanceFt >= dMaxFt) {
      return 0;
    }

    // Rust formula: P_max * (1 - d/d_max)^k
    const payoutFactor = Math.pow(1 - distanceFt / dMaxFt, k);
    const multiplier = pmax * payoutFactor;

    return Math.max(0, multiplier);
  };

  const data = generateCurveData();

  return (
    <div className="bg-gray-800 p-4 rounded-lg border-2 border-brand-deep-purple">
      <h3 className="text-lg font-semibold text-brand-tan mb-4">Payout Curve</h3>
      <ResponsiveContainer width="100%" height={300}>
        <LineChart data={data} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
          <XAxis
            dataKey="distance"
            stroke="#9CA3AF"
            label={{ value: 'Miss Distance (yards)', position: 'insideBottom', offset: -5, fill: '#9CA3AF' }}
          />
          <YAxis
            stroke="#9CA3AF"
            label={{ value: 'Payout Multiplier', angle: -90, position: 'insideLeft', fill: '#9CA3AF' }}
            domain={[0, Math.max(2, pmax * 1.1)]}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: '#1F2937',
              border: '1px solid #604c9c',
              borderRadius: '8px',
              color: '#dfc9ad',
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
            stroke="#7e6649"
            strokeDasharray="5 5"
            strokeWidth={2}
            label={{ value: 'Breakeven', position: 'right', fill: '#7e6649' }}
          />

          {/* Breakeven radius vertical line */}
          <ReferenceLine
            x={breakevenRadius}
            stroke="#7e6649"
            strokeDasharray="3 3"
            strokeWidth={1}
            label={{ value: `${breakevenRadius.toFixed(1)}y`, position: 'top', fill: '#7e6649' }}
          />

          {/* Payout curve */}
          <Line
            type="monotone"
            dataKey="multiplier"
            stroke="#604c9c"
            strokeWidth={3}
            dot={false}
            activeDot={{ r: 6 }}
          />

          {/* Last shot marker */}
          {lastShotDistance !== undefined && lastShotMultiplier !== undefined && (
            <ReferenceDot
              x={lastShotDistance}
              y={lastShotMultiplier}
              r={8}
              fill={lastShotMultiplier >= 1.0 ? '#10B981' : '#EF4444'}
              stroke="#ffffff"
              strokeWidth={2}
            />
          )}
        </LineChart>
      </ResponsiveContainer>

      {/* Legend */}
      <div className="mt-4 flex justify-center gap-6 text-sm">
        <div className="flex items-center gap-2">
          <div className="w-4 h-1 bg-brand-bright-purple"></div>
          <span className="text-gray-400">Payout Multiplier</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="w-4 h-1 bg-brand-dark-gold border-dashed"></div>
          <span className="text-gray-400">Breakeven (1.0x)</span>
        </div>
        {lastShotDistance !== undefined && (
          <div className="flex items-center gap-2">
            <div className={`w-3 h-3 rounded-full ${lastShotMultiplier && lastShotMultiplier >= 1.0 ? 'bg-green-500' : 'bg-red-500'}`}></div>
            <span className="text-gray-400">Last Shot</span>
          </div>
        )}
      </div>
    </div>
  );
}
