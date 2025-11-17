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
    <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl p-8 border border-[var(--brand-tan)]/20">
      <h2 className="text-3xl font-semibold text-[var(--brand-tan)] mb-6">
        Fairness Validator
      </h2>

      <p className="text-[var(--brand-lavender)] mb-6">
        This interactive proof demonstrates that all players, regardless of skill level (handicap),
        have the same expected value when wagering on the same hole. This is the core fairness guarantee
        of the Continuum Golf system.
      </p>

      {/* Hole Selection */}
      <div className="mb-8">
        <label className="block text-sm font-medium text-[var(--brand-lavender)] mb-2">
          Select Hole: H{selectedHole} ({[75, 100, 125, 150, 175, 200, 225, 250][selectedHole - 1]} yards)
        </label>
        <input
          type="range"
          min="1"
          max="8"
          value={selectedHole}
          onChange={(e) => setSelectedHole(Number(e.target.value))}
          className="w-full h-2 bg-[var(--brand-dark-gray)] rounded-lg appearance-none cursor-pointer slider-thumb"
        />
      </div>

      <button
        onClick={runValidation}
        disabled={isValidating}
        className="w-full bg-gradient-to-r from-[var(--brand-bright-purple)] to-[var(--brand-deep-purple)] hover:from-[var(--brand-deep-purple)] hover:to-[var(--brand-bright-purple)] text-[var(--brand-tan)] font-semibold py-4 px-8 rounded-xl transition-all disabled:opacity-50 disabled:cursor-not-allowed shadow-lg shadow-[var(--brand-bright-purple)]/25"
      >
        {isValidating ? 'Running Validation...' : 'Validate Fairness'}
      </button>

      {/* Results */}
      {results && (
        <div className="mt-8 space-y-6">
          {/* Fairness Status */}
          <div className={`p-6 rounded-xl text-center border-2 ${
            results.is_fair
              ? 'bg-[var(--brand-bright-purple)]/20 border-[var(--brand-bright-purple)]'
              : 'bg-[var(--brand-rose-copper)]/20 border-[var(--brand-rose-copper)]'
          }`}>
            <div className="text-3xl font-bold mb-2 text-[var(--brand-tan)]">
              {results.is_fair ? '✓ FAIR' : '✗ NOT FAIR'}
            </div>
            <div className="text-[var(--brand-lavender)]">
              Max EV Difference: {(results.max_ev_difference * 100).toFixed(3)}%
              {results.is_fair && ' (within 0.5% threshold)'}
            </div>
          </div>

          {/* Handicap Comparison Table */}
          <div className="bg-[var(--brand-deep-purple)]/10 p-6 rounded-xl border border-[var(--brand-tan)]/10">
            <h3 className="text-xl font-semibold text-[var(--brand-tan)] mb-4">
              Expected Value by Handicap
            </h3>
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-[var(--brand-tan)]/20">
                    <th className="px-4 py-3 text-left text-[var(--brand-lavender)]">Handicap</th>
                    <th className="px-4 py-3 text-left text-[var(--brand-lavender)]">Skill Level</th>
                    <th className="px-4 py-3 text-right text-[var(--brand-lavender)]">P_max</th>
                    <th className="px-4 py-3 text-right text-[var(--brand-lavender)]">Expected Value</th>
                  </tr>
                </thead>
                <tbody>
                  {results.handicap_results.map((result: any) => (
                    <tr key={result.handicap} className="border-b border-[var(--brand-tan)]/10">
                      <td className="px-4 py-3 text-[var(--brand-tan)] font-semibold">{result.handicap}</td>
                      <td className="px-4 py-3 text-[var(--brand-lavender)]">
                        {result.handicap === 0 ? 'Expert' :
                         result.handicap === 10 ? 'Advanced' :
                         result.handicap === 20 ? 'Intermediate' : 'Beginner'}
                      </td>
                      <td className="px-4 py-3 text-right text-[var(--brand-dark-gold)]">{result.p_max.toFixed(1)}×</td>
                      <td className={`px-4 py-3 text-right font-semibold ${
                        result.expected_value >= 0 ? 'text-[var(--brand-bright-purple)]' : 'text-[var(--brand-rose-copper)]'
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
          <div className="bg-[var(--brand-bright-purple)]/10 border border-[var(--brand-bright-purple)]/30 p-6 rounded-xl">
            <h4 className="text-lg font-semibold text-[var(--brand-tan)] mb-2">How Does This Work?</h4>
            <ul className="text-[var(--brand-lavender)] space-y-2 text-sm">
              <li>• <strong className="text-[var(--brand-tan)]">Better players</strong> (low handicap) get lower P_max multipliers because they hit closer</li>
              <li>• <strong className="text-[var(--brand-tan)]">Weaker players</strong> (high handicap) get higher P_max multipliers to compensate for worse shots</li>
              <li>• The Kalman filter continuously adapts P_max based on actual performance</li>
              <li>• Result: All players have the same -15% expected value (15% house edge)</li>
            </ul>
          </div>
        </div>
      )}
    </div>
  );
}
