import { useState, useEffect, useRef } from 'react';
import { LineChart, Line, Bar, BarChart, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, ScatterChart, Scatter, ZAxis } from 'recharts';
import init from '../wasm/continuum_golf_simulator';
import { WorkerPool } from '../workers/worker-pool';
import type { BaySimulationRequest } from '../workers/venue-worker';

interface Shot {
  shot_number: number;
  hole_id: number;
  distance_yds: number;
  wager: number;
  miss_distance_ft: number;
  multiplier: number;
  payout: number;
  cumulative_net: number;
  is_fat_tail: boolean;
  p_max: number;
}

interface PlayerResult {
  bay_id: number;
  handicap: number;
  pattern_type: string;
  sigma_x: number;
  sigma_y: number;
  rho: number;
  boundary_points?: Array<[number, number]>;
  total_wagered: number;
  total_won: number;
  net: number;
  rtp: number;
  shots: Shot[];
}

interface VenueResult {
  total_wagered: number;
  total_payouts: number;
  net_profit: number;
  hold_percentage: number;
  total_shots: number;
  num_bays: number;
  avg_rtp: number;
  players: PlayerResult[];
}

export default function VenueSimulator() {
  const [numBays, setNumBays] = useState(10);
  const [shotsPerHour, setShotsPerHour] = useState(10);
  const [hoursOfOperation, setHoursOfOperation] = useState(10);
  const [wager, setWager] = useState(10);
  const [isSimulating, setIsSimulating] = useState(false);
  const [venueResult, setVenueResult] = useState<VenueResult | null>(null);
  const [selectedBay, setSelectedBay] = useState<number | null>(null);
  const [wasmReady, setWasmReady] = useState(false);
  const [simulationProgress, setSimulationProgress] = useState({ completed: 0, total: 0 });
  const workerPoolRef = useRef<WorkerPool | null>(null);

  // Initialize WASM module and worker pool
  useEffect(() => {
    init()
      .then(() => {
        setWasmReady(true);
        console.log('✅ WASM module initialized for Venue Simulator');

        // Initialize worker pool (will use navigator.hardwareConcurrency)
        workerPoolRef.current = new WorkerPool();
      })
      .catch((error) => {
        console.error('❌ Failed to initialize WASM:', error);
      });

    // Cleanup worker pool on unmount
    return () => {
      if (workerPoolRef.current) {
        workerPoolRef.current.terminate();
        workerPoolRef.current = null;
      }
    };
  }, []);

  const runSimulation = async () => {
    if (!wasmReady || !workerPoolRef.current) {
      alert('System not ready. Please wait...');
      return;
    }

    setIsSimulating(true);
    setSimulationProgress({ completed: 0, total: numBays });

    try {
      const shotsPerBay = shotsPerHour * hoursOfOperation;

      console.log(`🚀 Starting parallel venue simulation: ${numBays} bays × ${shotsPerBay} shots`);
      console.log(`Using ${navigator.hardwareConcurrency || 8} CPU cores`);

      // Generate player handicaps (bell curve distribution)
      const handicaps: number[] = [];
      for (let i = 0; i < numBays; i++) {
        // Normal distribution: mean=15, std_dev=7
        const u1 = Math.random();
        const u2 = Math.random();
        const z = Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
        const handicap = Math.round(Math.max(0, Math.min(30, 15 + 7 * z)));
        handicaps.push(handicap);
      }

      // Create simulation requests for each bay
      const requests: BaySimulationRequest[] = handicaps.map((handicap, index) => ({
        bayId: index + 1,
        handicap,
        shotsPerBay,
        wager,
      }));

      // Run parallel simulation using worker pool
      const startTime = performance.now();

      const playerResults = await workerPoolRef.current.simulateBays(
        requests,
        (completed, total) => {
          setSimulationProgress({ completed, total });
          console.log(`Progress: ${completed}/${total} bays completed`);
        }
      );

      const endTime = performance.now();
      console.log(`✅ Simulation completed in ${((endTime - startTime) / 1000).toFixed(2)}s`);

      // Aggregate results
      const totalWagered = playerResults.reduce((sum, p) => sum + p.total_wagered, 0);
      const totalPayouts = playerResults.reduce((sum, p) => sum + p.total_won, 0);
      const netProfit = totalWagered - totalPayouts;
      const holdPercentage = totalWagered > 0 ? (netProfit / totalWagered) * 100 : 0;
      const totalShots = numBays * shotsPerBay;
      const avgRtp = totalWagered > 0 ? (totalPayouts / totalWagered) * 100 : 0;

      const result: VenueResult = {
        total_wagered: totalWagered,
        total_payouts: totalPayouts,
        net_profit: netProfit,
        hold_percentage: holdPercentage,
        total_shots: totalShots,
        num_bays: numBays,
        avg_rtp: avgRtp,
        players: playerResults,
      };

      console.log('Venue simulation result:', result);
      setVenueResult(result);

      // Select first bay by default
      if (result.players && result.players.length > 0) {
        setSelectedBay(result.players[0].bay_id);
      }
    } catch (error) {
      console.error('Simulation error:', error);
      alert('Failed to run simulation. Please check console for details.');
    } finally {
      setIsSimulating(false);
      setSimulationProgress({ completed: 0, total: 0 });
    }
  };

  const selectedPlayer = venueResult?.players.find(p => p.bay_id === selectedBay);

  // Prepare data for visualizations
  const bayPerformanceData = venueResult?.players.map(p => ({
    bay: `Bay ${p.bay_id}`,
    rtp: p.rtp,
    handicap: p.handicap,
    pattern: p.pattern_type,
    net: p.net,
  })) || [];

  // Calculate hole profitability across all bays
  const holeProfitability = venueResult?.players.reduce((acc, player) => {
    player.shots.forEach(shot => {
      const holeKey = `${shot.distance_yds}y`;
      if (!acc[holeKey]) {
        acc[holeKey] = { distance: shot.distance_yds, profit: 0, count: 0 };
      }
      acc[holeKey].profit += (shot.wager - shot.payout);
      acc[holeKey].count += 1;
    });
    return acc;
  }, {} as Record<string, { distance: number; profit: number; count: number }>);

  const holeProfitData = holeProfitability
    ? Object.values(holeProfitability)
        .sort((a, b) => a.distance - b.distance)
        .map(({ distance, profit, count }) => ({
          hole: `${distance}y`,
          avgProfit: profit / count,
          totalProfit: profit,
        }))
    : [];

  const handicapDistribution = venueResult?.players.map(p => ({
    handicap: p.handicap,
    rtp: p.rtp,
    bay: p.bay_id,
  })) || [];

  return (
    <div className="h-full w-full bg-gradient-to-br from-[var(--brand-deep-purple)]/30 to-[#000000] rounded-2xl p-4 backdrop-blur-3xl overflow-hidden flex flex-col">
      {/* Glass morphism overlay */}
      <div className="absolute inset-0 bg-gradient-to-br from-[var(--brand-bright-purple)]/[0.15] via-transparent to-[var(--brand-dark-gold)]/[0.05] pointer-events-none rounded-2xl" />

      <div className="relative mb-3">
        <h2 className="text-2xl font-bold text-white mb-1">Venue Simulation</h2>
        <p className="text-white/70 text-xs">
          Simulate multiple bays with diverse player patterns and analyze venue performance
        </p>
      </div>

      {/* Controls */}
      <div className="relative bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-xl p-3 border border-white/10 mb-3">
        <div className="grid grid-cols-5 gap-3">
          <div>
            <label className="block text-xs font-medium text-white/70 mb-1">
              Number of Bays: {numBays}
            </label>
            <input
              type="range"
              min="1"
              max="30"
              value={numBays}
              onChange={(e) => setNumBays(Number(e.target.value))}
              className="w-full h-1.5 bg-[var(--brand-dark-gray)] rounded-full appearance-none cursor-pointer"
              disabled={isSimulating}
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-white/70 mb-1">
              Shots per Hour: {shotsPerHour}
            </label>
            <input
              type="range"
              min="1"
              max="50"
              value={shotsPerHour}
              onChange={(e) => setShotsPerHour(Number(e.target.value))}
              className="w-full h-1.5 bg-[var(--brand-dark-gray)] rounded-full appearance-none cursor-pointer"
              disabled={isSimulating}
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-white/70 mb-1">
              Hours of Operation: {hoursOfOperation}
            </label>
            <input
              type="range"
              min="1"
              max="24"
              value={hoursOfOperation}
              onChange={(e) => setHoursOfOperation(Number(e.target.value))}
              className="w-full h-1.5 bg-[var(--brand-dark-gray)] rounded-full appearance-none cursor-pointer"
              disabled={isSimulating}
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-white/70 mb-1">
              Wager: ${wager}
            </label>
            <input
              type="range"
              min="1"
              max="100"
              value={wager}
              onChange={(e) => setWager(Number(e.target.value))}
              className="w-full h-1.5 bg-[var(--brand-dark-gray)] rounded-full appearance-none cursor-pointer"
              disabled={isSimulating}
            />
          </div>
          <div className="flex items-end">
            <button
              onClick={runSimulation}
              disabled={isSimulating || !wasmReady}
              className="w-full bg-gradient-to-r from-[var(--brand-bright-purple)] to-[var(--brand-deep-purple)] hover:from-[var(--brand-deep-purple)] hover:to-[var(--brand-bright-purple)] text-white font-medium py-2 px-4 rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed shadow-lg shadow-[var(--brand-bright-purple)]/25"
            >
              {!wasmReady ? 'Loading...' : isSimulating ? 'Simulating...' : 'Run Simulation'}
            </button>
          </div>
        </div>
      </div>

      {venueResult && (
        <div className="relative flex-1 grid grid-cols-12 gap-3 min-h-0">
          {/* Left Column - Overall Stats */}
          <div className="col-span-4 flex flex-col gap-3 overflow-y-auto">
            {/* Summary Stats */}
            <div className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-xl p-3 border border-white/10">
              <h3 className="text-sm font-semibold text-white mb-2">Venue Performance</h3>
              <div className="grid grid-cols-2 gap-2">
                <div className="bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                  <div className="text-[10px] text-white/70 mb-0.5">Total Wagered</div>
                  <div className="text-base font-bold text-white">
                    ${venueResult.total_wagered.toFixed(2)}
                  </div>
                </div>
                <div className="bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                  <div className="text-[10px] text-white/70 mb-0.5">Total Payouts</div>
                  <div className="text-base font-bold text-white">
                    ${venueResult.total_payouts.toFixed(2)}
                  </div>
                </div>
                <div className="bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                  <div className="text-[10px] text-white/70 mb-0.5">Net Profit</div>
                  <div className="text-base font-bold text-white">
                    ${venueResult.net_profit.toFixed(2)}
                  </div>
                </div>
                <div className="bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                  <div className="text-[10px] text-white/70 mb-0.5">Hold %</div>
                  <div className="text-base font-bold text-white">
                    {venueResult.hold_percentage.toFixed(2)}%
                  </div>
                </div>
                <div className="col-span-2 bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                  <div className="text-[10px] text-white/70 mb-0.5">Average RTP</div>
                  <div className="text-xl font-bold text-white">
                    {venueResult.avg_rtp.toFixed(2)}%
                  </div>
                </div>
              </div>
            </div>

            {/* Hole Profitability */}
            <div className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-xl p-3 border border-white/10 flex-1 min-h-0 flex flex-col">
              <h3 className="text-sm font-semibold text-white mb-2">Hole Profitability</h3>
              <div className="flex-1 min-h-0">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={holeProfitData}>
                    <CartesianGrid strokeDasharray="3 3" stroke="var(--brand-lavender)" opacity={0.1} />
                    <XAxis
                      dataKey="hole"
                      stroke="var(--brand-lavender)"
                      tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                      label={{ value: 'Distance', position: 'insideBottom', offset: -5, fill: 'var(--brand-lavender)' }}
                    />
                    <YAxis
                      stroke="var(--brand-lavender)"
                      tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                      label={{ value: 'Avg Profit ($)', angle: -90, position: 'insideLeft', fill: 'var(--brand-lavender)' }}
                    />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: 'var(--brand-deep-purple)',
                        border: '1px solid var(--brand-tan)',
                        borderRadius: '8px',
                        color: 'var(--brand-tan)',
                      }}
                      formatter={(value: number) => [`$${value.toFixed(2)}`, 'Avg Profit']}
                    />
                    <Bar dataKey="avgProfit" fill="var(--brand-dark-gold)" name="Avg Profit per Shot" />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            </div>

            {/* Handicap vs RTP Scatter */}
            <div className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-xl p-3 border border-white/10 flex-1 min-h-0 flex flex-col">
              <h3 className="text-sm font-semibold text-white mb-2">Handicap vs RTP</h3>
              <div className="flex-1 min-h-0">
                <ResponsiveContainer width="100%" height="100%">
                  <ScatterChart>
                    <CartesianGrid strokeDasharray="3 3" stroke="var(--brand-lavender)" opacity={0.1} />
                    <XAxis
                      dataKey="handicap"
                      name="Handicap"
                      stroke="var(--brand-lavender)"
                      tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                      label={{ value: 'Handicap', position: 'insideBottom', offset: -5, fill: 'var(--brand-lavender)' }}
                    />
                    <YAxis
                      dataKey="rtp"
                      name="RTP %"
                      stroke="var(--brand-lavender)"
                      tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                      label={{ value: 'RTP %', angle: -90, position: 'insideLeft', fill: 'var(--brand-lavender)' }}
                    />
                    <ZAxis range={[30, 100]} />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: 'var(--brand-deep-purple)',
                        border: '1px solid var(--brand-tan)',
                        borderRadius: '8px',
                        color: 'var(--brand-tan)',
                      }}
                      cursor={{ strokeDasharray: '3 3' }}
                    />
                    <Scatter name="Players" data={handicapDistribution} fill="var(--brand-lavender)" />
                  </ScatterChart>
                </ResponsiveContainer>
              </div>
            </div>
          </div>

          {/* Middle Column - Bay Performance */}
          <div className="col-span-4 flex flex-col gap-3 overflow-y-auto">
            <div className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-xl p-3 border border-white/10 flex-1 min-h-0 flex flex-col">
              <h3 className="text-sm font-semibold text-white mb-2">Bay Performance</h3>
              <div className="flex-1 min-h-0">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={bayPerformanceData}>
                    <CartesianGrid strokeDasharray="3 3" stroke="var(--brand-lavender)" opacity={0.1} />
                    <XAxis
                      dataKey="bay"
                      stroke="var(--brand-lavender)"
                      tick={{ fontSize: 9, fill: 'var(--brand-lavender)' }}
                      angle={-45}
                      textAnchor="end"
                      height={60}
                    />
                    <YAxis
                      stroke="var(--brand-lavender)"
                      tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                      label={{ value: 'RTP %', angle: -90, position: 'insideLeft', fill: 'var(--brand-lavender)' }}
                    />
                    <Tooltip
                      contentStyle={{
                        backgroundColor: 'var(--brand-deep-purple)',
                        border: '1px solid var(--brand-tan)',
                        borderRadius: '8px',
                        color: 'var(--brand-tan)',
                      }}
                    />
                    <Legend />
                    <Bar dataKey="rtp" fill="var(--brand-bright-purple)" name="RTP %" />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            </div>

            {/* Bay Selector */}
            <div className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-xl p-3 border border-white/10">
              <h3 className="text-sm font-semibold text-white mb-2">Select Bay</h3>
              <div className="grid grid-cols-6 gap-2">
                {venueResult.players.map(p => (
                  <button
                    key={p.bay_id}
                    onClick={() => setSelectedBay(p.bay_id)}
                    className={`py-1.5 px-2 rounded-lg text-xs font-medium transition-all ${
                      selectedBay === p.bay_id
                        ? 'bg-[var(--brand-bright-purple)] text-white border-2 border-[var(--brand-lavender)]'
                        : 'bg-[var(--brand-bright-purple)]/20 text-white/70 border border-[var(--brand-lavender)]/30 hover:bg-[var(--brand-bright-purple)]/40'
                    }`}
                  >
                    #{p.bay_id}
                  </button>
                ))}
              </div>
            </div>

            {/* Selected Bay Details */}
            {selectedPlayer && (
              <div className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-xl p-3 border border-white/10">
                <h3 className="text-sm font-semibold text-white mb-2">
                  Bay #{selectedPlayer.bay_id} Details
                </h3>
                <div className="grid grid-cols-2 gap-2 text-xs">
                  <div>
                    <span className="text-white/70">Handicap:</span>
                    <span className="text-white font-semibold ml-1">{selectedPlayer.handicap}</span>
                  </div>
                  <div>
                    <span className="text-white/70">Pattern:</span>
                    <span className="text-white font-semibold ml-1">{selectedPlayer.pattern_type}</span>
                  </div>
                  <div>
                    <span className="text-white/70">σ_x:</span>
                    <span className="text-white font-semibold ml-1">{selectedPlayer.sigma_x.toFixed(1)}ft</span>
                  </div>
                  <div>
                    <span className="text-white/70">σ_y:</span>
                    <span className="text-white font-semibold ml-1">{selectedPlayer.sigma_y.toFixed(1)}ft</span>
                  </div>
                  <div>
                    <span className="text-white/70">RTP:</span>
                    <span className="text-white font-semibold ml-1">{selectedPlayer.rtp.toFixed(2)}%</span>
                  </div>
                  <div>
                    <span className="text-white/70">Net:</span>
                    <span className="text-white font-semibold ml-1">${selectedPlayer.net.toFixed(2)}</span>
                  </div>
                </div>
              </div>
            )}

            {/* Pattern Visualization */}
            {selectedPlayer && selectedPlayer.boundary_points && (
              <div className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-xl p-3 border border-white/10">
                <h3 className="text-sm font-semibold text-white mb-2">
                  Shot Pattern
                </h3>
                <div className="flex justify-center">
                  <canvas
                    ref={(canvas) => {
                      if (canvas && selectedPlayer.boundary_points) {
                        const ctx = canvas.getContext('2d');
                        if (!ctx) return;

                        const width = 200;
                        const height = 200;
                        canvas.width = width;
                        canvas.height = height;

                        // Clear canvas
                        ctx.fillStyle = 'rgba(0, 0, 0, 0.3)';
                        ctx.fillRect(0, 0, width, height);

                        // Find bounds
                        const points = selectedPlayer.boundary_points;
                        const xs = points.map((p: [number, number]) => p[0]);
                        const ys = points.map((p: [number, number]) => p[1]);
                        const minX = Math.min(...xs);
                        const maxX = Math.max(...xs);
                        const minY = Math.min(...ys);
                        const maxY = Math.max(...ys);

                        // Add padding
                        const padding = 20;
                        const rangeX = maxX - minX;
                        const rangeY = maxY - minY;
                        const maxRange = Math.max(rangeX, rangeY);
                        const scale = (width - 2 * padding) / maxRange;

                        // Center the pattern
                        const centerX = width / 2;
                        const centerY = height / 2;
                        const patternCenterX = (minX + maxX) / 2;
                        const patternCenterY = (minY + maxY) / 2;

                        // Draw target (hole)
                        ctx.fillStyle = 'var(--brand-tan)';
                        ctx.beginPath();
                        ctx.arc(centerX, centerY, 3, 0, 2 * Math.PI);
                        ctx.fill();

                        // Draw pattern boundary
                        ctx.strokeStyle = 'var(--brand-bright-purple)';
                        ctx.lineWidth = 2;
                        ctx.beginPath();
                        points.forEach((point: [number, number], i: number) => {
                          const x = centerX + (point[0] - patternCenterX) * scale;
                          const y = centerY - (point[1] - patternCenterY) * scale; // Flip Y
                          if (i === 0) {
                            ctx.moveTo(x, y);
                          } else {
                            ctx.lineTo(x, y);
                          }
                        });
                        ctx.closePath();
                        ctx.stroke();

                        // Fill pattern with semi-transparent color
                        ctx.fillStyle = 'rgba(156, 108, 210, 0.2)';
                        ctx.fill();

                        // Draw axes
                        ctx.strokeStyle = 'var(--brand-lavender)';
                        ctx.lineWidth = 1;
                        ctx.setLineDash([2, 2]);
                        ctx.beginPath();
                        ctx.moveTo(centerX, 0);
                        ctx.lineTo(centerX, height);
                        ctx.moveTo(0, centerY);
                        ctx.lineTo(width, centerY);
                        ctx.stroke();
                        ctx.setLineDash([]);
                      }
                    }}
                    className="border border-[var(--brand-lavender)]/30 rounded-lg"
                  />
                </div>
                <div className="text-xs text-white/70 text-center mt-2">
                  Organic pattern boundary (100 points)
                </div>
              </div>
            )}
          </div>

          {/* Right Column - Shot Distribution & RTP Over Time */}
          <div className="col-span-4 flex flex-col gap-3 overflow-y-auto">
            {selectedPlayer && (
              <>
                {/* RTP Evolution */}
                <div className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-xl p-3 border border-white/10 flex-1 min-h-0 flex flex-col">
                  <h3 className="text-sm font-semibold text-white mb-2">
                    Bay #{selectedPlayer.bay_id} RTP Evolution
                  </h3>
                  <div className="flex-1 min-h-0">
                    <ResponsiveContainer width="100%" height="100%">
                      <LineChart data={selectedPlayer.shots.map((_shot, idx) => {
                        const cumulativeWager = selectedPlayer.shots.slice(0, idx + 1).reduce((sum, s) => sum + s.wager, 0);
                        const cumulativePayout = selectedPlayer.shots.slice(0, idx + 1).reduce((sum, s) => sum + s.payout, 0);
                        return {
                          shot: idx + 1,
                          rtp: (cumulativePayout / cumulativeWager) * 100,
                        };
                      })}>
                        <CartesianGrid strokeDasharray="3 3" stroke="var(--brand-lavender)" opacity={0.1} />
                        <XAxis
                          dataKey="shot"
                          stroke="var(--brand-lavender)"
                          tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                          label={{ value: 'Shot #', position: 'insideBottom', offset: -5, fill: 'var(--brand-lavender)' }}
                        />
                        <YAxis
                          stroke="var(--brand-lavender)"
                          tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                          label={{ value: 'RTP %', angle: -90, position: 'insideLeft', fill: 'var(--brand-lavender)' }}
                        />
                        <Tooltip
                          contentStyle={{
                            backgroundColor: 'var(--brand-deep-purple)',
                            border: '1px solid var(--brand-tan)',
                            borderRadius: '8px',
                            color: 'var(--brand-tan)',
                          }}
                        />
                        <Line type="monotone" dataKey="rtp" stroke="var(--brand-bright-purple)" strokeWidth={2} dot={false} />
                        <Line type="monotone" dataKey={() => 85} stroke="var(--brand-lavender)" strokeWidth={1} strokeDasharray="5 5" dot={false} name="Target (85%)" />
                      </LineChart>
                    </ResponsiveContainer>
                  </div>
                </div>

                {/* Payout Distribution */}
                <div className="bg-gradient-to-br from-black/60 to-black/40 backdrop-blur-xl rounded-xl p-3 border border-white/10 flex-1 min-h-0 flex flex-col">
                  <h3 className="text-sm font-semibold text-white mb-2">
                    Payout Distribution
                  </h3>
                  <div className="flex-1 min-h-0">
                    <ResponsiveContainer width="100%" height="100%">
                      <BarChart data={(() => {
                        const buckets = Array(11).fill(0);
                        selectedPlayer.shots.forEach(shot => {
                          const bucket = Math.min(Math.floor(shot.multiplier), 10);
                          buckets[bucket]++;
                        });
                        return buckets.map((count, idx) => ({
                          multiplier: idx === 10 ? '10+' : `${idx}x`,
                          count,
                        }));
                      })()}>
                        <CartesianGrid strokeDasharray="3 3" stroke="var(--brand-lavender)" opacity={0.1} />
                        <XAxis
                          dataKey="multiplier"
                          stroke="var(--brand-lavender)"
                          tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                        />
                        <YAxis
                          stroke="var(--brand-lavender)"
                          tick={{ fontSize: 10, fill: 'var(--brand-lavender)' }}
                        />
                        <Tooltip
                          contentStyle={{
                            backgroundColor: 'var(--brand-deep-purple)',
                            border: '1px solid var(--brand-tan)',
                            borderRadius: '8px',
                            color: 'var(--brand-tan)',
                          }}
                        />
                        <Bar dataKey="count" fill="var(--brand-dark-gold)" />
                      </BarChart>
                    </ResponsiveContainer>
                  </div>
                </div>
              </>
            )}
          </div>
        </div>
      )}

      {!venueResult && !isSimulating && (
        <div className="relative flex-1 flex items-center justify-center">
          <div className="text-center">
            <div className="text-white/70 mb-3">
              <svg className="w-16 h-16 mx-auto opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
              </svg>
            </div>
            <p className="text-white/70 text-sm">Configure simulation parameters and click "Run Simulation"</p>
          </div>
        </div>
      )}

      {isSimulating && (
        <div className="relative flex-1 flex items-center justify-center">
          <div className="w-full max-w-md px-8">
            <div className="text-center mb-6">
              <div className="text-white text-xl font-bold mb-2">Simulating Venue...</div>
              <p className="text-white/70 text-sm mb-2">
                Running {numBays} bays × {shotsPerHour * hoursOfOperation} shots
              </p>
              <p className="text-[var(--brand-dark-gold)] text-xs">
                Using {navigator.hardwareConcurrency || 8} CPU cores in parallel
              </p>
            </div>

            {/* Progress Bar */}
            <div className="relative h-3 bg-black/30 rounded-full overflow-hidden border border-white/10 mb-3">
              <div
                className="absolute inset-0 bg-gradient-to-r from-[var(--brand-bright-purple)] to-[var(--brand-dark-gold)] transition-all duration-300"
                style={{
                  width: `${simulationProgress.total > 0 ? (simulationProgress.completed / simulationProgress.total) * 100 : 0}%`,
                }}
              ></div>
              <div
                className="absolute inset-0 bg-gradient-to-r from-[var(--brand-bright-purple)] to-[var(--brand-dark-gold)] animate-[shimmer_1.5s_ease-in-out_infinite] opacity-50"
                style={{
                  backgroundSize: '200% 100%',
                  width: `${simulationProgress.total > 0 ? (simulationProgress.completed / simulationProgress.total) * 100 : 100}%`,
                }}
              ></div>
            </div>

            {/* Progress Text */}
            <div className="text-center mb-6">
              <p className="text-white/70 text-sm font-semibold">
                {simulationProgress.completed} / {simulationProgress.total} bays completed
                {simulationProgress.total > 0 && (
                  <span className="ml-2 text-[var(--brand-dark-gold)]">
                    ({Math.round((simulationProgress.completed / simulationProgress.total) * 100)}%)
                  </span>
                )}
              </p>
            </div>

            {/* Loading Animation */}
            <div className="mt-6 flex justify-center space-x-2">
              <div className="w-3 h-3 bg-[var(--brand-bright-purple)] rounded-full animate-bounce" style={{ animationDelay: '0ms' }}></div>
              <div className="w-3 h-3 bg-[#9e8cb4] rounded-full animate-bounce" style={{ animationDelay: '150ms' }}></div>
              <div className="w-3 h-3 bg-[var(--brand-dark-gold)] rounded-full animate-bounce" style={{ animationDelay: '300ms' }}></div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
