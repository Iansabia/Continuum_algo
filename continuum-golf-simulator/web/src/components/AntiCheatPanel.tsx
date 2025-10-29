import { motion, AnimatePresence } from 'framer-motion';

interface AnomalyReport {
  isSuspicious: boolean;
  confidence: number;
  detectedPatterns: string[];
  recommendedAction: string;
}

interface AntiCheatPanelProps {
  report: AnomalyReport | null;
  shotCount: number;
}

export default function AntiCheatPanel({ report, shotCount }: AntiCheatPanelProps) {
  // Don't show panel until we have enough shots for analysis
  if (shotCount < 20) {
    return (
      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl border border-[var(--brand-tan)]/20 p-3 h-full flex flex-col"
      >
        <h3 className="text-xs font-semibold text-[var(--brand-lavender)] uppercase tracking-wider mb-2 flex items-center gap-2">
          <span>🛡️</span>
          Anti-Cheat Monitor
        </h3>
        <div className="text-xs text-[var(--brand-lavender)]">
          <p>Need {20 - shotCount} more shots for analysis...</p>
        </div>
      </motion.div>
    );
  }

  if (!report) {
    return (
      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl border border-[var(--brand-tan)]/20 p-3 h-full flex flex-col"
      >
        <h3 className="text-xs font-semibold text-[var(--brand-lavender)] uppercase tracking-wider mb-2 flex items-center gap-2">
          <span>🛡️</span>
          Anti-Cheat Monitor
        </h3>
        <div className="text-xs text-[var(--brand-lavender)]">
          <p>Analyzing patterns...</p>
        </div>
      </motion.div>
    );
  }

  // Determine status color and icon
  const getSuspicionLevel = () => {
    if (report.isSuspicious) {
      if (report.confidence >= 0.8) {
        return {
          color: 'text-[var(--brand-rose-copper)]',
          bgColor: 'bg-[var(--brand-rose-copper)]/10',
          borderColor: 'border-[var(--brand-rose-copper)]/30',
          gradientColor: 'from-[var(--brand-rose-copper)] to-[#ac7c6c]',
          icon: '🚨',
          label: 'CRITICAL',
        };
      }
      return {
        color: 'text-[var(--brand-dark-gold)]',
        bgColor: 'bg-[var(--brand-dark-gold)]/10',
        borderColor: 'border-[var(--brand-dark-gold)]/30',
        gradientColor: 'from-[var(--brand-dark-gold)] to-[#7e6649]',
        icon: '⚠️',
        label: 'WARNING',
      };
    }

    if (report.confidence >= 0.4) {
      return {
        color: 'text-[var(--brand-tan)]',
        bgColor: 'bg-[var(--brand-tan)]/10',
        borderColor: 'border-[var(--brand-tan)]/30',
        gradientColor: 'from-[var(--brand-tan)] to-[#dfc9ad]',
        icon: '👀',
        label: 'MONITOR',
      };
    }

    return {
      color: 'text-[var(--brand-bright-purple)]',
      bgColor: 'bg-[var(--brand-bright-purple)]/10',
      borderColor: 'border-[var(--brand-bright-purple)]/30',
      gradientColor: 'from-[var(--brand-bright-purple)] to-[var(--brand-deep-purple)]',
      icon: '✅',
      label: 'CLEAN',
    };
  };

  const status = getSuspicionLevel();

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className={`bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl border ${status.borderColor} p-3 h-full flex flex-col`}
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-xs font-semibold text-[var(--brand-lavender)] uppercase tracking-wider flex items-center gap-2">
          <span>🛡️</span>
          Anti-Cheat Monitor
        </h3>
        <motion.div
          initial={{ scale: 1.1 }}
          animate={{ scale: 1 }}
          className={`text-xs font-bold ${status.color} flex items-center gap-1.5`}
        >
          <span>{status.icon}</span>
          <span>{status.label}</span>
        </motion.div>
      </div>

      {/* Suspicion Score */}
      <div className="mb-2">
        <div className="flex justify-between items-center mb-2">
          <span className="text-xs text-[var(--brand-lavender)]">Suspicion Level</span>
          <motion.span
            key={report.confidence}
            initial={{ scale: 1.1 }}
            animate={{ scale: 1 }}
            className={`text-sm font-semibold ${status.color}`}
          >
            {(report.confidence * 100).toFixed(0)}%
          </motion.span>
        </div>
        <div className="w-full bg-[var(--brand-dark-gray)] rounded-full h-2 overflow-hidden">
          <motion.div
            initial={{ width: 0 }}
            animate={{ width: `${report.confidence * 100}%` }}
            transition={{ duration: 0.6, ease: 'easeOut' }}
            className={`h-full bg-gradient-to-r ${status.gradientColor} shadow-lg`}
          />
        </div>
      </div>

      {/* Detected Patterns */}
      <AnimatePresence>
        {report.detectedPatterns.length > 0 && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="mb-2 flex-1 overflow-y-auto"
          >
            <div className="text-[10px] font-medium text-[var(--brand-lavender)] mb-1.5">
              Detected Patterns:
            </div>
            <div className="space-y-1.5">
              {report.detectedPatterns.map((pattern, index) => (
                <motion.div
                  key={index}
                  initial={{ opacity: 0, x: -10 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ delay: index * 0.05 }}
                  className={`text-[10px] ${status.bgColor} ${status.color} px-2 py-1.5 rounded-lg border ${status.borderColor}`}
                >
                  • {pattern}
                </motion.div>
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Recommended Action */}
      <div className={`text-[10px] ${status.bgColor} ${status.color} px-2 py-1.5 rounded-lg border ${status.borderColor}`}>
        <strong>Action:</strong> {report.recommendedAction}
      </div>

      {/* Analysis Info */}
      <div className="mt-2 pt-2 border-t border-[var(--brand-tan)]/20 text-[10px] text-[var(--brand-lavender)]">
        {shotCount} shots analyzed
      </div>
    </motion.div>
  );
}
