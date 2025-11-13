import { motion } from 'framer-motion';

interface SkillConfidenceBarProps {
  confidence: number; // 0-100
  sigma: number;
  shotsUsed: number;
}

export default function SkillConfidenceBar({
  confidence,
}: SkillConfidenceBarProps) {
  // Determine color based on confidence level - Brand colors
  const getColor = (conf: number) => {
    if (conf < 30) return 'from-[var(--brand-rose-copper)] to-[#ac7c6c]';
    if (conf < 60) return 'from-[var(--brand-dark-gold)] to-[#7e6649]';
    if (conf < 80) return 'from-[var(--brand-lavender)] to-[#9e8cb4]';
    return 'from-[var(--brand-bright-purple)] to-[var(--brand-deep-purple)]';
  };

  const getTextColor = (conf: number) => {
    if (conf < 30) return 'text-[var(--brand-rose-copper)]';
    if (conf < 60) return 'text-[var(--brand-dark-gold)]';
    if (conf < 80) return 'text-[var(--brand-lavender)]';
    return 'text-[var(--brand-bright-purple)]';
  };

  const getLabel = (conf: number) => {
    if (conf < 30) return 'Low - need more shots';
    if (conf < 60) return 'Moderate - stabilizing';
    if (conf < 80) return 'Good - reliable';
    return 'High - very accurate';
  };

  const barColor = getColor(confidence);
  const textColor = getTextColor(confidence);
  const label = getLabel(confidence);

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-2xl border border-white/10 p-3"
    >
      <h3 className="text-xs font-semibold text-white/70 uppercase tracking-wider mb-2">
        Confidence
      </h3>

      {/* Progress bar */}
      <div className="mb-2">
        <div className="flex justify-between items-center mb-2">
          <span className="text-xs text-white/70">Accuracy</span>
          <motion.span
            key={confidence}
            initial={{ scale: 1.1 }}
            animate={{ scale: 1 }}
            className={`text-sm font-semibold ${textColor}`}
          >
            {confidence.toFixed(0)}%
          </motion.span>
        </div>
        <div className="w-full bg-[var(--brand-dark-gray)] rounded-full h-2 overflow-hidden">
          <motion.div
            initial={{ width: 0 }}
            animate={{ width: `${confidence}%` }}
            transition={{ duration: 0.6, ease: 'easeOut' }}
            className={`h-full bg-gradient-to-r ${barColor} shadow-lg`}
          />
        </div>
      </div>

      {/* Confidence interpretation */}
      <p className="text-[10px] text-white/70">{label}</p>
    </motion.div>
  );
}
