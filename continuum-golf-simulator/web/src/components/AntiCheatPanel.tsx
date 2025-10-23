interface AnomalyReport {
  is_suspicious: boolean;
  confidence: number;
  detected_patterns: string[];
  recommended_action: string;
}

interface AntiCheatPanelProps {
  report: AnomalyReport | null;
  shotCount: number;
}

export default function AntiCheatPanel({ report, shotCount }: AntiCheatPanelProps) {
  // Don't show panel until we have enough shots for analysis
  if (shotCount < 20) {
    return (
      <div className="bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/30 shadow-lg">
        <h3 className="text-xs font-medium text-[#9e8cb4] mb-2 flex items-center gap-2">
          <span>🛡️</span>
          Anti-Cheat Monitor
        </h3>
        <div className="text-[10px] text-[#9e8cb4]/60">
          <p>Need {20 - shotCount} more shots for analysis...</p>
        </div>
      </div>
    );
  }

  if (!report) {
    return (
      <div className="bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/30 shadow-lg">
        <h3 className="text-xs font-medium text-[#9e8cb4] mb-2 flex items-center gap-2">
          <span>🛡️</span>
          Anti-Cheat Monitor
        </h3>
        <div className="text-[10px] text-[#9e8cb4]/60">
          <p>Analyzing patterns...</p>
        </div>
      </div>
    );
  }

  // Determine status color and icon
  const getSuspicionLevel = () => {
    if (report.is_suspicious) {
      if (report.confidence >= 0.8) {
        return {
          color: 'text-red-400',
          bgColor: 'bg-red-500/20',
          borderColor: 'border-red-500/50',
          icon: '🚨',
          label: 'CRITICAL',
        };
      }
      return {
        color: 'text-orange-400',
        bgColor: 'bg-orange-500/20',
        borderColor: 'border-orange-500/50',
        icon: '⚠️',
        label: 'WARNING',
      };
    }

    if (report.confidence >= 0.4) {
      return {
        color: 'text-yellow-400',
        bgColor: 'bg-yellow-500/20',
        borderColor: 'border-yellow-500/50',
        icon: '👀',
        label: 'MONITOR',
      };
    }

    return {
      color: 'text-green-400',
      bgColor: 'bg-green-500/20',
      borderColor: 'border-green-500/50',
      icon: '✅',
      label: 'CLEAN',
    };
  };

  const status = getSuspicionLevel();

  return (
    <div className={`bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border ${status.borderColor} shadow-lg`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-xs font-medium text-[#9e8cb4] flex items-center gap-2">
          <span>🛡️</span>
          Anti-Cheat Monitor
        </h3>
        <div className={`text-[10px] font-bold ${status.color} flex items-center gap-1`}>
          <span>{status.icon}</span>
          <span>{status.label}</span>
        </div>
      </div>

      {/* Suspicion Score */}
      <div className="mb-2">
        <div className="flex justify-between items-center mb-1">
          <span className="text-[10px] text-[#9e8cb4]/70">Suspicion Level</span>
          <span className={`text-[10px] font-semibold ${status.color}`}>
            {(report.confidence * 100).toFixed(0)}%
          </span>
        </div>
        <div className="w-full bg-[#493b7c]/30 rounded-full h-1.5">
          <div
            className={`h-1.5 rounded-full transition-all duration-300 ${
              report.is_suspicious
                ? report.confidence >= 0.8
                  ? 'bg-gradient-to-r from-red-500 to-red-600'
                  : 'bg-gradient-to-r from-orange-500 to-orange-600'
                : report.confidence >= 0.4
                ? 'bg-gradient-to-r from-yellow-500 to-yellow-600'
                : 'bg-gradient-to-r from-green-500 to-green-600'
            }`}
            style={{ width: `${report.confidence * 100}%` }}
          />
        </div>
      </div>

      {/* Detected Patterns */}
      {report.detected_patterns.length > 0 && (
        <div className="mb-2">
          <div className="text-[10px] font-medium text-[#9e8cb4]/70 mb-1">
            Detected Patterns:
          </div>
          <div className="space-y-1">
            {report.detected_patterns.map((pattern, index) => (
              <div
                key={index}
                className={`text-[9px] ${status.bgColor} ${status.color} px-2 py-1 rounded`}
              >
                • {pattern}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Recommended Action */}
      <div className={`text-[10px] ${status.bgColor} ${status.color} px-2 py-1.5 rounded mt-2`}>
        <strong>Action:</strong> {report.recommended_action}
      </div>

      {/* Analysis Info */}
      <div className="mt-2 pt-2 border-t border-[#9e8cb4]/20 text-[9px] text-[#9e8cb4]/50">
        Analyzing {shotCount} shots across 4 detection algorithms
      </div>
    </div>
  );
}
