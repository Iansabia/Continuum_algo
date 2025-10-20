import { useState } from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

export default function VenueSimulator() {
  const [numBays, setNumBays] = useState(50);
  const [hours, setHours] = useState(8);
  const [shotsPerHour, setShotsPerHour] = useState(100);
  const [isSimulating, setIsSimulating] = useState(false);
  const [results, setResults] = useState<any>(null);

  const runSimulation = async () => {
    setIsSimulating(true);

    try {
      // TODO: Load WASM and run venue simulation
      // Placeholder data
      const totalShots = numBays * hours * shotsPerHour;
      const placeholderData = {
        total_wagered: totalShots * 12.5,
        total_payouts: totalShots * 10.5,
        net_profit: totalShots * 2.0,
        hold_percentage: 16.0,
        profit_by_hour: Array.from({ length: Math.floor(hours) }, (_, i) => ({
          hour: i + 1,
          profit: (totalShots / hours) * 2.0 * (1 + Math.random() * 0.2),
        })),
      };

      setResults(placeholderData);
    } catch (error) {
      console.error('Simulation error:', error);
      alert('Simulation failed. WASM module not yet compiled.');
    } finally {
      setIsSimulating(false);
    }
  };

  return (
    <div className="bg-gray-800/50 backdrop-blur-sm rounded-lg p-8 border border-gray-700">
      <h2 className="text-3xl font-montserrat font-bold text-golf-gold mb-6">
        Venue Economics Simulator
      </h2>

      {/* Controls */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Number of Bays: {numBays}
          </label>
          <input
            type="range"
            min="10"
            max="100"
            step="5"
            value={numBays}
            onChange={(e) => setNumBays(Number(e.target.value))}
            className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Operating Hours: {hours}
          </label>
          <input
            type="range"
            min="1"
            max="24"
            value={hours}
            onChange={(e) => setHours(Number(e.target.value))}
            className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Shots/Bay/Hour: {shotsPerHour}
          </label>
          <input
            type="range"
            min="50"
            max="150"
            step="10"
            value={shotsPerHour}
            onChange={(e) => setShotsPerHour(Number(e.target.value))}
            className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
          />
        </div>
      </div>

      <button
        onClick={runSimulation}
        disabled={isSimulating}
        className="w-full bg-golf-gold text-golf-navy font-montserrat font-bold py-4 px-8 rounded-lg
                   hover:bg-yellow-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isSimulating ? 'Running Simulation...' : 'Run Venue Simulation'}
      </button>

      {/* Results */}
      {results && (
        <div className="mt-8 space-y-6">
          {/* Summary Stats */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="bg-gray-900/50 p-4 rounded-lg">
              <div className="text-sm text-gray-400">Total Handle</div>
              <div className="text-2xl font-bold text-white">${results.total_wagered.toLocaleString()}</div>
            </div>
            <div className="bg-gray-900/50 p-4 rounded-lg">
              <div className="text-sm text-gray-400">Total Payouts</div>
              <div className="text-2xl font-bold text-red-400">${results.total_payouts.toLocaleString()}</div>
            </div>
            <div className="bg-gray-900/50 p-4 rounded-lg">
              <div className="text-sm text-gray-400">Net Profit</div>
              <div className="text-2xl font-bold text-green-400">${results.net_profit.toLocaleString()}</div>
            </div>
            <div className="bg-gray-900/50 p-4 rounded-lg">
              <div className="text-sm text-gray-400">Hold %</div>
              <div className="text-2xl font-bold text-golf-gold">{results.hold_percentage.toFixed(1)}%</div>
            </div>
          </div>

          {/* Hourly Profit Chart */}
          <div className="bg-gray-900/50 p-6 rounded-lg">
            <h3 className="text-xl font-montserrat font-semibold text-golf-gold mb-4">
              Profit by Hour
            </h3>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={results.profit_by_hour}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis dataKey="hour" stroke="#9CA3AF" />
                <YAxis stroke="#9CA3AF" />
                <Tooltip
                  contentStyle={{ backgroundColor: '#1F2937', border: '1px solid #374151' }}
                  labelStyle={{ color: '#D4AF37' }}
                />
                <Bar dataKey="profit" fill="#D4AF37" />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>
      )}
    </div>
  );
}
