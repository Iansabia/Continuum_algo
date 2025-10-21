import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend, Area, ComposedChart } from 'recharts';

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
      <div className="bg-gray-800 p-4 rounded-lg border-2 border-brand-deep-purple">
        <h3 className="text-lg font-semibold text-brand-tan mb-4">P_max Evolution</h3>
        <div className="h-64 flex items-center justify-center text-gray-500">
          <p>No data yet. Take some shots to see skill evolution!</p>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-gray-800 p-4 rounded-lg border-2 border-brand-deep-purple">
      <h3 className="text-lg font-semibold text-brand-tan mb-4">P_max Evolution</h3>
      <ResponsiveContainer width="100%" height={300}>
        <ComposedChart data={history} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
          <XAxis
            dataKey="shotNumber"
            stroke="#9CA3AF"
            label={{ value: 'Shot Number', position: 'insideBottom', offset: -5, fill: '#9CA3AF' }}
          />
          <YAxis
            yAxisId="left"
            stroke="#604c9c"
            label={{ value: 'P_max', angle: -90, position: 'insideLeft', fill: '#604c9c' }}
            domain={[1, 'auto']}
          />
          <YAxis
            yAxisId="right"
            orientation="right"
            stroke="#9e8cb4"
            label={{ value: 'Confidence (%)', angle: 90, position: 'insideRight', fill: '#9e8cb4' }}
            domain={[0, 100]}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: '#1F2937',
              border: '1px solid #604c9c',
              borderRadius: '8px',
              color: '#dfc9ad',
            }}
            formatter={(value: number, name: string) => {
              if (name === 'pmax') return [`${value.toFixed(3)}`, 'P_max'];
              if (name === 'confidence') return [`${value.toFixed(1)}%`, 'Confidence'];
              if (name === 'sigma') return [`${value.toFixed(2)}y`, 'Sigma'];
              return [value, name];
            }}
          />
          <Legend
            wrapperStyle={{ paddingTop: '10px' }}
            formatter={(value) => {
              if (value === 'pmax') return 'P_max';
              if (value === 'confidence') return 'Confidence';
              return value;
            }}
          />

          {/* Confidence area (background) */}
          <Area
            yAxisId="right"
            type="monotone"
            dataKey="confidence"
            fill="#9e8cb4"
            fillOpacity={0.2}
            stroke="none"
          />

          {/* P_max line */}
          <Line
            yAxisId="left"
            type="monotone"
            dataKey="pmax"
            stroke="#604c9c"
            strokeWidth={3}
            dot={{ fill: '#604c9c', r: 4 }}
            activeDot={{ r: 6 }}
          />

          {/* Confidence line */}
          <Line
            yAxisId="right"
            type="monotone"
            dataKey="confidence"
            stroke="#9e8cb4"
            strokeWidth={2}
            dot={{ fill: '#9e8cb4', r: 3 }}
            activeDot={{ r: 5 }}
            strokeDasharray="5 5"
          />
        </ComposedChart>
      </ResponsiveContainer>

      {/* Stats summary */}
      <div className="mt-4 grid grid-cols-3 gap-4 text-center text-sm">
        <div>
          <p className="text-gray-500">Current P_max</p>
          <p className="text-brand-bright-purple font-semibold text-lg">
            {history[history.length - 1]?.pmax.toFixed(3) || 'N/A'}
          </p>
        </div>
        <div>
          <p className="text-gray-500">Confidence</p>
          <p className="text-brand-lavender font-semibold text-lg">
            {history[history.length - 1]?.confidence.toFixed(1) || 'N/A'}%
          </p>
        </div>
        <div>
          <p className="text-gray-500">Sigma (σ)</p>
          <p className="text-brand-tan font-semibold text-lg">
            {history[history.length - 1]?.sigma.toFixed(2) || 'N/A'}y
          </p>
        </div>
      </div>
    </div>
  );
}
