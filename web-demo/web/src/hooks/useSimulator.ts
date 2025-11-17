import { useState, useCallback } from 'react';

export interface Shot {
  distance: number;
  angle: number;
  wager: number;
  payout: number;
  profit: number;
  multiplier: number;
  x?: number;
  y?: number;
}

export interface SkillEstimate {
  sigma: number;
  pmax: number;
  confidence: number;
}

export interface SessionStats {
  totalWagered: number;
  totalWon: number;
  netProfit: number;
  rtp: number;
  shotCount: number;
}

export interface PmaxDataPoint {
  shotNumber: number;
  pmax: number;
  confidence: number;
  sigma: number;
}

export interface CurrentHole {
  id: number;
  distance: number;
  targetRadius: number;
  k: number;
}

const HOLES: Record<number, CurrentHole> = {
  1: { id: 1, distance: 75, targetRadius: 15, k: 2.5 },
  2: { id: 2, distance: 100, targetRadius: 20, k: 2.5 },
  3: { id: 3, distance: 125, targetRadius: 25, k: 2.5 },
  4: { id: 4, distance: 150, targetRadius: 30, k: 2.5 },
  5: { id: 5, distance: 175, targetRadius: 35, k: 2.5 },
  6: { id: 6, distance: 200, targetRadius: 40, k: 2.5 },
  7: { id: 7, distance: 225, targetRadius: 45, k: 2.5 },
  8: { id: 8, distance: 250, targetRadius: 50, k: 2.5 },
};

const handicapToSigma = (handicap: number): number => {
  return 3.0 + (handicap / 30) * 12.0;
};

const calculatePmax = (sigma: number, targetRadius: number): number => {
  const ratio = sigma / targetRadius;
  return Math.max(1.5, Math.min(10.0, 1.5 + (1 - ratio) * 8.5));
};

const simulateShot = (sigma: number, devDistance?: number): { distance: number; angle: number } => {
  if (devDistance !== undefined) {
    const angle = Math.random() * 2 * Math.PI;
    return { distance: devDistance, angle };
  }
  const u1 = Math.random();
  const u2 = Math.random();
  const z0 = Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
  const distance = Math.abs(z0 * sigma);
  const angle = Math.random() * 2 * Math.PI;
  return { distance, angle };
};

const calculateMultiplier = (distance: number, pmax: number, targetRadius: number, k: number): number => {
  if (distance > targetRadius) return 0;
  const normalizedDist = distance / targetRadius;
  return pmax * Math.pow(1 - normalizedDist, k);
};

export const useSimulator = (handicap: number, holeId: number) => {
  const [shots, setShots] = useState<Shot[]>([]);
  const [pmaxHistory, setPmaxHistory] = useState<PmaxDataPoint[]>([]);

  const currentHole = HOLES[holeId];
  const sigma = handicapToSigma(handicap);
  const pmax = calculatePmax(sigma, currentHole.targetRadius);
  const confidence = Math.min(100, shots.length * 2);

  const skillEstimate: SkillEstimate = {
    sigma,
    pmax,
    confidence,
  };

  const sessionStats: SessionStats = {
    totalWagered: shots.reduce((sum, shot) => sum + shot.wager, 0),
    totalWon: shots.reduce((sum, shot) => sum + shot.payout, 0),
    netProfit: shots.reduce((sum, shot) => sum + shot.profit, 0),
    rtp: shots.length > 0
      ? (shots.reduce((sum, shot) => sum + shot.payout, 0) / shots.reduce((sum, shot) => sum + shot.wager, 0)) * 100
      : 0,
    shotCount: shots.length,
  };

  const breakevenRadius = currentHole.targetRadius * (1 - Math.pow(1 / pmax, 1 / currentHole.k));

  const shootOnce = useCallback((wager: number, devDistance?: number) => {
    const { distance, angle } = simulateShot(sigma, devDistance);
    const multiplier = calculateMultiplier(distance, pmax, currentHole.targetRadius, currentHole.k);
    const payout = wager * multiplier;
    const profit = payout - wager;

    const newShot: Shot = {
      distance,
      angle,
      wager,
      payout,
      profit,
      multiplier,
    };

    setShots(prev => [...prev, newShot]);
    setPmaxHistory(prev => [...prev, {
      shotNumber: shots.length + 1,
      pmax,
      confidence,
      sigma,
    }]);
  }, [sigma, pmax, currentHole, shots.length, confidence]);

  const shootBatch = useCallback((wager: number, count: number) => {
    const newShots: Shot[] = [];
    const newHistory: PmaxDataPoint[] = [];

    for (let i = 0; i < count; i++) {
      const { distance, angle } = simulateShot(sigma);
      const multiplier = calculateMultiplier(distance, pmax, currentHole.targetRadius, currentHole.k);
      const payout = wager * multiplier;
      const profit = payout - wager;

      newShots.push({
        distance,
        angle,
        wager,
        payout,
        profit,
        multiplier,
      });

      newHistory.push({
        shotNumber: shots.length + i + 1,
        pmax,
        confidence,
        sigma,
      });
    }

    setShots(prev => [...prev, ...newShots]);
    setPmaxHistory(prev => [...prev, ...newHistory]);
  }, [sigma, pmax, currentHole, shots.length, confidence]);

  const reset = useCallback(() => {
    setShots([]);
    setPmaxHistory([]);
  }, []);

  return {
    shots,
    skillEstimate,
    pmaxHistory,
    sessionStats,
    breakevenRadius,
    currentHole,
    shootOnce,
    shootBatch,
    reset,
  };
};
