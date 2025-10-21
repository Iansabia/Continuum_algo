import { useState } from 'react';
import { useSimulator } from '../hooks/useSimulator';
import TargetVisualizer from './TargetVisualizer';
import PayoutCurveChart from './PayoutCurveChart';
import PmaxHistoryChart from './PmaxHistoryChart';
import SkillConfidenceBar from './SkillConfidenceBar';
import SessionTracker from './SessionTracker';

export default function PlayerSimulator() {
  const [handicap, setHandicap] = useState(10);
  const [wager, setWager] = useState(10);
  const [devMode, setDevMode] = useState(false);
  const [manualDistance, setManualDistance] = useState(5);
  const [isShooting, setIsShooting] = useState(false);

  const {
    shots,
    skillEstimate,
    pmaxHistory,
    sessionStats,
    breakevenRadius,
    shootOnce,
    reset,
  } = useSimulator(handicap);

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

  const handleReset = () => {
    if (confirm('Reset session? This will clear all shot data.')) {
      reset();
    }
  };

  const lastShot = shots.length > 0 ? shots[shots.length - 1] : null;

  return (
    <div className="bg-gray-800/50 backdrop-blur-sm rounded-lg p-8 border border-brand-deep-purple">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-3xl font-montserrat font-bold text-brand-tan">
          Continuum™ Player Simulator
        </h2>

        {/* Developer Mode Toggle */}
        <div className="flex items-center gap-3">
          <label className="text-sm text-gray-400">Developer Mode</label>
          <button
            onClick={() => setDevMode(!devMode)}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
              devMode ? 'bg-brand-bright-purple' : 'bg-gray-600'
            }`}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                devMode ? 'translate-x-6' : 'translate-x-1'
              }`}
            />
          </button>
        </div>
      </div>

      {/* Main Layout: 3 columns */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-8">
        {/* Left Column: Controls */}
        <div className="space-y-6">
          {/* Handicap Slider */}
          <div className="bg-gray-800 p-4 rounded-lg border-2 border-brand-deep-purple">
            <label className="block text-sm font-medium text-brand-tan mb-2">
              Handicap: {handicap}
            </label>
            <input
              type="range"
              min="0"
              max="30"
              value={handicap}
              onChange={(e) => setHandicap(Number(e.target.value))}
              className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
            />
            <p className="text-xs text-gray-500 mt-2">
              Initial skill estimate based on handicap
            </p>
          </div>

          {/* Wager Input */}
          <div className="bg-gray-800 p-4 rounded-lg border-2 border-brand-deep-purple">
            <label className="block text-sm font-medium text-brand-tan mb-2">
              Wager Amount: ${wager}
            </label>
            <input
              type="range"
              min="1"
              max="100"
              value={wager}
              onChange={(e) => setWager(Number(e.target.value))}
              className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
            />
            <p className="text-xs text-gray-500 mt-2">
              Amount to wager on next shot
            </p>
          </div>

          {/* Developer Mode Panel */}
          {devMode && (
            <div className="bg-gray-800 p-4 rounded-lg border-2 border-brand-bright-purple">
              <h3 className="text-sm font-semibold text-brand-bright-purple mb-3">
                🛠️ Developer Controls
              </h3>
              <div className="space-y-3">
                <div>
                  <label className="block text-xs text-gray-400 mb-1">
                    Manual Miss Distance (yards): {manualDistance.toFixed(1)}
                  </label>
                  <input
                    type="range"
                    min="0"
                    max="50"
                    step="0.1"
                    value={manualDistance}
                    onChange={(e) => setManualDistance(Number(e.target.value))}
                    className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
                  />
                </div>
                <div className="bg-gray-900 p-2 rounded text-xs">
                  <p className="text-gray-500">Kalman State:</p>
                  <p className="text-brand-lavender">σ = {skillEstimate.sigma.toFixed(2)}y</p>
                  <p className="text-brand-lavender">Conf = {skillEstimate.confidence.toFixed(1)}%</p>
                  <p className="text-brand-lavender">P_max = {skillEstimate.pmax.toFixed(3)}</p>
                </div>
              </div>
            </div>
          )}

          {/* Shoot Button */}
          <button
            onClick={handleShoot}
            disabled={isShooting}
            className="w-full bg-brand-bright-purple hover:bg-brand-lavender text-white font-montserrat font-bold py-4 px-8 rounded-lg
                       transition-colors disabled:opacity-50 disabled:cursor-not-allowed shadow-lg"
          >
            {isShooting ? 'Shooting...' : '⛳ Shoot'}
          </button>

          {/* Reset Button */}
          <button
            onClick={handleReset}
            className="w-full bg-gray-700 hover:bg-gray-600 text-gray-300 font-semibold py-2 px-4 rounded-lg
                       transition-colors"
          >
            Reset Session
          </button>
        </div>

        {/* Center Column: Target Visualizer */}
        <div>
          <TargetVisualizer
            sigma={skillEstimate.sigma}
            breakevenRadius={breakevenRadius}
            shots={shots}
            currentShot={isShooting ? lastShot : null}
            width={400}
            height={400}
          />
        </div>

        {/* Right Column: Session Info */}
        <div className="space-y-6">
          <SessionTracker stats={sessionStats} />
          <SkillConfidenceBar
            confidence={skillEstimate.confidence}
            sigma={skillEstimate.sigma}
            shotsUsed={shots.length}
          />
        </div>
      </div>

      {/* Charts Section */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <PayoutCurveChart
          pmax={skillEstimate.pmax}
          breakevenRadius={breakevenRadius}
          lastShotDistance={lastShot?.distance}
          lastShotMultiplier={lastShot?.multiplier}
        />
        <PmaxHistoryChart history={pmaxHistory} />
      </div>

      {/* Educational Info */}
      <div className="mt-8 bg-brand-deep-purple/20 border border-brand-deep-purple rounded-lg p-6">
        <h3 className="text-lg font-semibold text-brand-tan mb-3">
          How It Works
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-gray-300">
          <div>
            <h4 className="font-semibold text-brand-lavender mb-2">Kalman Filter Adaptation</h4>
            <p>
              The system uses a Kalman filter to estimate your skill (σ) in real-time.
              Skill updates occur every 5 shots or immediately when you place a high-stakes wager (10x average).
            </p>
          </div>
          <div>
            <h4 className="font-semibold text-brand-lavender mb-2">Dynamic Pricing</h4>
            <p>
              Payout multipliers are calculated as P_max × e^(-d²/(2r_max²)), where d is miss distance.
              The breakeven radius shows where multiplier = 1.0x.
            </p>
          </div>
          <div>
            <h4 className="font-semibold text-brand-lavender mb-2">Rayleigh Distribution</h4>
            <p>
              Shot dispersion follows a Rayleigh distribution (2D miss pattern).
              With 2% probability, shots may experience 3× worse dispersion (fat-tail events).
            </p>
          </div>
          <div>
            <h4 className="font-semibold text-brand-lavender mb-2">House Edge</h4>
            <p>
              Theoretical house edge is ~15% with optimized P_max.
              Actual edge converges to this value over many shots due to variance.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
