interface SkillConfidenceBarProps {
  confidence: number; // 0-100
  sigma: number;
  shotsUsed: number;
}

export default function SkillConfidenceBar({
  confidence,
}: SkillConfidenceBarProps) {
  // Determine color based on confidence level - using purple palette
  const getColor = (conf: number) => {
    if (conf < 30) return 'bg-red-500';
    if (conf < 60) return 'bg-[#ac7c6c]'; // rose-copper
    if (conf < 80) return 'bg-[#604c9c]'; // bright-purple
    return 'bg-green-500';
  };

  const getTextColor = (conf: number) => {
    if (conf < 30) return 'text-red-400';
    if (conf < 60) return 'text-[#ac7c6c]'; // rose-copper
    if (conf < 80) return 'text-[#604c9c]'; // bright-purple
    return 'text-green-400';
  };

  const barColor = getColor(confidence);
  const textColor = getTextColor(confidence);

  return (
    <div className="bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/30 shadow-lg">
      <h3 className="text-xs font-medium text-[#9e8cb4] mb-2">Confidence</h3>

      {/* Progress bar */}
      <div className="mb-2">
        <div className="flex justify-between items-center mb-1.5">
          <span className="text-[10px] text-[#9e8cb4]/70">Accuracy</span>
          <span className={`text-xs font-medium ${textColor}`}>
            {confidence.toFixed(0)}%
          </span>
        </div>
        <div className="w-full bg-[#493b7c]/20 rounded-full h-2 overflow-hidden">
          <div
            className={`${barColor} h-full transition-all duration-500 ease-out`}
            style={{ width: `${confidence}%` }}
          />
        </div>
      </div>

      {/* Confidence interpretation */}
      <div className="text-[10px] text-[#9e8cb4]/70">
        {confidence < 30 && <p>Low - need more shots</p>}
        {confidence >= 30 && confidence < 60 && <p>Moderate - stabilizing</p>}
        {confidence >= 60 && confidence < 80 && <p>Good - reliable</p>}
        {confidence >= 80 && <p>High - very accurate</p>}
      </div>
    </div>
  );
}
