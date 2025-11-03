import { useState, useEffect } from 'react';
import { LineChart, Line, Bar, BarChart, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, ScatterChart, Scatter, ZAxis } from 'recharts';
import init from '../wasm/continuum_golf_simulator';

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

  // Initialize WASM module
  useEffect(() => {
    init()
      .then(() => {
        setWasmReady(true);
        console.log('✅ WASM module initialized for Venue Simulator');
      })
      .catch((error) => {
        console.error('❌ Failed to initialize WASM:', error);
      });
  }, []);

  const runSimulation = async () => {
    if (!wasmReady) {
      alert('WASM module not ready. Please wait...');
      return;
    }

    setIsSimulating(true);

    try {
      // Dynamically import WASM module
      const { simulate_venue_enhanced } = await import('../wasm/continuum_golf_simulator');

      // Run enhanced venue simulation
      const result = simulate_venue_enhanced(numBays, shotsPerHour, hoursOfOperation, wager);

      console.log('Venue simulation result:', result);
      setVenueResult(result as VenueResult);

      // Select first bay by default
      if (result.players && result.players.length > 0) {
        setSelectedBay(result.players[0].bay_id);
      }
    } catch (error) {
      console.error('Simulation error:', error);
      alert('Failed to run simulation. Please check console for details.');
    } finally {
      setIsSimulating(false);
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

  const patternDistribution = venueResult?.players.reduce((acc, p) => {
    acc[p.pattern_type] = (acc[p.pattern_type] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  const patternData = patternDistribution ? Object.entries(patternDistribution).map(([pattern, count]) => ({
    pattern: pattern.charAt(0).toUpperCase() + pattern.slice(1),
    count,
  })) : [];

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
        <h2 className="text-2xl font-bold text-[var(--brand-tan)] mb-1">Venue Simulation</h2>
        <p className="text-[var(--brand-lavender)] text-xs">
          Simulate multiple bays with diverse player patterns and analyze venue performance
        </p>
      </div>

      {/* Controls */}
      <div className="relative bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-xl p-3 border border-[var(--brand-tan)]/20 mb-3">
        <div className="grid grid-cols-5 gap-3">
          <div>
            <label className="block text-xs font-medium text-[var(--brand-lavender)] mb-1">
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
            <label className="block text-xs font-medium text-[var(--brand-lavender)] mb-1">
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
            <label className="block text-xs font-medium text-[var(--brand-lavender)] mb-1">
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
            <label className="block text-xs font-medium text-[var(--brand-lavender)] mb-1">
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
              className="w-full bg-gradient-to-r from-[var(--brand-bright-purple)] to-[var(--brand-deep-purple)] hover:from-[var(--brand-deep-purple)] hover:to-[var(--brand-bright-purple)] text-[var(--brand-tan)] font-medium py-2 px-4 rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed shadow-lg shadow-[var(--brand-bright-purple)]/25"
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
            <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-xl p-3 border border-[var(--brand-tan)]/20">
              <h3 className="text-sm font-semibold text-[var(--brand-tan)] mb-2">Venue Performance</h3>
              <div className="grid grid-cols-2 gap-2">
                <div className="bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                  <div className="text-[10px] text-[var(--brand-lavender)] mb-0.5">Total Wagered</div>
                  <div className="text-base font-bold text-[var(--brand-tan)]">
                    ${venueResult.total_wagered.toFixed(2)}
                  </div>
                </div>
                <div className="bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                  <div className="text-[10px] text-[var(--brand-lavender)] mb-0.5">Total Payouts</div>
                  <div className="text-base font-bold text-[var(--brand-tan)]">
                    ${venueResult.total_payouts.toFixed(2)}
                  </div>
                </div>
                <div className="bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                  <div className="text-[10px] text-[var(--brand-lavender)] mb-0.5">Net Profit</div>
                  <div className="text-base font-bold text-[var(--brand-tan)]">
                    ${venueResult.net_profit.toFixed(2)}
                  </div>
                </div>
                <div className="bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                  <div className="text-[10px] text-[var(--brand-lavender)] mb-0.5">Hold %</div>
                  <div className="text-base font-bold text-[var(--brand-tan)]">
                    {venueResult.hold_percentage.toFixed(2)}%
                  </div>
                </div>
                <div className="col-span-2 bg-[var(--brand-bright-purple)]/10 rounded-lg p-2">
                  <div className="text-[10px] text-[var(--brand-lavender)] mb-0.5">Average RTP</div>
                  <div className="text-xl font-bold text-[var(--brand-tan)]">
                    {venueResult.avg_rtp.toFixed(2)}%
                  </div>
                </div>
              </div>
            </div>

            {/* Pattern Distribution */}
            <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-xl p-3 border border-[var(--brand-tan)]/20 flex-1 min-h-0 flex flex-col">
              <h3 className="text-sm font-semibold text-[var(--brand-tan)] mb-2">Pattern Distribution</h3>
              <div className="flex-1 min-h-0">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={patternData}>
                    <CartesianGrid strokeDasharray="3 3" stroke="var(--brand-lavender)" opacity={0.1} />
                    <XAxis
                      dataKey="pattern"
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
                    <Bar dataKey="count" fill="var(--brand-bright-purple)" />
                  </BarChart>
                </ResponsiveContainer>
              </div>
            </div>

            {/* Handicap vs RTP Scatter */}
            <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-xl p-3 border border-[var(--brand-tan)]/20 flex-1 min-h-0 flex flex-col">
              <h3 className="text-sm font-semibold text-[var(--brand-tan)] mb-2">Handicap vs RTP</h3>
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
            <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-xl p-3 border border-[var(--brand-tan)]/20 flex-1 min-h-0 flex flex-col">
              <h3 className="text-sm font-semibold text-[var(--brand-tan)] mb-2">Bay Performance</h3>
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
            <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-xl p-3 border border-[var(--brand-tan)]/20">
              <h3 className="text-sm font-semibold text-[var(--brand-tan)] mb-2">Select Bay</h3>
              <div className="grid grid-cols-6 gap-2">
                {venueResult.players.map(p => (
                  <button
                    key={p.bay_id}
                    onClick={() => setSelectedBay(p.bay_id)}
                    className={`py-1.5 px-2 rounded-lg text-xs font-medium transition-all ${
                      selectedBay === p.bay_id
                        ? 'bg-[var(--brand-bright-purple)] text-[var(--brand-tan)] border-2 border-[var(--brand-lavender)]'
                        : 'bg-[var(--brand-bright-purple)]/20 text-[var(--brand-lavender)] border border-[var(--brand-lavender)]/30 hover:bg-[var(--brand-bright-purple)]/40'
                    }`}
                  >
                    #{p.bay_id}
                  </button>
                ))}
              </div>
            </div>

            {/* Selected Bay Details */}
            {selectedPlayer && (
              <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-xl p-3 border border-[var(--brand-tan)]/20">
                <h3 className="text-sm font-semibold text-[var(--brand-tan)] mb-2">
                  Bay #{selectedPlayer.bay_id} Details
                </h3>
                <div className="grid grid-cols-2 gap-2 text-xs">
                  <div>
                    <span className="text-[var(--brand-lavender)]">Handicap:</span>
                    <span className="text-[var(--brand-tan)] font-semibold ml-1">{selectedPlayer.handicap}</span>
                  </div>
                  <div>
                    <span className="text-[var(--brand-lavender)]">Pattern:</span>
                    <span className="text-[var(--brand-tan)] font-semibold ml-1">{selectedPlayer.pattern_type}</span>
                  </div>
                  <div>
                    <span className="text-[var(--brand-lavender)]">σ_x:</span>
                    <span className="text-[var(--brand-tan)] font-semibold ml-1">{selectedPlayer.sigma_x.toFixed(1)}ft</span>
                  </div>
                  <div>
                    <span className="text-[var(--brand-lavender)]">σ_y:</span>
                    <span className="text-[var(--brand-tan)] font-semibold ml-1">{selectedPlayer.sigma_y.toFixed(1)}ft</span>
                  </div>
                  <div>
                    <span className="text-[var(--brand-lavender)]">RTP:</span>
                    <span className="text-[var(--brand-tan)] font-semibold ml-1">{selectedPlayer.rtp.toFixed(2)}%</span>
                  </div>
                  <div>
                    <span className="text-[var(--brand-lavender)]">Net:</span>
                    <span className="text-[var(--brand-tan)] font-semibold ml-1">${selectedPlayer.net.toFixed(2)}</span>
                  </div>
                </div>
              </div>
            )}

            {/* Pattern Visualization */}
            {selectedPlayer && selectedPlayer.boundary_points && (
              <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-xl p-3 border border-[var(--brand-tan)]/20">
                <h3 className="text-sm font-semibold text-[var(--brand-tan)] mb-2">
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
                        const xs = points.map(p => p[0]);
                        const ys = points.map(p => p[1]);
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
                        points.forEach((point, i) => {
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
                <div className="text-xs text-[var(--brand-lavender)] text-center mt-2">
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
                <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-xl p-3 border border-[var(--brand-tan)]/20 flex-1 min-h-0 flex flex-col">
                  <h3 className="text-sm font-semibold text-[var(--brand-tan)] mb-2">
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
                <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-xl p-3 border border-[var(--brand-tan)]/20 flex-1 min-h-0 flex flex-col">
                  <h3 className="text-sm font-semibold text-[var(--brand-tan)] mb-2">
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
            <div className="text-[var(--brand-lavender)] mb-3">
              <svg className="w-16 h-16 mx-auto opacity-50" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
              </svg>
            </div>
            <p className="text-[var(--brand-lavender)] text-sm">Configure simulation parameters and click "Run Simulation"</p>
          </div>
        </div>
      )}
    </div>
  );
}
