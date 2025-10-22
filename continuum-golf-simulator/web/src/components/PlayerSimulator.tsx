import { useState } from 'react';
import { useSimulator } from '../hooks/useSimulator';
import TargetVisualizer from './TargetVisualizer';
import BVNHeatmap3D from './BVNHeatmap3D';
import PayoutCurveChart from './PayoutCurveChart';
import PmaxHistoryChart from './PmaxHistoryChart';
import SkillConfidenceBar from './SkillConfidenceBar';
import SessionTracker from './SessionTracker';

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
  const [selectedHoleId, setSelectedHoleId] = useState<number>(1); // Default to Hole 1
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
    shootOnce,
    shootBatch,
    reset,
  } = useSimulator(handicap, selectedHoleId);

  const handleShoot = () => {
    setIsShooting(true);

    // Simulate shot with optional manual distance
    const distance = devMode ? manualDistance : undefined;
    shootOnce(wager, distance);

    // Brief animation delay
    setTimeout(() => {
      setIsShooting(false);
    }, 500);
  };

  const handleBatchShoot = () => {
    setIsShooting(true);

    // Simulate batch of shots
    shootBatch(wager, batchSize);

    // Brief animation delay
    setTimeout(() => {
      setIsShooting(false);
    }, 500);
  };

  const handleReset = () => {
    if (confirm('Reset session? This will clear all shot data.')) {
      reset();
    }
  };

  const lastShot = shots.length > 0 ? shots[shots.length - 1] : null;

  return (
    <div className="relative bg-gradient-to-br from-[#493b7c]/20 via-[#604c9c]/15 to-[#493b7c]/20 backdrop-blur-2xl rounded-2xl p-4 border border-[#604c9c]/40 shadow-2xl h-full overflow-hidden flex flex-col">
      {/* Frosted glass overlay */}
      <div className="absolute inset-0 bg-gradient-to-b from-white/5 to-transparent pointer-events-none rounded-2xl"></div>

      <div className="relative flex justify-between items-center mb-3">
        <h2 className="text-lg font-semibold text-[#dfc9ad] tracking-tight">
          Continuum Simulator
        </h2>

        {/* Developer Mode Toggle */}
        <div className="flex items-center gap-2">
          <label className="text-xs text-[#9e8cb4]">Developer</label>
          <button
            onClick={() => setDevMode(!devMode)}
            className={`relative inline-flex h-5 w-9 items-center rounded-full transition-all duration-200 ${
              devMode ? 'bg-[#604c9c]' : 'bg-[#493b7c]/30'
            }`}
          >
            <span
              className={`inline-block h-3.5 w-3.5 transform rounded-full bg-[#dfc9ad] transition-transform duration-200 ${
                devMode ? 'translate-x-4' : 'translate-x-0.5'
              }`}
            />
          </button>
        </div>
      </div>

      {/* Main Layout: Responsive grid */}
      <div className="relative grid grid-cols-1 lg:grid-cols-12 gap-3 flex-1 min-h-0">
        {/* Left Column: Controls - Responsive */}
        <div className="lg:col-span-2 space-y-3 overflow-y-auto">
          {/* Frosted Glass Controls */}
          <div className="bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/30 shadow-lg space-y-3">
            {/* Handicap */}
            <div>
              <label className="block text-xs font-medium text-[#9e8cb4] mb-1.5">
                Handicap: {handicap}
              </label>
              <input
                type="range"
                min="0"
                max="30"
                value={handicap}
                onChange={(e) => setHandicap(Number(e.target.value))}
                className="w-full h-1 bg-[#493b7c]/30 rounded-full appearance-none cursor-pointer accent-[#604c9c]"
              />
            </div>

            {/* Wager */}
            <div>
              <label className="block text-xs font-medium text-[#9e8cb4] mb-1.5">
                Wager: ${wager}
              </label>
              <input
                type="range"
                min="1"
                max="100"
                value={wager}
                onChange={(e) => setWager(Number(e.target.value))}
                className="w-full h-1 bg-[#493b7c]/30 rounded-full appearance-none cursor-pointer accent-[#604c9c]"
              />
            </div>

            {/* Hole Selection */}
            <div>
              <label className="block text-xs font-medium text-[#9e8cb4] mb-1.5">
                Hole
              </label>
              <select
                value={selectedHoleId}
                onChange={(e) => setSelectedHoleId(Number(e.target.value))}
                className="w-full bg-[#493b7c]/20 text-[#dfc9ad] border border-[#9e8cb4]/30 rounded-lg px-2 py-1.5 text-xs focus:outline-none focus:border-[#604c9c]"
              >
                {HOLES.map((hole) => (
                  <option key={hole.id} value={hole.id} className="bg-[#1F2937] text-[#dfc9ad]">
                    {hole.name}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {/* Current Hole Info */}
          <div className="bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/30 shadow-lg">
            <h3 className="text-xs font-medium text-[#9e8cb4] mb-2">
              Current Hole
            </h3>
            <div className="space-y-1.5 text-xs">
              <div className="flex justify-between">
                <span className="text-[#9e8cb4]/70">Distance</span>
                <span className="text-[#dfc9ad] font-medium">{currentHole.distance}y</span>
              </div>
              <div className="flex justify-between">
                <span className="text-[#9e8cb4]/70">Target</span>
                <span className="text-[#dfc9ad] font-medium">{currentHole.targetRadius.toFixed(1)}y</span>
              </div>
              <div className="flex justify-between">
                <span className="text-[#9e8cb4]/70">σ</span>
                <span className="text-[#dfc9ad] font-medium">{skillEstimate.sigma.toFixed(2)}y</span>
              </div>
              <div className="flex justify-between">
                <span className="text-[#9e8cb4]/70">P_max</span>
                <span className="text-[#dfc9ad] font-medium">{skillEstimate.pmax.toFixed(3)}</span>
              </div>
            </div>
          </div>

          {/* Session Stats Compact */}
          <SessionTracker stats={sessionStats} />

          {/* Skill Confidence */}
          <SkillConfidenceBar
            confidence={skillEstimate.confidence}
            sigma={skillEstimate.sigma}
            shotsUsed={shots.length}
          />

          {/* Developer Mode Panel */}
          {devMode && (
            <div className="bg-gradient-to-br from-[#604c9c]/20 to-[#493b7c]/20 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/40 shadow-lg">
              <h3 className="text-xs font-medium text-[#604c9c] mb-2">
                Dev Controls
              </h3>
              <div className="space-y-2">
                <div>
                  <label className="block text-xs text-[#9e8cb4]/80 mb-1.5">
                    Manual Miss: {manualDistance.toFixed(1)}y
                  </label>
                  <input
                    type="range"
                    min="0"
                    max="50"
                    step="0.1"
                    value={manualDistance}
                    onChange={(e) => setManualDistance(Number(e.target.value))}
                    className="w-full h-1 bg-[#493b7c]/30 rounded-full appearance-none cursor-pointer accent-[#604c9c]"
                  />
                </div>
              </div>
            </div>
          )}

          {/* Batch Size Control */}
          <div className="bg-gradient-to-br from-[#604c9c]/10 to-[#493b7c]/10 backdrop-blur-xl p-3 rounded-xl border border-[#9e8cb4]/30 shadow-lg">
            <label className="block text-xs font-medium text-[#9e8cb4] mb-1.5">
              Batch Size: {batchSize}
            </label>
            <input
              type="range"
              min="1"
              max="100"
              value={batchSize}
              onChange={(e) => setBatchSize(Number(e.target.value))}
              className="w-full h-1 bg-[#493b7c]/30 rounded-full appearance-none cursor-pointer accent-[#604c9c]"
            />
          </div>

          {/* Action Buttons */}
          <div className="space-y-2">
            <button
              onClick={handleShoot}
              disabled={isShooting}
              className="w-full bg-gradient-to-r from-[#604c9c] to-[#493b7c] hover:from-[#604c9c]/90 hover:to-[#493b7c]/90 text-[#dfc9ad] font-medium py-2.5 px-4 rounded-lg
                         transition-all disabled:opacity-50 disabled:cursor-not-allowed text-sm shadow-lg"
            >
              {isShooting ? 'Shooting...' : 'Shoot 1x'}
            </button>

            <button
              onClick={handleBatchShoot}
              disabled={isShooting}
              className="w-full bg-gradient-to-r from-[#9e8cb4] to-[#604c9c] hover:from-[#9e8cb4]/90 hover:to-[#604c9c]/90 text-[#dfc9ad] font-medium py-2.5 px-4 rounded-lg
                         transition-all disabled:opacity-50 disabled:cursor-not-allowed text-sm shadow-lg"
            >
              {isShooting ? 'Shooting...' : `Shoot ${batchSize}x`}
            </button>

            <button
              onClick={handleReset}
              className="w-full bg-[#493b7c]/20 hover:bg-[#493b7c]/30 text-[#9e8cb4] font-medium py-1.5 px-4 rounded-lg
                         transition-all text-xs border border-[#9e8cb4]/30"
            >
              Reset
            </button>
          </div>
        </div>

        {/* Center Column: Both Visualizers Side by Side (7 cols) */}
        <div className="lg:col-span-7 flex flex-col lg:flex-row gap-3">
          {/* 2D Target View */}
          <div className="flex-1 flex items-center justify-center bg-gradient-to-br from-[#604c9c]/5 to-[#493b7c]/5 backdrop-blur-xl rounded-xl border border-[#9e8cb4]/20">
            <TargetVisualizer
              sigma={skillEstimate.sigma}
              breakevenRadius={breakevenRadius}
              targetRadius={currentHole.targetRadius}
              shots={shots}
              currentShot={isShooting ? lastShot : null}
              width={400}
              height={400}
            />
          </div>

          {/* 3D BVN View */}
          <div className="flex-1">
            <BVNHeatmap3D
              sigmaX={skillEstimate.sigma}
              sigmaY={skillEstimate.sigma}
              currentPmax={skillEstimate.pmax}
              shots={shots}
            />
          </div>
        </div>

        {/* Right Column: Charts (3 cols) */}
        <div className="lg:col-span-3 flex flex-col gap-3">
          <div className="flex-1 min-h-[200px] lg:min-h-0">
            <PayoutCurveChart
              pmax={skillEstimate.pmax}
              breakevenRadius={breakevenRadius}
              targetRadius={currentHole.targetRadius}
              k={currentHole.k}
              lastShotDistance={lastShot?.distance}
              lastShotMultiplier={lastShot?.multiplier}
            />
          </div>
          <div className="flex-1 min-h-[200px] lg:min-h-0">
            <PmaxHistoryChart history={pmaxHistory} />
          </div>
        </div>
      </div>
    </div>
  );
}
