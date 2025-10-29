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
        className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl border border-[var(--brand-tan)]/20 p-4 h-full flex flex-col"
      >
        <h3 className="text-xs font-semibold text-[var(--brand-lavender)] uppercase tracking-wider mb-3">P_max Evolution</h3>
        <div className="flex-1 flex items-center justify-center text-[var(--brand-lavender)] text-sm">
          <p>Take shots to see evolution</p>
        </div>
      </motion.div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl border border-[var(--brand-tan)]/20 p-4 h-full flex flex-col"
    >
      <h3 className="text-xs font-semibold text-[var(--brand-lavender)] uppercase tracking-wider mb-3">P_max Evolution</h3>
      <div className="flex-1 min-h-0">
        <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={history} margin={{ top: 5, right: 15, left: 0, bottom: 5 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--brand-lavender)" opacity={0.1} />
          <XAxis
            dataKey="shotNumber"
            stroke="var(--brand-lavender)"
            tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
            label={{ value: 'Shot #', position: 'insideBottom', offset: -3, fill: 'var(--brand-lavender)', fontSize: 10 }}
          />
          <YAxis
            yAxisId="left"
            stroke="var(--brand-bright-purple)"
            tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
            label={{ value: 'P_max', angle: -90, position: 'insideLeft', fill: 'var(--brand-bright-purple)', fontSize: 10 }}
            domain={[1, 'auto']}
          />
          <YAxis
            yAxisId="right"
            orientation="right"
            stroke="var(--brand-dark-gold)"
            tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
            label={{ value: 'Conf%', angle: 90, position: 'insideRight', fill: 'var(--brand-dark-gold)', fontSize: 10 }}
            domain={[0, 100]}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: 'var(--brand-deep-purple)',
              border: '1px solid var(--brand-tan)',
              borderRadius: '12px',
              color: 'var(--brand-tan)',
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
            activeDot={{ r: 5, fill: 'var(--brand-bright-purple)', strokeWidth: 2, stroke: 'var(--brand-lavender)' }}
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
