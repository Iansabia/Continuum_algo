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
  const plBgColor = stats.netPL >= 0 ? 'bg-green-500/10' : 'bg-red-500/10';
  const plBorderColor = stats.netPL >= 0 ? 'border-green-500/30' : 'border-red-500/30';

  return (
    <div className="bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/30 shadow-lg">
      <h3 className="text-xs font-medium text-[#9e8cb4] mb-2">Session</h3>

      {/* Main P/L display */}
      <div className={`${plBgColor} ${plBorderColor} border rounded-lg p-2 mb-2`}>
        <p className="text-[10px] text-[#9e8cb4]/70">Net P/L</p>
        <p className={`text-lg font-semibold ${plColor}`}>
          {stats.netPL >= 0 ? '+' : ''}${stats.netPL.toFixed(2)}
        </p>
      </div>

      {/* Session metrics */}
      <div className="space-y-1.5 text-xs">
        <div className="flex justify-between">
          <span className="text-[#9e8cb4]/70">Shots</span>
          <span className="text-[#dfc9ad] font-medium">{stats.shotsTaken}</span>
        </div>

        <div className="flex justify-between">
          <span className="text-[#9e8cb4]/70">Wagered</span>
          <span className="text-[#dfc9ad] font-medium">${stats.totalWagered.toFixed(0)}</span>
        </div>

        <div className="flex justify-between">
          <span className="text-[#9e8cb4]/70">Won</span>
          <span className="text-green-400 font-medium">${stats.totalWon.toFixed(0)}</span>
        </div>

        <div className="h-px bg-[#9e8cb4]/20 my-1"></div>

        <div className="flex justify-between">
          <span className="text-[#9e8cb4]/70">RTP</span>
          <span className="text-[#604c9c] font-medium">
            {stats.totalWagered > 0
              ? ((stats.totalWon / stats.totalWagered) * 100).toFixed(1)
              : '0'}%
          </span>
        </div>

        <div className="flex justify-between">
          <span className="text-[#9e8cb4]/70">House Edge</span>
          <span className="text-[#dfc9ad]/80 font-medium">
            {stats.actualHouseEdge.toFixed(1)}%
          </span>
        </div>
      </div>
    </div>
  );
}
