interface SessionStats {
  shotsTaken: number;
  totalWagered: number;
  totalWon: number;
  netPL: number;
  actualHouseEdge: number;
  theoreticalHouseEdge: number;
}

interface SessionTrackerProps {
  stats: SessionStats;
}

export default function SessionTracker({ stats }: SessionTrackerProps) {
  const plColor = stats.netPL >= 0 ? 'text-green-400' : 'text-red-400';
  const plBgColor = stats.netPL >= 0 ? 'bg-green-900/30' : 'bg-red-900/30';
  const plBorderColor = stats.netPL >= 0 ? 'border-green-600' : 'border-red-600';

  return (
    <div className="bg-gray-800 p-4 rounded-lg border-2 border-brand-deep-purple">
      <h3 className="text-lg font-semibold text-brand-tan mb-4">Session Tracker</h3>

      {/* Main P/L display */}
      <div className={`${plBgColor} ${plBorderColor} border-2 rounded-lg p-4 mb-4`}>
        <p className="text-sm text-gray-400 mb-1">Net Profit/Loss</p>
        <p className={`text-3xl font-bold ${plColor}`}>
          {stats.netPL >= 0 ? '+' : ''}${stats.netPL.toFixed(2)}
        </p>
      </div>

      {/* Session metrics */}
      <div className="space-y-3">
        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-400">Shots Taken</span>
          <span className="text-brand-tan font-semibold">{stats.shotsTaken}</span>
        </div>

        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-400">Total Wagered</span>
          <span className="text-brand-tan font-semibold">${stats.totalWagered.toFixed(2)}</span>
        </div>

        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-400">Total Won</span>
          <span className="text-green-400 font-semibold">${stats.totalWon.toFixed(2)}</span>
        </div>

        <div className="h-px bg-gray-700 my-2"></div>

        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-400">RTP (Return to Player)</span>
          <span className="text-brand-bright-purple font-semibold">
            {stats.totalWagered > 0
              ? ((stats.totalWon / stats.totalWagered) * 100).toFixed(2)
              : '0.00'}%
          </span>
        </div>

        <div className="h-px bg-gray-700 my-2"></div>

        {/* House edge comparison */}
        <div className="bg-gray-900 p-3 rounded">
          <div className="flex justify-between items-center mb-2">
            <span className="text-xs text-gray-500">Actual House Edge</span>
            <span className="text-brand-rose-copper font-semibold">
              {stats.actualHouseEdge.toFixed(2)}%
            </span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-xs text-gray-500">Theoretical House Edge</span>
            <span className="text-gray-400 font-semibold">
              {stats.theoreticalHouseEdge.toFixed(2)}%
            </span>
          </div>
          <div className="mt-2 text-xs text-gray-500 italic">
            {(() => {
              const deviation = Math.abs(stats.actualHouseEdge - stats.theoreticalHouseEdge);
              const varianceThreshold = 5; // Allow 5% deviation (±5%)
              const minShotsForConvergence = 50; // Need 50+ shots for meaningful statistics

              if (deviation < varianceThreshold) {
                return (
                  <p className="text-green-400">
                    ✓ Within expected variance (±{deviation.toFixed(1)}%)
                  </p>
                );
              } else if (stats.shotsTaken < minShotsForConvergence) {
                return (
                  <p className="text-blue-400">
                    ℹ️ Need {minShotsForConvergence - stats.shotsTaken} more shots for convergence
                  </p>
                );
              } else {
                return (
                  <p className="text-yellow-400">
                    ⚠️ High variance: {deviation.toFixed(1)}% deviation
                  </p>
                );
              }
            })()}
          </div>
        </div>
      </div>

      {/* Reset button placeholder */}
      <button
        className="mt-4 w-full bg-brand-deep-purple hover:bg-brand-bright-purple text-brand-tan font-semibold py-2 px-4 rounded transition-colors"
        onClick={() => {
          if (confirm('Reset session? This will clear all shot data.')) {
            window.location.reload(); // Temporary - will be replaced with proper reset
          }
        }}
      >
        Reset Session
      </button>
    </div>
  );
}
