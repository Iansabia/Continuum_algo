import { useState, useCallback } from 'react';

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

export function useSimulator(initialHandicap: number = 10) {
  const [shots, setShots] = useState<Shot[]>([]);
  const [skillEstimate, setSkillEstimate] = useState<SkillEstimate>({
    sigma: 10, // Initial guess based on handicap
    confidence: 0,
    pmax: 1.176, // Initial P_max for sigma=10
  });
  const [pmaxHistory, setPmaxHistory] = useState<PmaxDataPoint[]>([]);
  const [kalmanState, setKalmanState] = useState<KalmanState>({
    mean: 10,
    variance: 100,
    measurementCount: 0,
  });

  // Calculate breakeven radius from P_max
  const calculateBreakevenRadius = (pmax: number, sigma: number): number => {
    return sigma * Math.sqrt(-2 * Math.log(1 / pmax));
  };

  // Calculate payout multiplier from distance
  const calculatePayoutMultiplier = (distance: number, pmax: number): number => {
    const rMax = Math.sqrt(-2 * Math.log(1 / pmax)); // Normalized to sigma=1
    return Math.max(0, pmax * Math.exp(-Math.pow(distance, 2) / (2 * Math.pow(rMax, 2))));
  };

  // Simulate a single shot (placeholder - will be replaced by WASM)
  const simulateShot = useCallback(
    (wager: number, manualDistance?: number): Shot => {
      let distance: number;

      if (manualDistance !== undefined) {
        // Developer mode: use manual distance
        distance = manualDistance;
      } else {
        // Simulate using Rayleigh distribution
        // Rayleigh: r = sigma * sqrt(-2 * ln(U)) where U ~ Uniform(0,1)
        const u = Math.random();
        distance = skillEstimate.sigma * Math.sqrt(-2 * Math.log(u));

        // Add fat-tail probability (2% chance of 3x worse)
        if (Math.random() < 0.02) {
          distance *= 3;
        }
      }

      // Random angle
      const angle = Math.random() * 2 * Math.PI;

      // Calculate payout
      const multiplier = calculatePayoutMultiplier(distance, skillEstimate.pmax);
      const payout = wager * multiplier;
      const profit = payout - wager;

      return {
        distance,
        angle,
        wager,
        payout,
        profit,
        multiplier,
      };
    },
    [skillEstimate]
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

      // Calculate new P_max from estimated sigma
      // P_max = exp(1/2) for optimized pricing (from Phase 6)
      const pmax = Math.exp(0.5);

      setKalmanState(state);
      setSkillEstimate({
        sigma: newMean,
        confidence,
        pmax,
      });

      return { sigma: newMean, confidence, pmax };
    },
    [kalmanState]
  );

  // Shoot once
  const shootOnce = useCallback(
    (wager: number, manualDistance?: number) => {
      const shot = simulateShot(wager, manualDistance);
      const newShots = [...shots, shot];
      setShots(newShots);

      // Determine if we should update Kalman filter
      const shouldUpdate =
        newShots.length % 5 === 0 || // Every 5 shots
        wager >= shots.reduce((sum, s) => sum + s.wager, 0) / shots.length * 10; // High-stakes (10x avg)

      if (shouldUpdate || newShots.length === 1) {
        // Get recent measurements for Kalman update
        const recentShots = newShots.slice(-5);
        const measurements = recentShots.map((s) => s.distance);
        const updated = updateKalman(measurements);

        // Add to P_max history
        setPmaxHistory((prev) => [
          ...prev,
          {
            shotNumber: newShots.length,
            pmax: updated.pmax,
            confidence: updated.confidence,
            sigma: updated.sigma,
          },
        ]);
      }

      return shot;
    },
    [shots, simulateShot, updateKalman]
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
    setSkillEstimate({
      sigma: 10,
      confidence: 0,
      pmax: 1.176,
    });
    setPmaxHistory([]);
    setKalmanState({
      mean: 10,
      variance: 100,
      measurementCount: 0,
    });
  }, []);

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
