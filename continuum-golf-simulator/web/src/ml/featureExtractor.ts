/**
 * ML Feature Extraction for Anti-Cheat Neural Network
 *
 * Converts raw shot history into normalized 8-dimensional feature vector
 * for autoencoder anomaly detection.
 */

export interface ShotData {
  missDistance: number;  // feet
  wager: number;
  multiplier: number;
  shotNumber: number;
}

export interface MLFeatures {
  rtpScore: number;           // 0-1: Return to player (sigmoid scaled, flags extreme RTP)
  shotConsistency: number;    // 0-1: Coefficient of variation (lower = more consistent)
  wagerCorrelation: number;   // 0-1: Wager-outcome correlation (scaled from -1,1)
  skillProgression: number;   // 0-1: Rate of skill improvement
  pmaxUtilization: number;    // 0-1: Efficiency of P_max usage
  confidenceStability: number; // 0-1: Inverse of confidence variance
  sessionIntensity: number;   // 0-1: Shots per minute (normalized)
  extremeOutcomes: number;    // 0-1: Frequency of outlier shots
  suspiciouslyPerfect: number; // 0-1: Detects "too good to be true" patterns
}

/**
 * Extract ML features from shot history and confidence data
 */
export function extractFeatures(
  shots: ShotData[],
  confidenceHistory: Array<{ shotNumber: number; confidence: number }>,
  sessionDurationMinutes: number = 10
): MLFeatures {
  if (shots.length < 10) {
    // Return neutral features for insufficient data
    return {
      rtpScore: 0.5,
      shotConsistency: 0.5,
      wagerCorrelation: 0.5,
      skillProgression: 0.5,
      pmaxUtilization: 0.5,
      confidenceStability: 0.5,
      sessionIntensity: 0.5,
      extremeOutcomes: 0.5,
      suspiciouslyPerfect: 0.0,
    };
  }

  // 1. RTP Score (Return to Player) - sigmoid to flag extreme values
  const totalWagered = shots.reduce((sum, s) => sum + s.wager, 0);
  const totalWon = shots.reduce((sum, s) => sum + (s.multiplier * s.wager), 0);
  const rtp = totalWon / totalWagered;
  // Sigmoid centered at RTP=1.5 (150%), flags extreme RTP >3.0 as ~1.0
  const rtpScore = 1.0 / (1.0 + Math.exp(-2.0 * (rtp - 1.5)));

  // 2. Shot Consistency (Coefficient of Variation)
  const missDistances = shots.map(s => s.missDistance);
  const meanMiss = missDistances.reduce((sum, d) => sum + d, 0) / missDistances.length;
  const stdDev = Math.sqrt(
    missDistances.reduce((sum, d) => sum + Math.pow(d - meanMiss, 2), 0) / missDistances.length
  );
  const coefficientOfVariation = meanMiss > 0 ? stdDev / meanMiss : 0;
  const shotConsistency = 1.0 - Math.min(coefficientOfVariation / 2.0, 1.0); // Lower CoV = higher consistency

  // 3. Wager-Outcome Correlation
  const meanWager = shots.reduce((sum, s) => sum + s.wager, 0) / shots.length;
  const meanMultiplier = shots.reduce((sum, s) => sum + s.multiplier, 0) / shots.length;

  const numerator = shots.reduce((sum, s) =>
    sum + (s.wager - meanWager) * (s.multiplier - meanMultiplier), 0);
  const wagerVar = shots.reduce((sum, s) => sum + Math.pow(s.wager - meanWager, 2), 0);
  const multVar = shots.reduce((sum, s) => sum + Math.pow(s.multiplier - meanMultiplier, 2), 0);

  const correlation = (wagerVar > 0 && multVar > 0)
    ? numerator / (Math.sqrt(wagerVar) * Math.sqrt(multVar))
    : 0;
  const wagerCorrelation = (correlation + 1.0) / 2.0; // Scale from [-1,1] to [0,1]

  // 4. Skill Progression (rate of miss distance improvement)
  const firstThirdShots = shots.slice(0, Math.floor(shots.length / 3));
  const lastThirdShots = shots.slice(Math.floor(shots.length * 2 / 3));

  const earlyAvgMiss = firstThirdShots.reduce((sum, s) => sum + s.missDistance, 0) / firstThirdShots.length;
  const lateAvgMiss = lastThirdShots.reduce((sum, s) => sum + s.missDistance, 0) / lastThirdShots.length;

  const improvementRate = earlyAvgMiss > 0 ? (earlyAvgMiss - lateAvgMiss) / earlyAvgMiss : 0;
  const skillProgression = Math.max(0, Math.min(improvementRate + 0.5, 1.0)); // Center at 0.5, cap at 1.0

  // 5. P_max Utilization (theoretical vs actual payout efficiency)
  // Higher multipliers on fewer shots = better P_max usage
  const top20PercentCount = Math.ceil(shots.length * 0.2);
  const topShots = [...shots].sort((a, b) => b.multiplier - a.multiplier).slice(0, top20PercentCount);
  const topShotsAvgWager = topShots.reduce((sum, s) => sum + s.wager, 0) / topShots.length;
  const avgWager = shots.reduce((sum, s) => sum + s.wager, 0) / shots.length;
  const pmaxUtilization = avgWager > 0 ? Math.min(topShotsAvgWager / avgWager / 3.0, 1.0) : 0.5;

  // 6. Confidence Stability (inverse variance of confidence)
  if (confidenceHistory.length >= 3) {
    const confidences = confidenceHistory.map(c => c.confidence);
    const meanConf = confidences.reduce((sum, c) => sum + c, 0) / confidences.length;
    const confVariance = confidences.reduce((sum, c) => sum + Math.pow(c - meanConf, 2), 0) / confidences.length;
    const confidenceStability = 1.0 - Math.min(confVariance / 1000, 1.0); // Normalize variance

    // 7. Session Intensity (shots per minute)
    const shotsPerMinute = shots.length / sessionDurationMinutes;
    const sessionIntensity = Math.min(shotsPerMinute / 10.0, 1.0); // Cap at 10 shots/min = 1.0

    // 8. Extreme Outcomes (frequency of shots >2σ from mean)
    const threshold = meanMiss + 2 * stdDev;
    const extremeCount = shots.filter(s => s.missDistance > threshold || s.missDistance < meanMiss - 2 * stdDev).length;
    const extremeOutcomes = extremeCount / shots.length;

    // 9. Suspiciously Perfect - detects "too good to be true" patterns
    // Flags: very low variance (<1 ft) + extreme RTP (>200%)
    const isLowVariance = stdDev < 1.0;
    const isExtremeRTP = rtp > 2.0;
    const isPerfectConsistency = stdDev < 0.1;
    const suspiciouslyPerfect = (isLowVariance && isExtremeRTP) ? 1.0 :
                                 (isPerfectConsistency && rtp > 1.5) ? 0.8 :
                                 (isLowVariance || isExtremeRTP) ? 0.5 : 0.0;

    return {
      rtpScore,
      shotConsistency,
      wagerCorrelation,
      skillProgression,
      pmaxUtilization,
      confidenceStability,
      sessionIntensity,
      extremeOutcomes,
      suspiciouslyPerfect,
    };
  }

  // Fallback for insufficient confidence data
  // Still calculate suspiciouslyPerfect even without confidence history
  const isLowVariance = stdDev < 1.0;
  const isExtremeRTP = rtp > 2.0;
  const isPerfectConsistency = stdDev < 0.1;
  const suspiciouslyPerfect = (isLowVariance && isExtremeRTP) ? 1.0 :
                               (isPerfectConsistency && rtp > 1.5) ? 0.8 :
                               (isLowVariance || isExtremeRTP) ? 0.5 : 0.0;

  return {
    rtpScore,
    shotConsistency,
    wagerCorrelation,
    skillProgression,
    pmaxUtilization,
    confidenceStability: 0.5,
    sessionIntensity: 0.5,
    extremeOutcomes: 0.5,
    suspiciouslyPerfect,
  };
}

/**
 * Convert MLFeatures object to tensor-ready array
 */
export function featuresToArray(features: MLFeatures): number[] {
  return [
    features.rtpScore,
    features.shotConsistency,
    features.wagerCorrelation,
    features.skillProgression,
    features.pmaxUtilization,
    features.confidenceStability,
    features.sessionIntensity,
    features.extremeOutcomes,
    features.suspiciouslyPerfect,
  ];
}

/**
 * Normalize features to ensure they're in [0, 1] range
 */
export function normalizeFeatures(features: MLFeatures): MLFeatures {
  return {
    rtpScore: Math.max(0, Math.min(features.rtpScore, 1)),
    shotConsistency: Math.max(0, Math.min(features.shotConsistency, 1)),
    wagerCorrelation: Math.max(0, Math.min(features.wagerCorrelation, 1)),
    skillProgression: Math.max(0, Math.min(features.skillProgression, 1)),
    pmaxUtilization: Math.max(0, Math.min(features.pmaxUtilization, 1)),
    confidenceStability: Math.max(0, Math.min(features.confidenceStability, 1)),
    sessionIntensity: Math.max(0, Math.min(features.sessionIntensity, 1)),
    extremeOutcomes: Math.max(0, Math.min(features.extremeOutcomes, 1)),
    suspiciouslyPerfect: Math.max(0, Math.min(features.suspiciouslyPerfect, 1)),
  };
}
