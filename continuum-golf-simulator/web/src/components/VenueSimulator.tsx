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
    <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl p-8 border border-[var(--brand-tan)]/20">
      <h2 className="text-3xl font-semibold text-[var(--brand-tan)] mb-6">
        Venue Economics Simulator
      </h2>

      {/* Controls */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        <div>
          <label className="block text-sm font-medium text-[var(--brand-lavender)] mb-2">
            Number of Bays: {numBays}
          </label>
          <input
            type="range"
            min="10"
            max="100"
            step="5"
            value={numBays}
            onChange={(e) => setNumBays(Number(e.target.value))}
            className="w-full h-2 bg-[var(--brand-dark-gray)] rounded-lg appearance-none cursor-pointer slider-thumb"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-[var(--brand-lavender)] mb-2">
            Operating Hours: {hours}
          </label>
          <input
            type="range"
            min="1"
            max="24"
            value={hours}
            onChange={(e) => setHours(Number(e.target.value))}
            className="w-full h-2 bg-[var(--brand-dark-gray)] rounded-lg appearance-none cursor-pointer slider-thumb"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-[var(--brand-lavender)] mb-2">
            Shots/Bay/Hour: {shotsPerHour}
          </label>
          <input
            type="range"
            min="50"
            max="150"
            step="10"
            value={shotsPerHour}
            onChange={(e) => setShotsPerHour(Number(e.target.value))}
            className="w-full h-2 bg-[var(--brand-dark-gray)] rounded-lg appearance-none cursor-pointer slider-thumb"
          />
        </div>
      </div>

      <button
        onClick={runSimulation}
        disabled={isSimulating}
        className="w-full bg-gradient-to-r from-[var(--brand-bright-purple)] to-[var(--brand-deep-purple)] hover:from-[var(--brand-deep-purple)] hover:to-[var(--brand-bright-purple)] text-[var(--brand-tan)] font-semibold py-4 px-8 rounded-xl transition-all disabled:opacity-50 disabled:cursor-not-allowed shadow-lg shadow-[var(--brand-bright-purple)]/25"
      >
        {isSimulating ? 'Running Simulation...' : 'Run Venue Simulation'}
      </button>

      {/* Results */}
      {results && (
        <div className="mt-8 space-y-6">
          {/* Summary Stats */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="bg-[var(--brand-deep-purple)]/10 p-4 rounded-xl border border-[var(--brand-tan)]/10">
              <div className="text-sm text-[var(--brand-lavender)]">Total Handle</div>
              <div className="text-2xl font-bold text-[var(--brand-tan)]">${results.total_wagered.toLocaleString()}</div>
            </div>
            <div className="bg-[var(--brand-deep-purple)]/10 p-4 rounded-xl border border-[var(--brand-tan)]/10">
              <div className="text-sm text-[var(--brand-lavender)]">Total Payouts</div>
              <div className="text-2xl font-bold text-[var(--brand-rose-copper)]">${results.total_payouts.toLocaleString()}</div>
            </div>
            <div className="bg-[var(--brand-deep-purple)]/10 p-4 rounded-xl border border-[var(--brand-tan)]/10">
              <div className="text-sm text-[var(--brand-lavender)]">Net Profit</div>
              <div className="text-2xl font-bold text-[var(--brand-tan)]">${results.net_profit.toLocaleString()}</div>
            </div>
            <div className="bg-[var(--brand-deep-purple)]/10 p-4 rounded-xl border border-[var(--brand-tan)]/10">
              <div className="text-sm text-[var(--brand-lavender)]">Hold %</div>
              <div className="text-2xl font-bold text-[var(--brand-dark-gold)]">{results.hold_percentage.toFixed(1)}%</div>
            </div>
          </div>

          {/* Hourly Profit Chart */}
          <div className="bg-[var(--brand-deep-purple)]/10 p-6 rounded-xl border border-[var(--brand-tan)]/10">
            <h3 className="text-xl font-semibold text-[var(--brand-tan)] mb-4">
              Profit by Hour
            </h3>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={results.profit_by_hour}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--brand-lavender)" opacity={0.2} />
                <XAxis dataKey="hour" stroke="var(--brand-lavender)" />
                <YAxis stroke="var(--brand-lavender)" />
                <Tooltip
                  contentStyle={{ backgroundColor: 'var(--brand-deep-purple)', border: '1px solid var(--brand-tan)', borderRadius: '8px', opacity: 0.95 }}
                  labelStyle={{ color: 'var(--brand-tan)' }}
                />
                <Bar dataKey="profit" fill="var(--brand-dark-gold)" />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>
      )}
    </div>
  );
}
