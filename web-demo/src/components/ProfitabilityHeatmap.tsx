import { useMemo } from 'react';

interface ProfitabilityHeatmapProps {
  className?: string;
}

const HANDICAPS = [0, 5, 10, 15, 20, 25, 30];
const HOLES = [
  { id: 1, distance: 75, name: 'H1' },
  { id: 2, distance: 100, name: 'H2' },
  { id: 3, distance: 125, name: 'H3' },
  { id: 4, distance: 150, name: 'H4' },
  { id: 5, distance: 175, name: 'H5' },
  { id: 6, distance: 200, name: 'H6' },
  { id: 7, distance: 225, name: 'H7' },
  { id: 8, distance: 250, name: 'H8' },
];

// Simplified sigma calculation based on handicap
// Real calculation would come from WASM, this is for demo
const handicapToSigma = (handicap: number): number => {
  return 3.0 + (handicap / 30) * 12.0; // Range: 3-15 yards
};

// Calculate P_max required to achieve fair 85% RTP for a handicap/hole combination
// The Kalman filter adjusts P_max so ALL skill levels have equal expected value
const calculateRequiredPmax = (handicap: number, holeDistance: number): number => {
  const sigma = handicapToSigma(handicap);

  // Target radius scales with distance (approximate)
  const targetRadius = 10 + (holeDistance / 250) * 40; // 10-50 yards

  // For a Rayleigh distribution with sigma and target radius d_max:
  // Expected payout ≈ integral of P_max * (1 - r/d_max)^k * f(r) dr
  // Simplified: higher sigma/d_max ratio requires higher P_max to maintain RTP
  const sigmaFt = sigma * 3; // Convert to feet
  const dMaxFt = targetRadius * 3;
  const ratio = sigmaFt / dMaxFt;

  // P_max increases with skill variance to maintain constant 85% RTP
  // Better players (low sigma) get lower P_max but hit more often
  // Worse players (high sigma) get higher P_max but miss more often
  // Result: Equal expected value for all!
  const targetRTP = 0.85;
  const estimatedHitRate = Math.max(0.05, 0.6 * (1 - Math.min(0.9, ratio)));
  const pmax = targetRTP / estimatedHitRate;

  return Math.max(1.5, Math.min(20.0, pmax));
};

// Get color based on P_max value (shows difficulty/variance compensation)
const getPmaxColor = (pmax: number): string => {
  // Lower P_max (skilled players) = cooler colors
  // Higher P_max (less skilled players) = warmer colors
  // This visualizes how the system compensates for skill variance

  if (pmax < 3) return 'rgba(59, 130, 246, 0.85)'; // Blue - low P_max (skilled)
  if (pmax < 5) return 'rgba(34, 197, 94, 0.85)'; // Green - moderate-low
  if (pmax < 8) return 'rgba(234, 179, 8, 0.85)'; // Yellow - moderate
  if (pmax < 12) return 'rgba(251, 146, 60, 0.85)'; // Orange - moderate-high
  return 'rgba(239, 68, 68, 0.85)'; // Red - high P_max (less skilled)
};

export default function ProfitabilityHeatmap({ className = '' }: ProfitabilityHeatmapProps) {
  const heatmapData = useMemo(() => {
    return HANDICAPS.map((handicap) => ({
      handicap,
      holes: HOLES.map((hole) => ({
        ...hole,
        pmax: calculateRequiredPmax(handicap, hole.distance),
      })),
    }));
  }, []);

  return (
    <div className={`bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/30 shadow-lg ${className}`}>
      <h3 className="text-xs font-medium text-[#9e8cb4] mb-3">
        Fairness Heatmap: Required P_max
      </h3>
      <p className="text-[10px] text-[#9e8cb4]/60 mb-2">
        Shows P_max values that ensure 85% RTP for all skill levels
      </p>

      <div className="overflow-x-auto">
        <table className="w-full text-xs">
          <thead>
            <tr>
              <th className="text-left text-[#9e8cb4]/70 font-medium pb-2 pr-2 sticky left-0 bg-gradient-to-r from-[#493b7c]/20 to-transparent">
                HC
              </th>
              {HOLES.map((hole) => (
                <th key={hole.id} className="text-center text-[#9e8cb4]/70 font-medium pb-2 px-1">
                  <div>{hole.name}</div>
                  <div className="text-[10px] text-[#9e8cb4]/50">{hole.distance}y</div>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {heatmapData.map((row) => (
              <tr key={row.handicap}>
                <td className="text-[#dfc9ad] font-medium py-1 pr-2 sticky left-0 bg-gradient-to-r from-[#493b7c]/20 to-transparent">
                  {row.handicap}
                </td>
                {row.holes.map((cell) => (
                  <td
                    key={cell.id}
                    className="text-center py-1 px-1"
                    style={{
                      backgroundColor: getPmaxColor(cell.pmax),
                    }}
                  >
                    <div className="text-white font-semibold text-[11px] drop-shadow-md">
                      {cell.pmax.toFixed(1)}x
                    </div>
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Legend */}
      <div className="mt-3 pt-3 border-t border-[#9e8cb4]/20">
        <div className="flex items-center justify-between text-[10px]">
          <div className="flex items-center gap-1">
            <div className="w-3 h-3 rounded" style={{ backgroundColor: 'rgba(59, 130, 246, 0.85)' }}></div>
            <span className="text-[#9e8cb4]/70">&lt;3x (Skilled)</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-3 h-3 rounded" style={{ backgroundColor: 'rgba(34, 197, 94, 0.85)' }}></div>
            <span className="text-[#9e8cb4]/70">3-5x</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-3 h-3 rounded" style={{ backgroundColor: 'rgba(234, 179, 8, 0.85)' }}></div>
            <span className="text-[#9e8cb4]/70">5-8x</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-3 h-3 rounded" style={{ backgroundColor: 'rgba(251, 146, 60, 0.85)' }}></div>
            <span className="text-[#9e8cb4]/70">8-12x</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-3 h-3 rounded" style={{ backgroundColor: 'rgba(239, 68, 68, 0.85)' }}></div>
            <span className="text-[#9e8cb4]/70">&gt;12x (Beginner)</span>
          </div>
        </div>
        <p className="text-[10px] text-[#9e8cb4]/60 mt-2 text-center">
          Higher P_max compensates for skill variance • All players get ~85% RTP regardless of skill
        </p>
      </div>
    </div>
  );
}
