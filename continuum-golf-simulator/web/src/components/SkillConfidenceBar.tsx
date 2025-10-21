interface SkillConfidenceBarProps {
  confidence: number; // 0-100
  sigma: number;
  shotsUsed: number;
}

export default function SkillConfidenceBar({
  confidence,
  sigma,
  shotsUsed,
}: SkillConfidenceBarProps) {
  // Determine color based on confidence level
  const getColor = (conf: number) => {
    if (conf < 30) return 'bg-red-500';
    if (conf < 60) return 'bg-yellow-500';
    if (conf < 80) return 'bg-brand-lavender';
    return 'bg-green-500';
  };

  const getTextColor = (conf: number) => {
    if (conf < 30) return 'text-red-400';
    if (conf < 60) return 'text-yellow-400';
    if (conf < 80) return 'text-brand-lavender';
    return 'text-green-400';
  };

  const barColor = getColor(confidence);
  const textColor = getTextColor(confidence);

  return (
    <div className="bg-gray-800 p-4 rounded-lg border-2 border-brand-deep-purple">
      <h3 className="text-lg font-semibold text-brand-tan mb-4">Skill Confidence</h3>

      {/* Progress bar */}
      <div className="mb-4">
        <div className="flex justify-between items-center mb-2">
          <span className="text-sm text-gray-400">Estimation Confidence</span>
          <span className={`text-sm font-semibold ${textColor}`}>
            {confidence.toFixed(1)}%
          </span>
        </div>
        <div className="w-full bg-gray-700 rounded-full h-6 overflow-hidden">
          <div
            className={`${barColor} h-full transition-all duration-500 ease-out flex items-center justify-end pr-2`}
            style={{ width: `${confidence}%` }}
          >
            {confidence > 15 && (
              <span className="text-xs font-semibold text-white">
                {confidence.toFixed(0)}%
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Skill estimate details */}
      <div className="grid grid-cols-2 gap-4 text-sm">
        <div className="bg-gray-900 p-3 rounded">
          <p className="text-gray-500 text-xs mb-1">Sigma (σ)</p>
          <p className="text-brand-bright-purple font-semibold text-lg">
            {sigma.toFixed(2)} yards
          </p>
        </div>
        <div className="bg-gray-900 p-3 rounded">
          <p className="text-gray-500 text-xs mb-1">Shots Used</p>
          <p className="text-brand-tan font-semibold text-lg">
            {shotsUsed}
          </p>
        </div>
      </div>

      {/* Confidence interpretation */}
      <div className="mt-4 text-xs text-gray-500 italic">
        {confidence < 30 && (
          <p>⚠️ Low confidence - need more shots for accurate skill estimation</p>
        )}
        {confidence >= 30 && confidence < 60 && (
          <p>📊 Moderate confidence - skill estimate is stabilizing</p>
        )}
        {confidence >= 60 && confidence < 80 && (
          <p>✓ Good confidence - skill estimate is reliable</p>
        )}
        {confidence >= 80 && (
          <p>✓✓ High confidence - skill estimate is very accurate</p>
        )}
      </div>
    </div>
  );
}
