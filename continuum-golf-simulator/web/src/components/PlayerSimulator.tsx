import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useSimulator } from '../hooks/useSimulator';
import { useExportPDF } from '../hooks/useExportPDF';
import TargetVisualizer from './TargetVisualizer';
import BVNHeatmap3D from './BVNHeatmap3D';
import PayoutCurveChart from './PayoutCurveChart';
import PmaxHistoryChart from './PmaxHistoryChart';
import SkillConfidenceBar from './SkillConfidenceBar';
import SessionTracker from './SessionTracker';
import AntiCheatPanel from './AntiCheatPanel';

const HOLES = [
  { id: 1, distance: 75, name: 'Hole 1 (75y)' },
  { id: 2, distance: 100, name: 'Hole 2 (100y)' },
  { id: 3, distance: 125, name: 'Hole 3 (125y)' },
  { id: 4, distance: 150, name: 'Hole 4 (150y)' },
  { id: 5, distance: 175, name: 'Hole 5 (175y)' },
  { id: 6, distance: 200, name: 'Hole 6 (200y)' },
  { id: 7, distance: 225, name: 'Hole 7 (225y)' },
  { id: 8, distance: 250, name: 'Hole 8 (250y)' },
];

export default function PlayerSimulator() {
  const [handicap, setHandicap] = useState(10);
  const [wager, setWager] = useState(10);
  const [selectedHoleId, setSelectedHoleId] = useState<number>(1);
  const [devMode, setDevMode] = useState(false);
  const [manualDistance, setManualDistance] = useState(5);
  const [isShooting, setIsShooting] = useState(false);
  const [batchSize, setBatchSize] = useState(10);

  const {
    shots,
    skillEstimate,
    pmaxHistory,
    sessionStats,
    breakevenRadius,
    currentHole,
    antiCheatReport,
    shootOnce,
    shootBatch,
    reset,
  } = useSimulator(handicap, selectedHoleId);

  const { exportToPDF } = useExportPDF();

  const handleShoot = () => {
    setIsShooting(true);
    const distance = devMode ? manualDistance : undefined;
    shootOnce(wager, distance);
    setTimeout(() => setIsShooting(false), 500);
  };

  const handleBatchShoot = () => {
    setIsShooting(true);
    shootBatch(wager, batchSize);
    setTimeout(() => setIsShooting(false), 500);
  };

  const handleReset = () => {
    if (confirm('Reset session? This will clear all shot data.')) {
      reset();
    }
  };

  const handleExport = () => {
    exportToPDF({
      handicap,
      sessionStats,
      skillEstimate,
      currentHole,
      breakevenRadius,
    });
  };

  const lastShot = shots.length > 0 ? shots[shots.length - 1] : null;

  return (
    <div className="relative h-full overflow-hidden flex flex-col">
      {/* Header */}
      <motion.div
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="relative mb-3"
      >
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold text-white mb-1">Continuum Golf</h2>
            <p className="text-white/70 text-xs">AI-Powered Simulator</p>
          </div>

          {/* Developer Mode Toggle */}
          <motion.button
            whileTap={{ scale: 0.95 }}
            onClick={() => setDevMode(!devMode)}
            className={`relative inline-flex h-8 w-14 items-center rounded-full transition-colors duration-200 ${
              devMode ? 'bg-gradient-to-r from-[#604c9c] to-[#493b7c]' : 'bg-black/30'
            }`}
          >
            <motion.span
              layout
              transition={{ type: 'spring', stiffness: 500, damping: 30 }}
              className={`inline-block h-6 w-6 transform rounded-full bg-white shadow-lg transition ${
                devMode ? 'translate-x-7' : 'translate-x-1'
              }`}
            />
          </motion.button>
        </div>
      </motion.div>

      {/* Main Content */}
      <div className="relative flex-1 grid grid-cols-12 gap-3 min-h-0">

          {/* Left Sidebar - Controls */}
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.1 }}
            className="col-span-2 flex flex-col gap-3 overflow-y-auto"
          >
            {/* Controls Card */}
            <div className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-2xl border border-white/10 p-3 space-y-3">
              {/* Handicap */}
              <div>
                <label className="block text-xs font-medium text-white/70 mb-2">
                  Handicap <span className="text-white font-semibold">{handicap}</span>
                </label>
                <input
                  type="range"
                  min="0"
                  max="30"
                  value={handicap}
                  onChange={(e) => setHandicap(Number(e.target.value))}
                  className="w-full h-1.5 bg-[var(--brand-dark-gray)] rounded-full appearance-none cursor-pointer slider-thumb"
                />
              </div>

              {/* Wager */}
              <div>
                <label className="block text-xs font-medium text-white/70 mb-2">
                  Wager <span className="text-white font-semibold">${wager}</span>
                </label>
                <input
                  type="range"
                  min="1"
                  max="100"
                  value={wager}
                  onChange={(e) => setWager(Number(e.target.value))}
                  className="w-full h-1.5 bg-[var(--brand-dark-gray)] rounded-full appearance-none cursor-pointer slider-thumb"
                />
              </div>

              {/* Hole Selection */}
              <div>
                <label className="block text-xs font-medium text-white/70 mb-2">Hole</label>
                <select
                  value={selectedHoleId}
                  onChange={(e) => setSelectedHoleId(Number(e.target.value))}
                  className="w-full bg-white/40 backdrop-blur-xl text-white border border-white/10 rounded-xl px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-[var(--brand-bright-purple)]/50 transition"
                >
                  {HOLES.map((hole) => (
                    <option key={hole.id} value={hole.id} className="bg-[#2a2a2a] text-white">
                      {hole.name}
                    </option>
                  ))}
                </select>
              </div>

              {/* Batch Size */}
              <div>
                <label className="block text-xs font-medium text-white/70 mb-2">
                  Batch Size <span className="text-white font-semibold">{batchSize}</span>
                </label>
                <input
                  type="range"
                  min="1"
                  max="100"
                  value={batchSize}
                  onChange={(e) => setBatchSize(Number(e.target.value))}
                  className="w-full h-1.5 bg-[var(--brand-dark-gray)] rounded-full appearance-none cursor-pointer slider-thumb"
                />
              </div>
            </div>

            {/* Current Hole Stats */}
            <div className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-2xl border border-white/10 p-3">
              <h3 className="text-xs font-semibold text-white/70 uppercase tracking-wider mb-3">
                Current Hole
              </h3>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between items-center">
                  <span className="text-white/70">Distance</span>
                  <span className="text-white font-medium">{currentHole.distance}y</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-white/70">Target</span>
                  <span className="text-white font-medium">{currentHole.targetRadius.toFixed(1)}y</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-white/70">σ</span>
                  <span className="text-white font-medium">{skillEstimate.sigma.toFixed(2)}y</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-white/70">P_max</span>
                  <span className="text-white font-medium">{skillEstimate.pmax.toFixed(3)}</span>
                </div>
              </div>
            </div>

            {/* Session Stats */}
            <SessionTracker stats={sessionStats} />

            {/* Confidence Bar */}
            <SkillConfidenceBar
              confidence={skillEstimate.confidence}
              sigma={skillEstimate.sigma}
              shotsUsed={shots.length}
            />

            {/* Developer Controls */}
            <AnimatePresence>
              {devMode && (
                <motion.div
                  initial={{ opacity: 0, height: 0 }}
                  animate={{ opacity: 1, height: 'auto' }}
                  exit={{ opacity: 0, height: 0 }}
                  className="bg-[var(--brand-bright-purple)]/10 backdrop-blur-xl rounded-2xl border border-[var(--brand-bright-purple)]/30 p-4"
                >
                  <h3 className="text-xs font-semibold text-[var(--brand-bright-purple)] uppercase tracking-wider mb-3">
                    Developer Mode
                  </h3>
                  <div>
                    <label className="block text-xs font-medium text-white/70 mb-2">
                      Manual Miss <span className="text-white font-semibold">{manualDistance.toFixed(1)}y</span>
                    </label>
                    <input
                      type="range"
                      min="0"
                      max="50"
                      step="0.1"
                      value={manualDistance}
                      onChange={(e) => setManualDistance(Number(e.target.value))}
                      className="w-full h-1.5 bg-[var(--brand-dark-gray)] rounded-full appearance-none cursor-pointer slider-thumb"
                    />
                  </div>
                </motion.div>
              )}
            </AnimatePresence>

            {/* Action Buttons */}
            <div className="space-y-2">
              <motion.button
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                onClick={handleShoot}
                disabled={isShooting}
                className="w-full bg-gradient-to-r from-[var(--brand-bright-purple)] to-[var(--brand-deep-purple)] hover:from-[var(--brand-deep-purple)] hover:to-[var(--brand-bright-purple)] text-white font-medium py-3 px-4 rounded-xl transition-all disabled:opacity-50 disabled:cursor-not-allowed shadow-lg shadow-[var(--brand-bright-purple)]/25"
              >
                {isShooting ? 'Shooting...' : 'Shoot 1x'}
              </motion.button>

              <motion.button
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                onClick={handleBatchShoot}
                disabled={isShooting}
                className="w-full bg-gradient-to-br from-black/40 to-black/30 hover:from-black/50 hover:to-black/40 text-white font-medium py-3 px-4 rounded-xl transition-all disabled:opacity-50 disabled:cursor-not-allowed border border-white/10"
              >
                {isShooting ? 'Shooting...' : `Shoot ${batchSize}x`}
              </motion.button>

              <div className="grid grid-cols-2 gap-2">
                <motion.button
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  onClick={handleExport}
                  disabled={shots.length === 0}
                  className="bg-gradient-to-br from-black/40 to-black/30 hover:from-black/50 hover:to-black/40 text-white font-medium py-2 px-3 rounded-xl transition-all disabled:opacity-50 disabled:cursor-not-allowed border border-white/10 text-sm"
                >
                  Export
                </motion.button>

                <motion.button
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  onClick={handleReset}
                  className="bg-black/20 hover:bg-black/30 text-white/70 hover:text-white font-medium py-2 px-3 rounded-xl transition-all border border-white/10 text-sm"
                >
                  Reset
                </motion.button>
              </div>
            </div>
          </motion.div>

          {/* Center - Visualizers */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
            className="col-span-6 flex flex-col gap-3 h-full"
          >
            {/* 2D Target View - Dark glass background */}
            <div className="flex-1 flex items-center justify-center relative bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-2xl border border-white/10 p-4">
              <TargetVisualizer
                sigma={skillEstimate.sigma}
                breakevenRadius={breakevenRadius}
                targetRadius={currentHole.targetRadius}
                shots={shots}
                currentShot={isShooting ? lastShot : null}
                width={450}
                height={450}
              />
              <p className="absolute bottom-2 left-1/2 transform -translate-x-1/2 text-xs text-center text-white/80 font-medium">
                Target: {currentHole.targetRadius.toFixed(2)}y | σ = {skillEstimate.sigma.toFixed(2)}y | Breakeven: {breakevenRadius.toFixed(2)}y
              </p>
            </div>

            {/* 3D BVN View - Dark glass background */}
            <div className="flex-1 relative bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-2xl border border-white/10">
              <BVNHeatmap3D
                sigmaX={skillEstimate.sigmaX}
                sigmaY={skillEstimate.sigmaY}
                currentPmax={skillEstimate.pmax}
                shots={shots}
              />
            </div>
          </motion.div>

          {/* Right Sidebar - Charts & Anti-Cheat */}
          <motion.div
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.3 }}
            className="col-span-4 flex flex-col gap-3 h-full overflow-y-auto"
          >
            {/* Payout Curve */}
            <div className="h-1/3 min-h-0">
              <PayoutCurveChart
                pmax={skillEstimate.pmax}
                breakevenRadius={breakevenRadius}
                targetRadius={currentHole.targetRadius}
                k={currentHole.k}
                lastShotDistance={lastShot?.distance}
                lastShotMultiplier={lastShot?.multiplier}
              />
            </div>

            {/* P_max Evolution */}
            <div className="h-1/3 min-h-0">
              <PmaxHistoryChart history={pmaxHistory} />
            </div>

            {/* Anti-Cheat Monitor */}
            <div className="flex-1 min-h-0">
              <AntiCheatPanel
                report={antiCheatReport}
                shotCount={shots.length}
              />
            </div>
          </motion.div>
      </div>
    </div>
  );
}
