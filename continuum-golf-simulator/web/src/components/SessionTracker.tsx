import { motion } from 'framer-motion';

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
  const plColor = stats.netPL >= 0 ? 'text-[var(--brand-bright-purple)]' : 'text-[var(--brand-rose-copper)]';
  const plBgColor = stats.netPL >= 0 ? 'bg-[var(--brand-bright-purple)]/10' : 'bg-[var(--brand-rose-copper)]/10';
  const plBorderColor = stats.netPL >= 0 ? 'border-[var(--brand-bright-purple)]/30' : 'border-[var(--brand-rose-copper)]/30';

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl border border-[var(--brand-tan)]/20 p-3"
    >
      <h3 className="text-xs font-semibold text-[var(--brand-lavender)] uppercase tracking-wider mb-2">
        Session Stats
      </h3>

      {/* Main P/L display */}
      <motion.div
        animate={{
          scale: stats.netPL !== 0 ? [1, 1.02, 1] : 1,
        }}
        transition={{ duration: 0.3 }}
        className={`${plBgColor} ${plBorderColor} border rounded-xl p-2 mb-2`}
      >
        <p className="text-[10px] text-[var(--brand-lavender)] mb-0.5">Net P/L</p>
        <p className={`text-xl font-semibold ${plColor}`}>
          {stats.netPL >= 0 ? '+' : ''}${stats.netPL.toFixed(2)}
        </p>
      </motion.div>

      {/* Session metrics */}
      <div className="space-y-1.5 text-xs">
        <div className="flex justify-between items-center">
          <span className="text-[var(--brand-lavender)]">Shots</span>
          <span className="text-[var(--brand-tan)] font-medium">{stats.shotsTaken}</span>
        </div>

        <div className="flex justify-between items-center">
          <span className="text-[var(--brand-lavender)]">Wagered</span>
          <span className="text-[var(--brand-tan)] font-medium">${stats.totalWagered.toFixed(0)}</span>
        </div>

        <div className="flex justify-between items-center">
          <span className="text-[var(--brand-lavender)]">Won</span>
          <span className="text-[var(--brand-bright-purple)] font-medium">${stats.totalWon.toFixed(0)}</span>
        </div>

        <div className="h-px bg-[var(--brand-tan)]/20 my-1.5"></div>

        <div className="flex justify-between items-center">
          <span className="text-[var(--brand-lavender)]">RTP</span>
          <span className="text-[var(--brand-dark-gold)] font-medium">
            {stats.totalWagered > 0
              ? ((stats.totalWon / stats.totalWagered) * 100).toFixed(1)
              : '0'}%
          </span>
        </div>

        <div className="flex justify-between items-center">
          <span className="text-[var(--brand-lavender)]">House Edge</span>
          <span className="text-[var(--brand-tan)] font-medium">
            {stats.actualHouseEdge.toFixed(1)}%
          </span>
        </div>
      </div>
    </motion.div>
  );
}
