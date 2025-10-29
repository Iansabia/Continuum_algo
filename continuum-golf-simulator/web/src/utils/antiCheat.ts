/**
 * Frontend Anti-Cheat Detection
 *
 * Detects cheating patterns including:
 * - Sandbagging (intentional poor performance to inflate P_max)
 * - Cherry-picking (high wagers only on good shots)
 * - Sudden skill jumps (potential account sharing)
 * - Confidence anomalies (erratic skill patterns)
 */

export interface ShotData {
  missDistance: number;  // feet
  wager: number;
  multiplier: number;
  shotNumber: number;
}

export interface AnomalyReport {
  isSuspicious: boolean;
  confidence: number;  // 0.0-1.0
  detectedPatterns: string[];
  recommendedAction: string;
}

/**
 * Detect sandbagging pattern
 *
 * Indicators:
 * - High variance in miss distances
 * - Sudden high wagers after establishing poor baseline
 */
export function detectSandbagging(shots: ShotData[]): AnomalyReport {
  if (shots.length < 20) {
    return {
      isSuspicious: false,
      confidence: 0.0,
      detectedPatterns: [],
      recommendedAction: 'Insufficient data',
    };
  }

  const patterns: string[] = [];
  let confidence = 0.0;

  // Check variance in miss distances
  const meanMiss = shots.reduce((sum, s) => sum + s.missDistance, 0) / shots.length;
  const variance = shots.reduce((sum, s) => sum + Math.pow(s.missDistance - meanMiss, 2), 0) / shots.length;
  const stdDev = Math.sqrt(variance);

  if (stdDev > meanMiss * 0.8) {
    patterns.push(`High variance in shot quality (σ=${stdDev.toFixed(1)})`);
    confidence += 0.3;
  }

  // Check correlation between wager size and shot quality
  const correlation = calculateWagerQualityCorrelation(shots);
  if (correlation < -0.5) {
    patterns.push(`Negative correlation: high wagers on bad shots (${correlation.toFixed(2)})`);
    confidence += 0.4;
  }

  // Check for wager pattern changes
  if (shots.length >= 50) {
    const firstHalfAvgWager = shots.slice(0, 25).reduce((sum, s) => sum + s.wager, 0) / 25;
    const secondHalfAvgWager = shots.slice(25).reduce((sum, s) => sum + s.wager, 0) / (shots.length - 25);

    if (secondHalfAvgWager > firstHalfAvgWager * 5.0) {
      patterns.push('Sudden wager increase after baseline period');
      confidence += 0.3;
    }
  }

  const isSuspicious = confidence >= 0.6;
  const recommendedAction = isSuspicious
    ? 'Flag for manual review - potential sandbagging'
    : 'Continue monitoring';

  return {
    isSuspicious,
    confidence,
    detectedPatterns: patterns,
    recommendedAction,
  };
}

/**
 * Detect cherry-picking (bet timing exploitation)
 *
 * Indicators:
 * - High wagers correlated with good shots
 * - Bimodal wager distribution with better multipliers on high wagers
 */
export function detectCherryPicking(shots: ShotData[]): AnomalyReport {
  if (shots.length < 10) {
    return {
      isSuspicious: false,
      confidence: 0.0,
      detectedPatterns: [],
      recommendedAction: 'Insufficient data',
    };
  }

  const patterns: string[] = [];
  let confidence = 0.0;

  // Calculate correlation between wager and payout multiplier
  const correlation = calculateWagerQualityCorrelation(shots);

  if (correlation > 0.5) {
    patterns.push(`Strong positive correlation: high wagers on good shots (${correlation.toFixed(2)})`);
    confidence += 0.5;
  }

  // Check for bimodal wager distribution
  const avgWager = shots.reduce((sum, s) => sum + s.wager, 0) / shots.length;
  const lowWagerShots = shots.filter(s => s.wager < avgWager);
  const highWagerShots = shots.filter(s => s.wager >= avgWager);

  if (lowWagerShots.length > 0 && highWagerShots.length > 0) {
    const lowAvgMult = lowWagerShots.reduce((sum, s) => sum + s.multiplier, 0) / lowWagerShots.length;
    const highAvgMult = highWagerShots.reduce((sum, s) => sum + s.multiplier, 0) / highWagerShots.length;

    if (highAvgMult > lowAvgMult * 1.5) {
      patterns.push('Bimodal betting: significantly better multipliers on high wagers');
      confidence += 0.4;
    }
  }

  const isSuspicious = confidence > 0.6;
  const recommendedAction = isSuspicious
    ? 'Limit max wager variance per session'
    : 'Normal betting pattern';

  return {
    isSuspicious,
    confidence,
    detectedPatterns: patterns,
    recommendedAction,
  };
}

/**
 * Detect sudden skill jumps (potential account sharing)
 *
 * Compares recent performance to historical baseline
 */
export function detectSkillJump(shots: ShotData[]): AnomalyReport {
  if (shots.length < 30) {
    return {
      isSuspicious: false,
      confidence: 0.0,
      detectedPatterns: [],
      recommendedAction: 'Insufficient data for comparison',
    };
  }

  const patterns: string[] = [];
  let confidence = 0.0;

  // Split shots: first 70% vs last 30%
  const splitPoint = Math.floor(shots.length * 0.7);
  const historicalShots = shots.slice(0, splitPoint);
  const recentShots = shots.slice(splitPoint);

  // Compare average performance
  const historicalAvgMiss = historicalShots.reduce((sum, s) => sum + s.missDistance, 0) / historicalShots.length;
  const recentAvgMiss = recentShots.reduce((sum, s) => sum + s.missDistance, 0) / recentShots.length;

  const improvementRate = (historicalAvgMiss - recentAvgMiss) / historicalAvgMiss;

  if (improvementRate > 0.4) {
    patterns.push(`Sudden skill improvement: ${(improvementRate * 100).toFixed(1)}% better`);
    confidence += 0.5;
  }

  // Check wager increase coinciding with skill jump
  const historicalAvgWager = historicalShots.reduce((sum, s) => sum + s.wager, 0) / historicalShots.length;
  const recentAvgWager = recentShots.reduce((sum, s) => sum + s.wager, 0) / recentShots.length;

  if (recentAvgWager > historicalAvgWager * 3.0 && improvementRate > 0.3) {
    patterns.push('Skill jump coincides with increased wagers');
    confidence += 0.4;
  }

  const isSuspicious = confidence > 0.7;
  const recommendedAction = isSuspicious
    ? 'URGENT: Flag for immediate review - possible account sharing'
    : confidence > 0.5
    ? 'Monitor closely for continued pattern'
    : 'Normal skill progression';

  return {
    isSuspicious,
    confidence,
    detectedPatterns: patterns,
    recommendedAction,
  };
}

/**
 * Detect confidence anomalies using EWMA approach
 *
 * Tracks confidence volatility and detects abnormal swings
 */
export function detectConfidenceAnomaly(confidenceHistory: Array<{ shotNumber: number; confidence: number }>): AnomalyReport {
  if (confidenceHistory.length < 10) {
    return {
      isSuspicious: false,
      confidence: 0.0,
      detectedPatterns: [],
      recommendedAction: 'Insufficient confidence history',
    };
  }

  const patterns: string[] = [];
  let suspicionScore = 0.0;

  // Check for sudden drops (>30% drop in confidence)
  let maxDrop = 0.0;
  for (let i = 1; i < confidenceHistory.length; i++) {
    const prevConf = confidenceHistory[i - 1].confidence;
    const currConf = confidenceHistory[i].confidence;

    // Only check for drops when previous confidence was reasonably high (>40%)
    if (prevConf > 40.0) {
      const drop = prevConf - currConf;
      if (drop > maxDrop) {
        maxDrop = drop;
      }
    }
  }

  if (maxDrop > 30.0) {
    patterns.push(`Sudden confidence drop: ${maxDrop.toFixed(1)}% → indicates skill inconsistency`);
    suspicionScore += 0.5;
  }

  // Check for multiple moderate drops (>15% each)
  let moderateDrops = 0;
  for (let i = 1; i < confidenceHistory.length; i++) {
    const prevConf = confidenceHistory[i - 1].confidence;
    const currConf = confidenceHistory[i].confidence;

    if (prevConf > 30.0 && (prevConf - currConf) > 15.0) {
      moderateDrops++;
    }
  }

  if (moderateDrops >= 3) {
    patterns.push(`Multiple confidence drops (${moderateDrops}x) → erratic skill pattern`);
    suspicionScore += 0.3;
  }

  // EWMA volatility detection - only flags abnormal swings after stabilization
  const deltas: number[] = [];
  for (let i = 1; i < confidenceHistory.length; i++) {
    deltas.push(Math.abs(confidenceHistory[i].confidence - confidenceHistory[i - 1].confidence));
  }

  if (deltas.length >= 5) {
    // Calculate baseline volatility (average delta across all history)
    const baselineVolatility = deltas.reduce((sum, d) => sum + d, 0) / deltas.length;

    // Calculate recent volatility (last 3 measurements)
    const recentStart = Math.max(0, deltas.length - 3);
    const recentDeltas = deltas.slice(recentStart);
    const recentVolatility = recentDeltas.reduce((sum, d) => sum + d, 0) / recentDeltas.length;

    // Get total shots
    const totalShots = confidenceHistory[confidenceHistory.length - 1].shotNumber;

    // Flag only if recent volatility is significantly higher than baseline
    // AND we have enough data to establish a pattern (>30 shots)
    if (totalShots > 30 && recentVolatility > baselineVolatility * 3.0 && recentVolatility > 15.0) {
      patterns.push(
        `Abnormal confidence swings: recent volatility ${recentVolatility.toFixed(1)}% vs baseline ${baselineVolatility.toFixed(1)}% → unstable skill`
      );
      suspicionScore += 0.2;
    }
  }

  const isSuspicious = suspicionScore >= 0.6;
  const recommendedAction = isSuspicious
    ? 'ALERT: Possible account sharing or bot usage - investigate immediately'
    : suspicionScore >= 0.4
    ? 'CAUTION: Monitor for continued anomalies'
    : 'Normal confidence pattern';

  return {
    isSuspicious,
    confidence: suspicionScore,
    detectedPatterns: patterns,
    recommendedAction,
  };
}

/**
 * Calculate correlation between wager size and shot quality (multiplier)
 */
function calculateWagerQualityCorrelation(shots: ShotData[]): number {
  if (shots.length < 2) {
    return 0.0;
  }

  const n = shots.length;
  const meanWager = shots.reduce((sum, s) => sum + s.wager, 0) / n;
  const meanQuality = shots.reduce((sum, s) => sum + s.multiplier, 0) / n;

  const numerator = shots.reduce((sum, s) => sum + (s.wager - meanWager) * (s.multiplier - meanQuality), 0);

  const wagerVariance = shots.reduce((sum, s) => sum + Math.pow(s.wager - meanWager, 2), 0);
  const qualityVariance = shots.reduce((sum, s) => sum + Math.pow(s.multiplier - meanQuality, 2), 0);

  if (wagerVariance === 0.0 || qualityVariance === 0.0) {
    return 0.0;
  }

  return numerator / (Math.sqrt(wagerVariance) * Math.sqrt(qualityVariance));
}

/**
 * Run all anti-cheat detectors and return the most suspicious report
 */
export function runAntiCheatAnalysis(
  shots: ShotData[],
  confidenceHistory: Array<{ shotNumber: number; confidence: number }>
): AnomalyReport | null {
  if (shots.length < 20) {
    return null;
  }

  // Run all detectors
  const sandbagging = detectSandbagging(shots);
  const cherryPicking = detectCherryPicking(shots);
  const skillJump = detectSkillJump(shots);
  const confidenceAnomaly = detectConfidenceAnomaly(confidenceHistory);

  // Find the most suspicious report
  const reports = [sandbagging, cherryPicking, skillJump, confidenceAnomaly];
  const mostSuspicious = reports.reduce((max, report) =>
    report.confidence > max.confidence ? report : max
  );

  return mostSuspicious;
}
