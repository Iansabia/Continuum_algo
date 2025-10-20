import { useState } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

export default function PlayerSimulator() {
  const [handicap, setHandicap] = useState(15);
  const [numShots, setNumShots] = useState(100);
  const [wagerMin, setWagerMin] = useState(5);
  const [wagerMax, setWagerMax] = useState(20);
  const [isSimulating, setIsSimulating] = useState(false);
  const [results, setResults] = useState<any>(null);

  const runSimulation = async () => {
    setIsSimulating(true);

    try {
      // TODO: Load WASM and run simulation
      // const wasm = await import('../../pkg/continuum_golf_simulator');
      // const result = wasm.simulate_player_session(handicap, numShots, wagerMin, wagerMax, null);

      // Placeholder data for now
      const placeholderData = {
        total_wagered: 1250.0,
        total_won: 1050.0,
        net_gain_loss: -200.0,
        session_house_edge: 16.0,
        shots: Array.from({ length: numShots }, (_, i) => ({
          shot_number: i + 1,
          cumulative_net: -5 * i + Math.random() * 50,
        })),
        final_skills: [
          { category: 'Wedge', sigma: 42.3, confidence: 78.2, p_max_current: 8.5 },
          { category: 'MidIron', sigma: 58.1, confidence: 65.4, p_max_current: 7.2 },
          { category: 'LongIron', sigma: 81.7, confidence: 51.3, p_max_current: 6.1 },
        ],
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
        Player Session Simulator
      </h2>

      {/* Controls */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Handicap: {handicap}
          </label>
          <input
            type="range"
            min="0"
            max="30"
            value={handicap}
            onChange={(e) => setHandicap(Number(e.target.value))}
            className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Number of Shots: {numShots}
          </label>
          <input
            type="range"
            min="10"
            max="1000"
            step="10"
            value={numShots}
            onChange={(e) => setNumShots(Number(e.target.value))}
            className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Min Wager: ${wagerMin}
          </label>
          <input
            type="range"
            min="1"
            max="50"
            value={wagerMin}
            onChange={(e) => setWagerMin(Number(e.target.value))}
            className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-300 mb-2">
            Max Wager: ${wagerMax}
          </label>
          <input
            type="range"
            min="5"
            max="100"
            value={wagerMax}
            onChange={(e) => setWagerMax(Number(e.target.value))}
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
        {isSimulating ? 'Running Simulation...' : 'Run Simulation'}
      </button>

      {/* Results */}
      {results && (
        <div className="mt-8 space-y-6">
          {/* Summary Stats */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div className="bg-gray-900/50 p-4 rounded-lg">
              <div className="text-sm text-gray-400">Total Wagered</div>
              <div className="text-2xl font-bold text-white">${results.total_wagered.toFixed(2)}</div>
            </div>
            <div className="bg-gray-900/50 p-4 rounded-lg">
              <div className="text-sm text-gray-400">Total Won</div>
              <div className="text-2xl font-bold text-green-400">${results.total_won.toFixed(2)}</div>
            </div>
            <div className="bg-gray-900/50 p-4 rounded-lg">
              <div className="text-sm text-gray-400">Net P/L</div>
              <div className={`text-2xl font-bold ${results.net_gain_loss >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                ${results.net_gain_loss.toFixed(2)}
              </div>
            </div>
            <div className="bg-gray-900/50 p-4 rounded-lg">
              <div className="text-sm text-gray-400">House Edge</div>
              <div className="text-2xl font-bold text-golf-gold">{results.session_house_edge.toFixed(1)}%</div>
            </div>
          </div>

          {/* Cumulative P/L Chart */}
          <div className="bg-gray-900/50 p-6 rounded-lg">
            <h3 className="text-xl font-montserrat font-semibold text-golf-gold mb-4">
              Cumulative Profit/Loss
            </h3>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={results.shots}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis dataKey="shot_number" stroke="#9CA3AF" />
                <YAxis stroke="#9CA3AF" />
                <Tooltip
                  contentStyle={{ backgroundColor: '#1F2937', border: '1px solid #374151' }}
                  labelStyle={{ color: '#D4AF37' }}
                />
                <Legend />
                <Line
                  type="monotone"
                  dataKey="cumulative_net"
                  stroke="#D4AF37"
                  strokeWidth={2}
                  name="Cumulative P/L ($)"
                />
              </LineChart>
            </ResponsiveContainer>
          </div>

          {/* Skill Profiles */}
          <div className="bg-gray-900/50 p-6 rounded-lg">
            <h3 className="text-xl font-montserrat font-semibold text-golf-gold mb-4">
              Final Skill Profiles
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {results.final_skills.map((skill: any) => (
                <div key={skill.category} className="bg-gray-800 p-4 rounded-lg">
                  <div className="text-lg font-semibold text-white mb-2">{skill.category}</div>
                  <div className="space-y-1 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-400">Dispersion (σ):</span>
                      <span className="text-white">{skill.sigma.toFixed(1)} ft</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">Confidence:</span>
                      <span className="text-green-400">{skill.confidence.toFixed(1)}%</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">P_max:</span>
                      <span className="text-golf-gold">{skill.p_max_current.toFixed(1)}×</span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
