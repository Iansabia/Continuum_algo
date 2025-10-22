import { Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Area, ComposedChart } from 'recharts';

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
      <div className="bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/30 shadow-lg h-full flex flex-col">
        <h3 className="text-xs font-medium text-[#9e8cb4] mb-2">P_max Evolution</h3>
        <div className="flex-1 flex items-center justify-center text-[#9e8cb4]/60 text-xs">
          <p>Take shots to see evolution</p>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/30 shadow-lg h-full flex flex-col">
      <h3 className="text-xs font-medium text-[#9e8cb4] mb-2">P_max Evolution</h3>
      <div className="flex-1 min-h-0">
        <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={history} margin={{ top: 5, right: 15, left: 0, bottom: 5 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="rgba(158,140,180,0.1)" />
          <XAxis
            dataKey="shotNumber"
            stroke="rgba(158,140,180,0.5)"
            tick={{ fontSize: 10, fill: 'rgba(158,140,180,0.7)' }}
            label={{ value: 'Shot #', position: 'insideBottom', offset: -3, fill: 'rgba(158,140,180,0.7)', fontSize: 10 }}
          />
          <YAxis
            yAxisId="left"
            stroke="#604c9c"
            tick={{ fontSize: 10, fill: 'rgba(158,140,180,0.7)' }}
            label={{ value: 'P_max', angle: -90, position: 'insideLeft', fill: '#604c9c', fontSize: 10 }}
            domain={[1, 'auto']}
          />
          <YAxis
            yAxisId="right"
            orientation="right"
            stroke="#9e8cb4"
            tick={{ fontSize: 10, fill: 'rgba(158,140,180,0.7)' }}
            label={{ value: 'Conf%', angle: 90, position: 'insideRight', fill: '#9e8cb4', fontSize: 10 }}
            domain={[0, 100]}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: 'rgba(73,59,124,0.9)',
              border: '1px solid rgba(158,140,180,0.3)',
              borderRadius: '8px',
              color: 'rgba(223,201,173,1)',
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
            fill="#9e8cb4"
            fillOpacity={0.15}
            stroke="none"
          />

          {/* P_max line */}
          <Line
            yAxisId="left"
            type="monotone"
            dataKey="pmax"
            stroke="#604c9c"
            strokeWidth={2}
            dot={{ fill: '#604c9c', r: 2 }}
            activeDot={{ r: 4, fill: '#604c9c' }}
          />

          {/* Confidence line */}
          <Line
            yAxisId="right"
            type="monotone"
            dataKey="confidence"
            stroke="#9e8cb4"
            strokeWidth={1.5}
            dot={false}
            strokeDasharray="3 3"
          />
        </ComposedChart>
      </ResponsiveContainer>
      </div>
    </div>
  );
}
