import { useState, useCallback, useEffect } from 'react';
import init, { simulate_player_session } from '../wasm/continuum_golf_simulator';

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

interface WasmSessionResult {
  total_wagered: number;
  total_won: number;
  net_gain_loss: number;
  session_house_edge: number;
  shots: WasmShotOutcome[];
  final_skills: WasmSkillProfile[];
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

export function useSimulator(initialHandicap: number = 10) {
  const [wasmReady, setWasmReady] = useState(false);
  const [shots, setShots] = useState<Shot[]>([]);

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

  // Calculate breakeven radius from P_max
  // Note: d_max is the maximum target radius (where payout = 0)
  // For now, we use a default d_max of 30 feet (10 yards) as an approximation
  const calculateBreakevenRadius = (pmax: number, sigma: number, dMaxFt: number = 30): number => {
    if (!pmax || pmax <= 0 || isNaN(pmax)) {
      console.warn('⚠️ Invalid P_max for breakeven calculation:', pmax);
      return sigma; // Fallback: return sigma as approximation
    }

    // Simplified formula: r_breakeven ≈ d_max * sqrt(1 - (1/P_max)^(1/k))
    // Using k = 5 (curve steepness from Rust)
    const k = 5.0;
    const breakevenFt = dMaxFt * Math.sqrt(Math.max(0, 1 - Math.pow(1 / pmax, 1 / k)));

    // Convert back to yards for display
    return breakevenFt / 3;
  };

  // Calculate payout multiplier from distance
  // Uses the Rust formula: payout_factor = (1 - d/d_max)^k
  const calculatePayoutMultiplier = (distanceYards: number, pmax: number, dMaxFt: number = 30): number => {
    if (!pmax || pmax <= 0 || isNaN(pmax)) {
      console.warn('⚠️ Invalid P_max for payout calculation:', pmax);
      return 0;
    }

    // Convert distance from yards to feet
    const distanceFt = distanceYards * 3;

    // If beyond target radius, no payout
    if (distanceFt >= dMaxFt) {
      return 0;
    }

    // Rust formula: P_max * (1 - d/d_max)^k
    const k = 5.0;
    const payoutFactor = Math.pow(1 - distanceFt / dMaxFt, k);
    const multiplier = pmax * payoutFactor;

    return Math.max(0, multiplier);
  };

  // WASM wrapper: simulates a single shot using Rust implementation
  const simulateShotWasm = useCallback(
    (handicap: number, wager: number): { shot: Shot; skillProfile: WasmSkillProfile | null } => {
      try {
        // Call WASM with num_shots=1
        const result: WasmSessionResult = simulate_player_session(
          handicap,
          1, // Single shot
          wager,
          wager,
          null // No specific hole (random selection)
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

        return { shot, skillProfile };
      } catch (error) {
        console.error('WASM simulation error:', error);
        throw error;
      }
    },
    []
  );

  // Simulate a single shot
  const simulateShot = useCallback(
    (wager: number, manualDistance?: number): { shot: Shot; skillProfile: WasmSkillProfile | null } => {
      // Developer mode: use manual distance with placeholder simulation
      if (manualDistance !== undefined) {
        const distance = manualDistance;
        const angle = Math.random() * 2 * Math.PI;
        const multiplier = calculatePayoutMultiplier(distance, skillEstimate.pmax);
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
        };
      }

      // Use WASM if ready, otherwise fall back to placeholder
      if (wasmReady) {
        try {
          return simulateShotWasm(initialHandicap, wager);
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
      const multiplier = calculatePayoutMultiplier(distance, skillEstimate.pmax);
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
      };
    },
    [wasmReady, initialHandicap, skillEstimate, simulateShotWasm]
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

  // Shoot once
  const shootOnce = useCallback(
    (wager: number, manualDistance?: number) => {
      const { shot, skillProfile } = simulateShot(wager, manualDistance);
      const newShots = [...shots, shot];
      setShots(newShots);

      // Determine if we should update skill estimate
      // Update every 10 shots OR on first shot OR on high-stakes wager
      const shouldUpdate =
        newShots.length === 1 ||
        newShots.length % 10 === 0 ||
        (shots.length > 0 && wager >= shots.reduce((sum, s) => sum + s.wager, 0) / shots.length * 10);

      let updated: SkillEstimate = { ...skillEstimate };

      // Only update skill estimate periodically
      if (shouldUpdate) {
        // Use WASM skill profile if available AND VALID
        if (
          skillProfile &&
          skillProfile.p_max_current > 0 &&
          !isNaN(skillProfile.p_max_current) &&
          skillProfile.sigma > 0 &&
          !isNaN(skillProfile.sigma)
        ) {
          // WASM returned valid data - smooth it with existing estimates
          const smoothingFactor = 0.3; // 30% new data, 70% old data (slow adaptation)

          // Smooth sigma
          const newSigma = skillEstimate.sigma * (1 - smoothingFactor) + skillProfile.sigma * smoothingFactor;

          // Confidence should only increase (never decrease)
          const newConfidence = Math.max(
            skillEstimate.confidence,
            Math.max(0, Math.min(100, skillProfile.confidence))
          );

          // Smooth P_max (critical for financial stability)
          const newPmax = skillEstimate.pmax * (1 - smoothingFactor) + skillProfile.p_max_current * smoothingFactor;

          updated = {
            sigma: Math.max(1, newSigma),
            confidence: newConfidence,
            pmax: Math.max(5, Math.min(50, newPmax)), // Clamp to safe range
          };

          console.log('✅ WASM Update (Smoothed):', {
            oldPmax: skillEstimate.pmax.toFixed(2),
            wasmPmax: skillProfile.p_max_current.toFixed(2),
            newPmax: updated.pmax.toFixed(2),
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
            console.warn('⚠️ WASM skill profile invalid, skipping update:', {
              sigma: skillProfile.sigma,
              p_max_current: skillProfile.p_max_current,
              confidence: skillProfile.confidence,
            });
          } else {
            // No WASM profile - use placeholder Kalman
            const recentShots = newShots.slice(-10);
            const measurements = recentShots.map((s) => s.distance);
            const kalmanResult = updateKalman(measurements);

            // Smooth the fallback result too
            const smoothingFactor = 0.2; // Even slower for fallback

            updated = {
              sigma: skillEstimate.sigma * (1 - smoothingFactor) + kalmanResult.sigma * smoothingFactor,
              confidence: Math.max(skillEstimate.confidence, kalmanResult.confidence), // Never decrease
              pmax: skillEstimate.pmax * (1 - smoothingFactor) + kalmanResult.pmax * smoothingFactor,
            };

            setSkillEstimate(updated);
          }
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

  return {
    shots,
    skillEstimate,
    pmaxHistory,
    sessionStats: getSessionStats(),
    breakevenRadius: calculateBreakevenRadius(skillEstimate.pmax, skillEstimate.sigma),
    shootOnce,
    reset,
  };
}
