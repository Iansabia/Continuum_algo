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

// Calculate expected house edge for a handicap/hole combination
const calculateHouseEdge = (handicap: number, holeDistance: number): number => {
  const sigma = handicapToSigma(handicap);

  // Target radius scales with distance (approximate)
  const targetRadius = 10 + (holeDistance / 250) * 40; // 10-50 yards

  // Expected RTP: easier holes (larger targets) = higher player return
  // House edge = 1 - RTP
  const ratio = sigma / targetRadius;
  const rtp = 0.85 - (ratio - 0.2) * 0.15; // Target: 75-90% RTP
  const houseEdge = (1 - Math.max(0.75, Math.min(0.90, rtp))) * 100;

  return houseEdge;
};

// Get color based on house edge percentage
const getHeatmapColor = (houseEdge: number): string => {
  // Green (low edge, player-friendly) to Red (high edge, house-friendly)
  // Ideal range: 10-25%

  if (houseEdge < 10) return 'rgba(239, 68, 68, 0.9)'; // Red - too low, unsustainable
  if (houseEdge < 15) return 'rgba(251, 146, 60, 0.8)'; // Orange - acceptable
  if (houseEdge < 20) return 'rgba(34, 197, 94, 0.9)'; // Green - ideal
  if (houseEdge < 25) return 'rgba(34, 197, 94, 0.7)'; // Light green - good
  return 'rgba(59, 130, 246, 0.7)'; // Blue - high but fair
};

export default function ProfitabilityHeatmap({ className = '' }: ProfitabilityHeatmapProps) {
  const heatmapData = useMemo(() => {
    return HANDICAPS.map((handicap) => ({
      handicap,
      holes: HOLES.map((hole) => ({
        ...hole,
        houseEdge: calculateHouseEdge(handicap, hole.distance),
      })),
    }));
  }, []);

  return (
    <div className={`bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/30 shadow-lg ${className}`}>
      <h3 className="text-xs font-medium text-[#9e8cb4] mb-3">
        Profitability Heatmap: House Edge %
      </h3>

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
                      backgroundColor: getHeatmapColor(cell.houseEdge),
                    }}
                  >
                    <div className="text-white font-semibold text-[11px] drop-shadow-md">
                      {cell.houseEdge.toFixed(1)}%
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
            <div className="w-3 h-3 rounded" style={{ backgroundColor: 'rgba(239, 68, 68, 0.9)' }}></div>
            <span className="text-[#9e8cb4]/70">&lt;10% (Risk)</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-3 h-3 rounded" style={{ backgroundColor: 'rgba(251, 146, 60, 0.8)' }}></div>
            <span className="text-[#9e8cb4]/70">10-15%</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-3 h-3 rounded" style={{ backgroundColor: 'rgba(34, 197, 94, 0.9)' }}></div>
            <span className="text-[#9e8cb4]/70">15-20% (Ideal)</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-3 h-3 rounded" style={{ backgroundColor: 'rgba(34, 197, 94, 0.7)' }}></div>
            <span className="text-[#9e8cb4]/70">20-25%</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-3 h-3 rounded" style={{ backgroundColor: 'rgba(59, 130, 246, 0.7)' }}></div>
            <span className="text-[#9e8cb4]/70">&gt;25%</span>
          </div>
        </div>
        <p className="text-[10px] text-[#9e8cb4]/60 mt-2 text-center">
          Shows expected house profitability across skill levels and hole difficulties
        </p>
      </div>
    </div>
  );
}
