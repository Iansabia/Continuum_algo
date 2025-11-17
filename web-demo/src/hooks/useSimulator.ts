import { useState, useCallback, useEffect } from 'react';
import init, { simulate_player_session, analyze_anti_cheat } from '../wasm/continuum_golf_simulator';

// Hole configurations matching Rust HOLE_CONFIGURATIONS
const HOLE_CONFIGS = [
  { id: 1, distance_yds: 75, d_max_ft: 17.95, rtp: 0.85, k: 5.0 },
  { id: 2, distance_yds: 100, d_max_ft: 25.69, rtp: 0.85, k: 5.0 },
  { id: 3, distance_yds: 125, d_max_ft: 36.71, rtp: 0.85, k: 5.5 },
  { id: 4, distance_yds: 150, d_max_ft: 47.58, rtp: 0.85, k: 6.0 },
  { id: 5, distance_yds: 175, d_max_ft: 59.09, rtp: 0.85, k: 6.0 },
  { id: 6, distance_yds: 200, d_max_ft: 73.58, rtp: 0.85, k: 6.5 },
  { id: 7, distance_yds: 225, d_max_ft: 84.84, rtp: 0.85, k: 6.5 },
  { id: 8, distance_yds: 250, d_max_ft: 101.14, rtp: 0.85, k: 6.5 },
];

// Get club category for a hole (matches Rust ClubCategory)
function getCategoryForHole(holeId: number): string {
  const hole = HOLE_CONFIGS.find(h => h.id === holeId);
  if (!hole) return "Wedge";

  if (hole.distance_yds < 140) return "Wedge";
  if (hole.distance_yds < 210) return "MidIron";
  return "LongIron";
}

// WASM result types
interface WasmShotOutcome {
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

interface WasmSkillProfile {
  category: string;
  sigma: number;
  confidence: number;
  p_max_current: number;
}

interface WasmAnomalyReport {
  is_suspicious: boolean;
  confidence: number;
  detected_patterns: string[];
  recommended_action: string;
}

interface WasmSessionResult {
  total_wagered: number;
  total_won: number;
  net_gain_loss: number;
  session_house_edge: number;
  shots: WasmShotOutcome[];
  final_skills: WasmSkillProfile[];
  anti_cheat_report: WasmAnomalyReport | null;
}

export interface Shot {
  distance: number;
  angle: number;
  wager: number;
  payout: number;
  profit: number;
  multiplier: number;
}

export interface SkillEstimate {
  sigma: number;
  sigmaX: number;
  sigmaY: number;
  confidence: number;
  pmax: number;
}

export interface PmaxDataPoint {
  shotNumber: number;
  pmax: number;
  confidence: number;
  sigma: number;
}

export interface SessionStats {
  shotsTaken: number;
  totalWagered: number;
  totalWon: number;
  netPL: number;
  actualHouseEdge: number;
  theoreticalHouseEdge: number;
}

interface KalmanState {
  mean: number;
  variance: number;
  measurementCount: number;
}

// Calculate sigmaX and sigmaY from shot coordinates
const calculateDirectionalSigmas = (shots: Shot[], currentSigma: number = 8): { sigmaX: number; sigmaY: number } => {
  // Need at least 2 shots to calculate meaningful directional standard deviations
  // For 0-1 shots, use the overall sigma estimate
  if (shots.length < 2) {
    return { sigmaX: currentSigma, sigmaY: currentSigma };
  }

  // Convert polar to Cartesian coordinates
  const xCoords: number[] = [];
  const yCoords: number[] = [];

  shots.forEach(shot => {
    const x = shot.distance * Math.cos(shot.angle);
    const y = shot.distance * Math.sin(shot.angle);
    xCoords.push(x);
    yCoords.push(y);
  });

  // Calculate mean
  const meanX = xCoords.reduce((sum, x) => sum + x, 0) / xCoords.length;
  const meanY = yCoords.reduce((sum, y) => sum + y, 0) / yCoords.length;

  // Calculate sample variance (using n-1 for Bessel's correction)
  const varianceX = xCoords.reduce((sum, x) => sum + Math.pow(x - meanX, 2), 0) / (xCoords.length - 1);
  const varianceY = yCoords.reduce((sum, y) => sum + Math.pow(y - meanY, 2), 0) / (yCoords.length - 1);

  const sigmaX = Math.sqrt(varianceX);
  const sigmaY = Math.sqrt(varianceY);

  return {
    sigmaX: Math.max(0.1, sigmaX), // Minimum to avoid division by zero
    sigmaY: Math.max(0.1, sigmaY)
  };
};

// Calculate initial P_max from handicap
// Uses a stable formula based on expected dispersion
const calculateInitialPmax = (handicap: number): number => {
  // Handicap to sigma mapping (rough approximation)
  // Handicap 0 ≈ 3y, Handicap 10 ≈ 7y, Handicap 20 ≈ 12y, Handicap 30 ≈ 18y
  const sigma = 3 + (handicap * 0.5); // Linear mapping
  const sigmaFt = sigma * 3; // Convert to feet

  // Using default d_max of 30 feet and RTP of 0.85
  const dMaxFt = 30;
  const rtp = 0.85;

  // Expected payout approximation for Rayleigh distribution
  // Better players (low sigma) → higher E[payout] → lower P_max
  // Worse players (high sigma) → lower E[payout] → higher P_max
  const normalizedSigma = sigmaFt / dMaxFt; // 0 to ~1
  const expectedPayout = Math.max(0.02, 0.4 * (1 - Math.min(0.9, normalizedSigma)));
  const pmax = rtp / expectedPayout;

  // Clamp to reasonable range: 5 to 50x
  return Math.max(5, Math.min(50, pmax));
};

export interface AnomalyReport {
  isSuspicious: boolean;
  confidence: number;
  detectedPatterns: string[];
  recommendedAction: string;
}

export function useSimulator(initialHandicap: number = 10, selectedHoleId: number = 1) {
  const [wasmReady, setWasmReady] = useState(false);
  const [shots, setShots] = useState<Shot[]>([]);
  const [currentHoleId, setCurrentHoleId] = useState<number>(selectedHoleId); // Track which hole was used for last shot
  const [antiCheatReport, setAntiCheatReport] = useState<AnomalyReport | null>(null);

  // Calculate initial sigma and P_max from handicap
  const initialSigma = 3 + (initialHandicap * 0.5);
  const initialPmax = calculateInitialPmax(initialHandicap);

  const [skillEstimate, setSkillEstimate] = useState<SkillEstimate>({
    sigma: initialSigma,
    sigmaX: initialSigma,
    sigmaY: initialSigma,
    confidence: 0,
    pmax: initialPmax,
  });
  const [pmaxHistory, setPmaxHistory] = useState<PmaxDataPoint[]>([]);
  const [kalmanState, setKalmanState] = useState<KalmanState>({
    mean: initialSigma,
    variance: 100,
    measurementCount: 0,
  });

  // Initialize WASM module
  useEffect(() => {
    init()
      .then(() => {
        setWasmReady(true);
        console.log('✅ WASM module initialized successfully');
      })
      .catch((error) => {
        console.error('❌ Failed to initialize WASM module:', error);
        console.warn('⚠️ Falling back to placeholder simulation');
      });
  }, []);

  // Clear shots when hole changes
  useEffect(() => {
    if (currentHoleId !== selectedHoleId) {
      console.log(`🔄 Hole changed from ${currentHoleId} to ${selectedHoleId}, clearing shots`);
      setShots([]);
      setPmaxHistory([]);
      setCurrentHoleId(selectedHoleId);
    }
  }, [selectedHoleId, currentHoleId]);

  // Get hole configuration by ID
  const getHoleConfig = (holeId: number) => {
    const config = HOLE_CONFIGS.find(h => h.id === holeId);
    if (!config) {
      console.warn(`⚠️ Invalid hole ID ${holeId}, falling back to hole 1`);
      return HOLE_CONFIGS[0];
    }
    return config;
  };

  // Calculate breakeven radius from P_max for a specific hole
  const calculateBreakevenRadius = (pmax: number, holeId: number): number => {
    if (!pmax || pmax <= 0 || isNaN(pmax)) {
      console.warn('⚠️ Invalid P_max for breakeven calculation:', pmax);
      return 0;
    }

    const hole = getHoleConfig(holeId);
    const dMaxFt = hole.d_max_ft;
    const k = hole.k;

    // Formula: d_break = d_max * (1 - P_max^(-1/k))
    const breakevenFt = dMaxFt * (1 - Math.pow(pmax, -1 / k));

    // Convert back to yards for display
    return breakevenFt / 3;
  };

  // WASM wrapper: simulates a single shot using Rust implementation
  const simulateShotWasm = useCallback(
    (handicap: number, wager: number, holeId: number, manualDistance?: number): { shot: Shot; skillProfile: WasmSkillProfile | null; holeId: number; antiCheatReport: AnomalyReport | null } => {
      try {
        // Call WASM with num_shots=1, pass manual distance if provided
        const result: WasmSessionResult = simulate_player_session(
          handicap,
          1, // Single shot
          wager,
          wager,
          holeId, // Pass selected hole ID
          manualDistance // Pass manual miss distance for dev mode
        );

        if (!result.shots || result.shots.length === 0) {
          throw new Error('WASM returned no shots');
        }

        const wasmShot = result.shots[0];

        // Debug: Log WASM return values
        console.log('🔍 WASM Shot Result:', {
          hole_id: wasmShot.hole_id,
          miss_distance_ft: wasmShot.miss_distance_ft,
          multiplier: wasmShot.multiplier,
          payout: wasmShot.payout,
          p_max: wasmShot.p_max,
          is_fat_tail: wasmShot.is_fat_tail,
        });

        // Store the hole ID that was used
        const usedHoleId = wasmShot.hole_id;

        // Map WASM result to UI Shot interface
        const shot: Shot = {
          distance: wasmShot.miss_distance_ft / 3, // Convert feet to yards
          angle: Math.random() * 2 * Math.PI, // Random angle (WASM doesn't provide)
          wager: wasmShot.wager,
          payout: wasmShot.payout,
          profit: wasmShot.payout - wasmShot.wager,
          multiplier: wasmShot.multiplier,
        };

        // Extract skill profile matching the hole's category
        const holeCategory = getCategoryForHole(usedHoleId);
        const skillProfile = result.final_skills && result.final_skills.length > 0
          ? result.final_skills.find(s => s.category === holeCategory) || result.final_skills[0]
          : null;

        // Debug: Log skill profile
        if (skillProfile) {
          console.log('🔍 WASM Skill Profile:', {
            category: skillProfile.category,
            sigma: skillProfile.sigma,
            confidence: skillProfile.confidence,
            p_max_current: skillProfile.p_max_current,
          });
        } else {
          console.warn('⚠️ WASM returned no skill profile');
        }

        // Extract anti-cheat report from result
        const antiCheatReport: AnomalyReport | null = result.anti_cheat_report ? {
          isSuspicious: result.anti_cheat_report.is_suspicious,
          confidence: result.anti_cheat_report.confidence,
          detectedPatterns: result.anti_cheat_report.detected_patterns,
          recommendedAction: result.anti_cheat_report.recommended_action,
        } : null;

        return { shot, skillProfile, holeId: usedHoleId, antiCheatReport };
      } catch (error) {
        console.error('WASM simulation error:', error);
        throw error;
      }
    },
    []
  );

  // Simulate a single shot - always uses Rust WASM
  const simulateShot = useCallback(
    (wager: number, manualDistance?: number): { shot: Shot; skillProfile: WasmSkillProfile | null; holeId: number; antiCheatReport: AnomalyReport | null } => {
      if (!wasmReady) {
        throw new Error('WASM not ready - cannot simulate shot');
      }

      // Use the selected hole ID
      const holeId = selectedHoleId;

      // Log developer mode if manual distance is provided
      if (manualDistance !== undefined) {
        console.log('🔧 Developer mode: manual distance =', manualDistance, 'yards');
      }

      // Always use WASM (with optional manual distance for dev mode)
      return simulateShotWasm(initialHandicap, wager, holeId, manualDistance);
    },
    [wasmReady, initialHandicap, selectedHoleId, simulateShotWasm]
  );

  // Update Kalman filter with new measurement
  const updateKalman = useCallback(
    (measurements: number[]) => {
      let state = { ...kalmanState };

      // For Rayleigh distribution, we estimate sigma from sample variance
      // Sample mean of r^2 should be 2*sigma^2
      const sumSquared = measurements.reduce((sum, r) => sum + r * r, 0);
      const meanSquared = sumSquared / measurements.length;
      const estimatedSigmaSquared = Math.max(0.01, meanSquared / 2); // Minimum 0.01 to avoid division by zero
      const estimatedSigma = Math.sqrt(estimatedSigmaSquared);

      // Kalman filter update
      const measurementVariance = estimatedSigmaSquared / measurements.length;
      const kalmanGain = state.variance / (state.variance + measurementVariance);

      const newMean = state.mean + kalmanGain * (estimatedSigma - state.mean);
      const newVariance = (1 - kalmanGain) * state.variance;
      const newCount = state.measurementCount + measurements.length;

      state = {
        mean: newMean,
        variance: newVariance,
        measurementCount: newCount,
      };

      // Calculate confidence (0-100%) based on variance and sample size
      const confidence = Math.min(100, (newCount / (newCount + 10)) * 100 * (1 - Math.min(1, newVariance / 100)));

      // Estimate P_max using simplified formula
      // P_max ≈ RTP / E[payout]
      // For a Rayleigh distribution with sigma and target d_max:
      // E[payout] ≈ 0.5 * (1 - sigma_ft / d_max_ft) for rough approximation
      // Using d_max = 30 feet and RTP = 0.85 (15% house edge)
      const sigmaFt = newMean * 3; // Convert sigma from yards to feet
      const dMaxFt = 30; // Default target radius
      const rtp = 0.85; // 15% house edge

      // Simplified expected payout calculation
      // Better players (lower sigma) → higher E[payout] → lower P_max
      // Worse players (higher sigma) → lower E[payout] → higher P_max
      const expectedPayout = Math.max(0.01, 0.5 * (1 - Math.min(0.95, sigmaFt / dMaxFt)));
      const pmax = rtp / expectedPayout;

      // Sanity check: P_max should be in reasonable range (5 to 100)
      const clampedPmax = Math.max(5, Math.min(100, pmax));

      console.log('📊 Kalman Update (Fallback):', {
        sigma: newMean.toFixed(2),
        confidence: confidence.toFixed(1) + '%',
        estimatedPmax: pmax.toFixed(2),
        clampedPmax: clampedPmax.toFixed(2),
      });

      // Calculate directional sigmas from actual shot data
      const { sigmaX, sigmaY } = calculateDirectionalSigmas(shots, newMean);

      setKalmanState(state);
      setSkillEstimate({
        sigma: newMean,
        sigmaX,
        sigmaY,
        confidence,
        pmax: clampedPmax,
      });

      return { sigma: newMean, confidence, pmax: clampedPmax };
    },
    [kalmanState, shots]
  );

  // Shoot batch using WASM - behaves identically to clicking 1x button multiple times
  const shootBatch = useCallback(
    (wager: number, numShots: number) => {
      if (numShots <= 0) return [];

      console.log(`🎯 Batch mode: simulating ${numShots} shots (incremental)...`);

      const batchShots: Shot[] = [];
      let currentShotState = [...shots]; // Array spread, not object spread!
      let currentSkillEstimate = { ...skillEstimate }; // Mutable copy for tracking across iterations

      // Shoot each shot individually, just like clicking 1x button multiple times
      for (let i = 0; i < numShots; i++) {
        try {
          // Simulate one shot using WASM
          const { shot, skillProfile, holeId } = simulateShot(wager);
          batchShots.push(shot);
          currentShotState = [...currentShotState, shot];
          setShots(currentShotState);
          setCurrentHoleId(holeId);

          // MCMC updates every shot - Rust handles smoothing
          const shotNumber = currentShotState.length;
          const shouldUpdate = true; // Always update with MCMC

          let updated: SkillEstimate = { ...currentSkillEstimate };

          // Update skill estimate (MCMC updates every shot)
          if (shouldUpdate) {
            console.log(`🔄 Batch update triggered at shot ${shotNumber}/${numShots}`, {
              hasSkillProfile: !!skillProfile,
              wasmReady,
            });
            
            // Use WASM skill profile if available AND VALID
            if (
              skillProfile &&
              skillProfile.p_max_current > 0 &&
              !isNaN(skillProfile.p_max_current) &&
              skillProfile.sigma > 0 &&
              !isNaN(skillProfile.sigma)
            ) {
              // Trust WASM's MCMC P_max calculation directly (no UI rate limiting)
              let pmaxToUse = skillProfile.p_max_current;

              const sigmaCurrent = Math.max(1, skillProfile.sigma);
              const { sigmaX, sigmaY } = calculateDirectionalSigmas(currentShotState, sigmaCurrent);

              updated = {
                sigma: sigmaCurrent,
                sigmaX,
                sigmaY,
                confidence: Math.max(currentSkillEstimate.confidence, Math.max(0, Math.min(100, skillProfile.confidence))),
                pmax: pmaxToUse,
              };

              console.log(`✅ Batch update (shot ${shotNumber}/${numShots}):`, {
                sigma: updated.sigma.toFixed(2),
                pmax: updated.pmax.toFixed(2),
                confidence: updated.confidence.toFixed(1) + '%',
              });

              setSkillEstimate(updated);
              setKalmanState({
                mean: updated.sigma,
                variance: 100 * (1 - updated.confidence / 100),
                measurementCount: shotNumber,
              });
            }
          }

          // ALWAYS add current state to P_max history for charting (matching shootOnce)
          setPmaxHistory((prev) => [
            ...prev,
            {
              shotNumber,
              pmax: updated.pmax,
              confidence: updated.confidence,
              sigma: updated.sigma,
            },
          ]);

          // Update currentSkillEstimate for next iteration
          currentSkillEstimate = updated;
        } catch (error) {
          console.error(`❌ Batch shot ${i + 1}/${numShots} failed:`, error);
          break; // Stop batch on error
        }
      }

      // REMOVED: Anti-cheat analysis from batch shooter
      // The anti-cheat call was re-simulating the entire session which corrupted P_max
      // Anti-cheat analysis now only runs in the standalone shootOnce function

      console.log(`✅ Batch complete: ${batchShots.length} shots`);
      return batchShots;
    },
    [shots, wasmReady, initialHandicap, selectedHoleId, skillEstimate, simulateShot]
  );

  // Shoot once
  const shootOnce = useCallback(
    (wager: number, manualDistance?: number) => {
      console.log('🎯 shootOnce called:', { wager, manualDistance, isDefined: manualDistance !== undefined });
      const { shot, skillProfile, holeId } = simulateShot(wager, manualDistance);
      const newShots = [...shots, shot];
      setShots(newShots);
      setCurrentHoleId(holeId); // Track which hole was used

      // MCMC updates every shot now - no more batching needed
      // The Rust MCMC system handles smoothing internally, so we update UI every shot
      const shouldUpdate = true; // Always update with MCMC

      let updated: SkillEstimate = { ...skillEstimate };

      // Update skill estimate (MCMC updates every shot)
      if (shouldUpdate) {
        console.log('🔄 Update triggered at shot', newShots.length, {
          hasSkillProfile: !!skillProfile,
          manualMode: manualDistance !== undefined,
          wasmReady,
        });

        // Use WASM skill profile if available AND VALID
        if (
          skillProfile &&
          skillProfile.p_max_current > 0 &&
          !isNaN(skillProfile.p_max_current) &&
          skillProfile.sigma > 0 &&
          !isNaN(skillProfile.sigma)
        ) {
          // Trust WASM's MCMC P_max calculation directly
          // The Rust backend maintains persistent player state and has built-in
          // MCMC smoothing, so no additional UI rate limiting is needed
          let pmaxToUse = skillProfile.p_max_current;

          const sigmaCurrent = Math.max(1, skillProfile.sigma);

          // Calculate directional sigmas from actual shot data
          const { sigmaX: sigmaX1, sigmaY: sigmaY1 } = calculateDirectionalSigmas(newShots, sigmaCurrent);

          updated = {
            sigma: sigmaCurrent,
            sigmaX: sigmaX1,
            sigmaY: sigmaY1,
            confidence: Math.max(skillEstimate.confidence, Math.max(0, Math.min(100, skillProfile.confidence))), // Never decrease
            pmax: pmaxToUse,
          };

          console.log('✅ WASM Update (UI rate-limited):', {
            sigma: updated.sigma.toFixed(2),
            sigmaX: updated.sigmaX.toFixed(2),
            sigmaY: updated.sigmaY.toFixed(2),
            pmax: updated.pmax.toFixed(2),
            confidence: updated.confidence.toFixed(1) + '%',
            shots: newShots.length,
          });

          setSkillEstimate(updated);

          // Update Kalman state
          setKalmanState({
            mean: updated.sigma,
            variance: 100 * (1 - updated.confidence / 100),
            measurementCount: newShots.length,
          });
        } else {
          // WASM failed or returned bad data - use fallback Kalman
          if (skillProfile) {
            console.warn('⚠️ WASM skill profile invalid, using fallback Kalman:', {
              sigma: skillProfile.sigma,
              p_max_current: skillProfile.p_max_current,
              confidence: skillProfile.confidence,
            });
          } else {
            console.log('📊 No WASM profile (manual mode or WASM not ready), using fallback Kalman');
          }

          // Use placeholder Kalman for all non-WASM cases
          const recentShots = newShots.slice(-10);
          const measurements = recentShots.map((s) => s.distance);
          const kalmanResult = updateKalman(measurements);

          // Calculate directional sigmas from actual shot data
          const { sigmaX: sigmaX2, sigmaY: sigmaY2 } = calculateDirectionalSigmas(newShots, kalmanResult.sigma);

          updated = {
            sigma: kalmanResult.sigma,
            sigmaX: sigmaX2,
            sigmaY: sigmaY2,
            confidence: Math.max(skillEstimate.confidence, kalmanResult.confidence), // Never decrease
            pmax: kalmanResult.pmax,
          };

          console.log('✅ Fallback Kalman Update:', {
            sigma: updated.sigma.toFixed(2),
            sigmaX: updated.sigmaX.toFixed(2),
            sigmaY: updated.sigmaY.toFixed(2),
            pmax: updated.pmax.toFixed(2),
            confidence: updated.confidence.toFixed(1) + '%',
            shots: newShots.length,
          });

          setSkillEstimate(updated);
        }
      }

      // Always add current state to P_max history for charting
      setPmaxHistory((prev) => [
        ...prev,
        {
          shotNumber: newShots.length,
          pmax: updated.pmax,
          confidence: updated.confidence,
          sigma: updated.sigma,
        },
      ]);

      // Run anti-cheat analysis on accumulated UI shots every 5 shots
      // Uses stateless analyze_anti_cheat function that doesn't corrupt MCMC
      if (newShots.length >= 10 && newShots.length % 5 === 0) {
        try {
          // Convert UI shots to WASM ShotOutcome format
          const wasmShots = newShots.map((s, idx) => ({
            shot_number: idx + 1,
            hole_id: currentHoleId,
            wager: s.wager,
            miss_distance_ft: s.distance * 3, // Convert yards to feet
            multiplier: s.multiplier,
            payout: s.payout,
            is_fat_tail: false,
            p_max: skillEstimate.pmax,
          }));

          const antiCheatResult = analyze_anti_cheat(wasmShots);

          setAntiCheatReport({
            isSuspicious: antiCheatResult.is_suspicious,
            confidence: antiCheatResult.confidence,
            detectedPatterns: antiCheatResult.detected_patterns,
            recommendedAction: antiCheatResult.recommended_action,
          });

          if (antiCheatResult.is_suspicious) {
            console.warn('🚨 Anti-Cheat Alert:', {
              confidence: (antiCheatResult.confidence * 100).toFixed(0) + '%',
              patterns: antiCheatResult.detected_patterns,
            });
          }
        } catch (error) {
          console.warn('⚠️ Anti-cheat analysis failed:', error);
        }
      }

      return shot;
    },
    [shots, simulateShot, updateKalman, skillEstimate, pmaxHistory, initialHandicap, selectedHoleId]
  );

  // Calculate session stats
  const getSessionStats = useCallback((): SessionStats => {
    const shotsTaken = shots.length;
    const totalWagered = shots.reduce((sum, s) => sum + s.wager, 0);
    const totalWon = shots.reduce((sum, s) => sum + s.payout, 0);
    const netPL = totalWon - totalWagered;

    const actualHouseEdge = totalWagered > 0 ? ((totalWagered - totalWon) / totalWagered) * 100 : 0;
    const theoreticalHouseEdge = 15; // ~15% for optimized pricing

    return {
      shotsTaken,
      totalWagered,
      totalWon,
      netPL,
      actualHouseEdge,
      theoreticalHouseEdge,
    };
  }, [shots]);

  // Reset session
  const reset = useCallback(() => {
    setShots([]);

    // Recalculate initial values from handicap
    const resetSigma = 3 + (initialHandicap * 0.5);
    const resetPmax = calculateInitialPmax(initialHandicap);

    setSkillEstimate({
      sigma: resetSigma,
      sigmaX: resetSigma,
      sigmaY: resetSigma,
      confidence: 0,
      pmax: resetPmax,
    });
    setPmaxHistory([]);
    setAntiCheatReport(null);
    setKalmanState({
      mean: resetSigma,
      variance: 100,
      measurementCount: 0,
    });

    console.log('🔄 Session Reset:', {
      handicap: initialHandicap,
      initialSigma: resetSigma.toFixed(2),
      initialPmax: resetPmax.toFixed(2),
    });
  }, [initialHandicap]);

  // Get current hole configuration
  const currentHole = getHoleConfig(currentHoleId);

  return {
    shots,
    skillEstimate,
    pmaxHistory,
    sessionStats: getSessionStats(),
    breakevenRadius: calculateBreakevenRadius(skillEstimate.pmax, currentHoleId),
    currentHole: {
      id: currentHole.id,
      distance: currentHole.distance_yds,
      targetRadius: currentHole.d_max_ft / 3, // Convert feet to yards for display
      k: currentHole.k, // Curve steepness
    },
    antiCheatReport,
    shootOnce,
    shootBatch,
    reset,
  };
}
