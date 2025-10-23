import { useState, useCallback, useEffect } from 'react';
import init, { simulate_player_session } from '../wasm/continuum_golf_simulator';

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
  is_suspicious: boolean;
  confidence: number;
  detected_patterns: string[];
  recommended_action: string;
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

  // Calculate payout multiplier from distance for a specific hole
  const calculatePayoutMultiplier = (distanceYards: number, pmax: number, holeId: number): number => {
    if (!pmax || pmax <= 0 || isNaN(pmax)) {
      console.warn('⚠️ Invalid P_max for payout calculation:', pmax);
      return 0;
    }

    const hole = getHoleConfig(holeId);
    const dMaxFt = hole.d_max_ft;
    const k = hole.k;

    // Convert distance from yards to feet
    const distanceFt = distanceYards * 3;

    // If beyond target radius, no payout
    if (distanceFt >= dMaxFt) {
      return 0;
    }

    // Rust formula: P_max * (1 - d/d_max)^k
    const payoutFactor = Math.pow(1 - distanceFt / dMaxFt, k);
    const multiplier = pmax * payoutFactor;

    return Math.max(0, multiplier);
  };

  // WASM wrapper: simulates a single shot using Rust implementation
  const simulateShotWasm = useCallback(
    (handicap: number, wager: number, holeId: number): { shot: Shot; skillProfile: WasmSkillProfile | null; holeId: number } => {
      try {
        // Call WASM with num_shots=1
        const result: WasmSessionResult = simulate_player_session(
          handicap,
          1, // Single shot
          wager,
          wager,
          holeId // Pass selected hole ID
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

        // Extract skill profile for Kalman update (use first category, typically Wedge)
        const skillProfile = result.final_skills && result.final_skills.length > 0
          ? result.final_skills[0]
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

        return { shot, skillProfile, holeId: usedHoleId };
      } catch (error) {
        console.error('WASM simulation error:', error);
        throw error;
      }
    },
    []
  );

  // Simulate a single shot
  const simulateShot = useCallback(
    (wager: number, manualDistance?: number): { shot: Shot; skillProfile: WasmSkillProfile | null; holeId: number } => {
      // Use the selected hole ID
      const holeId = selectedHoleId;

      // Developer mode: use manual distance with placeholder simulation
      if (manualDistance !== undefined) {
        const distance = manualDistance;
        const angle = Math.random() * 2 * Math.PI;
        const multiplier = calculatePayoutMultiplier(distance, skillEstimate.pmax, holeId);
        const payout = wager * multiplier;
        const profit = payout - wager;

        return {
          shot: {
            distance,
            angle,
            wager,
            payout,
            profit,
            multiplier,
          },
          skillProfile: null, // No skill update in manual mode
          holeId,
        };
      }

      // Use WASM if ready, otherwise fall back to placeholder
      if (wasmReady) {
        try {
          return simulateShotWasm(initialHandicap, wager, holeId);
        } catch (error) {
          console.warn('⚠️ WASM simulation failed, using placeholder:', error);
          // Fall through to placeholder
        }
      }

      // Placeholder simulation (fallback)
      const u = Math.random();
      let distance = skillEstimate.sigma * Math.sqrt(-2 * Math.log(u));

      // Add fat-tail probability (2% chance of 3x worse)
      if (Math.random() < 0.02) {
        distance *= 3;
      }

      const angle = Math.random() * 2 * Math.PI;
      const multiplier = calculatePayoutMultiplier(distance, skillEstimate.pmax, holeId);
      const payout = wager * multiplier;
      const profit = payout - wager;

      return {
        shot: {
          distance,
          angle,
          wager,
          payout,
          profit,
          multiplier,
        },
        skillProfile: null,
        holeId,
      };
    },
    [wasmReady, initialHandicap, selectedHoleId, skillEstimate, simulateShotWasm]
  );

  // Update Kalman filter with new measurement
  const updateKalman = useCallback(
    (measurements: number[]) => {
      let state = { ...kalmanState };

      // For Rayleigh distribution, we estimate sigma from sample variance
      // Sample mean of r^2 should be 2*sigma^2
      const sumSquared = measurements.reduce((sum, r) => sum + r * r, 0);
      const meanSquared = sumSquared / measurements.length;
      const estimatedSigmaSquared = meanSquared / 2;
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

      setKalmanState(state);
      setSkillEstimate({
        sigma: newMean,
        confidence,
        pmax: clampedPmax,
      });

      return { sigma: newMean, confidence, pmax: clampedPmax };
    },
    [kalmanState]
  );

  // Shoot batch using WASM
  const shootBatch = useCallback(
    (wager: number, numShots: number) => {
      if (numShots <= 0) return [];

      const holeId = selectedHoleId;

      try {
        // Call WASM with batch of shots
        const result: WasmSessionResult = simulate_player_session(
          initialHandicap,
          numShots,
          wager,
          wager,
          holeId
        );

        if (!result.shots || result.shots.length === 0) {
          throw new Error('WASM returned no shots');
        }

        console.log(`🎯 Batch WASM Result: ${result.shots.length} shots simulated`);

        // Map all WASM shots to UI format and create P_max history entries
        const batchShots: Shot[] = result.shots.map((wasmShot) => ({
          distance: wasmShot.miss_distance_ft / 3, // Convert feet to yards
          angle: Math.random() * 2 * Math.PI, // Random angle
          wager: wasmShot.wager,
          payout: wasmShot.payout,
          profit: wasmShot.payout - wasmShot.wager,
          multiplier: wasmShot.multiplier,
        }));

        const newShots = [...shots, ...batchShots];
        setShots(newShots);
        setCurrentHoleId(holeId);

        // For batch shots, we should only add ONE data point at the END
        // showing the final smoothed P_max, not per-shot fluctuations
        // This prevents the spiky graph issue

        // Get final skill profile
        const skillProfile = result.final_skills && result.final_skills.length > 0
          ? result.final_skills[0]
          : null;

        if (skillProfile && skillProfile.p_max_current > 0 && !isNaN(skillProfile.p_max_current)) {
          const updated: SkillEstimate = {
            sigma: Math.max(1, skillProfile.sigma),
            confidence: Math.max(skillEstimate.confidence, Math.max(0, Math.min(100, skillProfile.confidence))),
            pmax: skillProfile.p_max_current,
          };

          console.log('✅ Batch WASM Update:', {
            sigma: updated.sigma.toFixed(2),
            pmax: updated.pmax.toFixed(2),
            confidence: updated.confidence.toFixed(1) + '%',
            totalShots: newShots.length,
            batchSize: numShots,
          });

          setSkillEstimate(updated);
          setKalmanState({
            mean: updated.sigma,
            variance: 100 * (1 - updated.confidence / 100),
            measurementCount: newShots.length,
          });

          // Update anti-cheat report if available
          if (result.anti_cheat_report) {
            setAntiCheatReport(result.anti_cheat_report);

            // Log suspicious activity
            if (result.anti_cheat_report.is_suspicious) {
              console.warn('🚨 Anti-Cheat Alert:', {
                confidence: (result.anti_cheat_report.confidence * 100).toFixed(0) + '%',
                patterns: result.anti_cheat_report.detected_patterns,
                action: result.anti_cheat_report.recommended_action,
              });
            }
          }

          // Add ONE smoothed P_max data point for the entire batch
          // This shows the final converged value after processing all shots
          setPmaxHistory((prev) => [...prev, {
            shotNumber: newShots.length,
            pmax: updated.pmax,
            confidence: updated.confidence,
            sigma: updated.sigma,
          }]);
        }

        return batchShots;
      } catch (error) {
        console.error('❌ Batch WASM simulation failed:', error);
        return [];
      }
    },
    [shots, wasmReady, initialHandicap, selectedHoleId, skillEstimate]
  );

  // Shoot once
  const shootOnce = useCallback(
    (wager: number, manualDistance?: number) => {
      const { shot, skillProfile, holeId } = simulateShot(wager, manualDistance);
      const newShots = [...shots, shot];
      setShots(newShots);
      setCurrentHoleId(holeId); // Track which hole was used

      // Determine if we should update skill estimate
      // Adaptive update frequency to match Rust's smoothing:
      // - Shots 1-30: Update every shot (aggressive anti-sandbagging)
      // - Shots 31-100: Update every 5 shots (balanced transition)
      // - Shots 100+: Update every 10 shots (conservative stability)
      const shouldUpdate =
        newShots.length <= 30 || // Early phase: every shot
        (newShots.length <= 100 && newShots.length % 5 === 0) || // Transition: every 5
        (newShots.length > 100 && newShots.length % 10 === 0) || // Mature: every 10
        (shots.length > 0 && wager >= shots.reduce((sum, s) => sum + s.wager, 0) / shots.length * 10); // High stakes

      let updated: SkillEstimate = { ...skillEstimate };

      // Only update skill estimate periodically
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
          // WASM returns calculated P_max, but we need UI-side rate limiting
          // because WASM creates a new player for each shot (no state persistence)

          // WAGER-BASED RATE LIMITING:
          // Philosophy: Big bets = high confidence in true skill = allow more P_max adaptation
          // Small bets = testing/uncertain = conservative P_max changes
          let pmaxToUse = skillProfile.p_max_current;
          const currentPmax = skillEstimate.pmax;

          if (currentPmax > 0 && newShots.length > 1) {
            // Calculate average wager from shot history
            const avgWager = newShots.reduce((sum, s) => sum + s.wager, 0) / newShots.length;
            const wagerRatio = avgWager > 0 ? wager / avgWager : 1.0;

            // HYBRID MODEL: Base rate + wager bonus
            const baseMaxChange = 0.15; // 15% base protection for all players
            const wagerBonus = Math.min(0.15, Math.max(0, (wagerRatio - 1.0) * 0.05)); // +5% per 1x wager increase, capped at +15%
            const totalMaxChange = baseMaxChange + wagerBonus; // Max 30% total

            const maxIncreaseRatio = 1.0 + totalMaxChange;
            const maxDecreaseRatio = 1.0 - totalMaxChange;

            const pmaxClamped = Math.min(
              currentPmax * maxIncreaseRatio,
              Math.max(currentPmax * maxDecreaseRatio, pmaxToUse)
            );

            const wasClamped = Math.abs(pmaxToUse - pmaxClamped) > 0.01;

            if (wasClamped || wagerRatio > 1.5) {
              console.log('⚠️ Wager-based P_max rate limiting:', {
                wager: `$${wager.toFixed(2)}`,
                avgWager: `$${avgWager.toFixed(2)}`,
                wagerRatio: wagerRatio.toFixed(2) + 'x',
                allowedChange: `±${(totalMaxChange * 100).toFixed(1)}%`,
                breakdown: `base ${(baseMaxChange * 100).toFixed(0)}% + bonus ${(wagerBonus * 100).toFixed(0)}%`,
                from: currentPmax.toFixed(2) + 'x',
                calculated: pmaxToUse.toFixed(2) + 'x',
                final: pmaxClamped.toFixed(2) + 'x',
                clamped: wasClamped ? 'YES' : 'NO',
              });
            }

            pmaxToUse = pmaxClamped;
          }

          updated = {
            sigma: Math.max(1, skillProfile.sigma),
            confidence: Math.max(skillEstimate.confidence, Math.max(0, Math.min(100, skillProfile.confidence))), // Never decrease
            pmax: pmaxToUse,
          };

          console.log('✅ WASM Update (UI rate-limited):', {
            sigma: updated.sigma.toFixed(2),
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

          updated = {
            sigma: kalmanResult.sigma,
            confidence: Math.max(skillEstimate.confidence, kalmanResult.confidence), // Never decrease
            pmax: kalmanResult.pmax,
          };

          console.log('✅ Fallback Kalman Update:', {
            sigma: updated.sigma.toFixed(2),
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

      return shot;
    },
    [shots, simulateShot, updateKalman, skillEstimate]
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
