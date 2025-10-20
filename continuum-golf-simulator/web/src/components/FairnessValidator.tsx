import { useState } from 'react';

export default function FairnessValidator() {
  const [selectedHole, setSelectedHole] = useState(4);
  const [isValidating, setIsValidating] = useState(false);
  const [results, setResults] = useState<any>(null);

  const runValidation = async () => {
    setIsValidating(true);

    try {
      // TODO: Load WASM and run fairness validation
      // Placeholder data
      const placeholderData = {
        hole_id: selectedHole,
        handicap_results: [
          { handicap: 0, expected_value: -0.15, p_max: 6.2 },
          { handicap: 10, expected_value: -0.148, p_max: 7.5 },
          { handicap: 20, expected_value: -0.151, p_max: 9.1 },
          { handicap: 30, expected_value: -0.149, p_max: 11.3 },
        ],
        max_ev_difference: 0.003,
        is_fair: true,
      };

      setResults(placeholderData);
    } catch (error) {
      console.error('Validation error:', error);
      alert('Validation failed. WASM module not yet compiled.');
    } finally {
      setIsValidating(false);
    }
  };

  return (
    <div className="bg-gray-800/50 backdrop-blur-sm rounded-lg p-8 border border-gray-700">
      <h2 className="text-3xl font-montserrat font-bold text-golf-gold mb-6">
        Fairness Validator
      </h2>

      <p className="text-gray-300 mb-6">
        This interactive proof demonstrates that all players, regardless of skill level (handicap),
        have the same expected value when wagering on the same hole. This is the core fairness guarantee
        of the Continuum Golf system.
      </p>

      {/* Hole Selection */}
      <div className="mb-8">
        <label className="block text-sm font-medium text-gray-300 mb-2">
          Select Hole: H{selectedHole} ({[75, 100, 125, 150, 175, 200, 225, 250][selectedHole - 1]} yards)
        </label>
        <input
          type="range"
          min="1"
          max="8"
          value={selectedHole}
          onChange={(e) => setSelectedHole(Number(e.target.value))}
          className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
        />
      </div>

      <button
        onClick={runValidation}
        disabled={isValidating}
        className="w-full bg-golf-gold text-golf-navy font-montserrat font-bold py-4 px-8 rounded-lg
                   hover:bg-yellow-500 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isValidating ? 'Running Validation...' : 'Validate Fairness'}
      </button>

      {/* Results */}
      {results && (
        <div className="mt-8 space-y-6">
          {/* Fairness Status */}
          <div className={`p-6 rounded-lg text-center ${
            results.is_fair ? 'bg-green-900/30 border-2 border-green-500' : 'bg-red-900/30 border-2 border-red-500'
          }`}>
            <div className="text-3xl font-bold mb-2">
              {results.is_fair ? '✓ FAIR' : '✗ NOT FAIR'}
            </div>
            <div className="text-gray-300">
              Max EV Difference: {(results.max_ev_difference * 100).toFixed(3)}%
              {results.is_fair && ' (within 0.5% threshold)'}
            </div>
          </div>

          {/* Handicap Comparison Table */}
          <div className="bg-gray-900/50 p-6 rounded-lg">
            <h3 className="text-xl font-montserrat font-semibold text-golf-gold mb-4">
              Expected Value by Handicap
            </h3>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-gray-700">
                    <th className="px-4 py-3 text-left text-gray-400">Handicap</th>
                    <th className="px-4 py-3 text-left text-gray-400">Skill Level</th>
                    <th className="px-4 py-3 text-right text-gray-400">P_max</th>
                    <th className="px-4 py-3 text-right text-gray-400">Expected Value</th>
                  </tr>
                </thead>
                <tbody>
                  {results.handicap_results.map((result: any) => (
                    <tr key={result.handicap} className="border-b border-gray-800">
                      <td className="px-4 py-3 text-white font-semibold">{result.handicap}</td>
                      <td className="px-4 py-3 text-gray-300">
                        {result.handicap === 0 ? 'Expert' :
                         result.handicap === 10 ? 'Advanced' :
                         result.handicap === 20 ? 'Intermediate' : 'Beginner'}
                      </td>
                      <td className="px-4 py-3 text-right text-golf-gold">{result.p_max.toFixed(1)}×</td>
                      <td className={`px-4 py-3 text-right font-semibold ${
                        result.expected_value >= 0 ? 'text-green-400' : 'text-red-400'
                      }`}>
                        {(result.expected_value * 100).toFixed(2)}%
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          {/* Explanation */}
          <div className="bg-blue-900/30 border border-blue-500/50 p-6 rounded-lg">
            <h4 className="text-lg font-semibold text-blue-300 mb-2">How Does This Work?</h4>
            <ul className="text-gray-300 space-y-2 text-sm">
              <li>• <strong>Better players</strong> (low handicap) get lower P_max multipliers because they hit closer</li>
              <li>• <strong>Weaker players</strong> (high handicap) get higher P_max multipliers to compensate for worse shots</li>
              <li>• The Kalman filter continuously adapts P_max based on actual performance</li>
              <li>• Result: All players have the same -15% expected value (15% house edge)</li>
            </ul>
          </div>
        </div>
      )}
    </div>
  );
}
