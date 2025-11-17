import { Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Area, ComposedChart } from 'recharts';
import { motion } from 'framer-motion';

interface PmaxDataPoint {
  shotNumber: number;
  pmax: number;
  confidence: number;
  sigma: number;
}

interface PmaxHistoryChartProps {
  history: PmaxDataPoint[];
}

export default function PmaxHistoryChart({ history }: PmaxHistoryChartProps) {
  if (history.length === 0) {
    return (
      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-2xl border border-white/10 p-4 h-full flex flex-col"
      >
        <h3 className="text-xs font-semibold text-white/70 uppercase tracking-wider mb-3">P_max Evolution</h3>
        <div className="flex-1 flex items-center justify-center text-white/70 text-sm">
          <p>Take shots to see evolution</p>
        </div>
      </motion.div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-2xl border border-white/10 p-4 h-full flex flex-col"
    >
      <h3 className="text-xs font-semibold text-white/70 uppercase tracking-wider mb-3">P_max Evolution</h3>
      <div className="flex-1 min-h-0">
        <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={history} margin={{ top: 5, right: 15, left: 0, bottom: 5 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#493b7c" opacity={0.1} />
          <XAxis
            dataKey="shotNumber"
            stroke="#dfc9ad"
            tick={{ fontSize: 10, fill: '#dfc9ad' }}
            label={{ value: 'Shot #', position: 'insideBottom', offset: -3, fill: '#dfc9ad', fontSize: 10 }}
          />
          <YAxis
            yAxisId="left"
            stroke="var(--brand-bright-purple)"
            tick={{ fontSize: 10, fill: '#dfc9ad' }}
            label={{ value: 'P_max', angle: -90, position: 'insideLeft', fill: 'var(--brand-bright-purple)', fontSize: 10 }}
            domain={[1, 'auto']}
          />
          <YAxis
            yAxisId="right"
            orientation="right"
            stroke="var(--brand-dark-gold)"
            tick={{ fontSize: 10, fill: '#dfc9ad' }}
            label={{ value: 'Conf%', angle: 90, position: 'insideRight', fill: 'var(--brand-dark-gold)', fontSize: 10 }}
            domain={[0, 100]}
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
              if (name === 'pmax') return [`${value.toFixed(3)}`, 'P_max'];
              if (name === 'confidence') return [`${value.toFixed(1)}%`, 'Confidence'];
              if (name === 'sigma') return [`${value.toFixed(2)}y`, 'Sigma'];
              return [value, name];
            }}
          />
          {/* Confidence area (background) */}
          <Area
            yAxisId="right"
            type="monotone"
            dataKey="confidence"
            fill="var(--brand-dark-gold)"
            fillOpacity={0.1}
            stroke="none"
          />

          {/* P_max line */}
          <Line
            yAxisId="left"
            type="monotone"
            dataKey="pmax"
            stroke="var(--brand-bright-purple)"
            strokeWidth={2.5}
            dot={{ fill: 'var(--brand-bright-purple)', r: 2 }}
            activeDot={{ r: 5, fill: 'var(--brand-bright-purple)', strokeWidth: 2, stroke: '#dfc9ad' }}
          />

          {/* Confidence line */}
          <Line
            yAxisId="right"
            type="monotone"
            dataKey="confidence"
            stroke="var(--brand-dark-gold)"
            strokeWidth={2}
            dot={false}
            strokeDasharray="4 4"
          />
        </ComposedChart>
      </ResponsiveContainer>
      </div>
    </motion.div>
  );
}
