/**
 * Synthetic Training Data Generator
 *
 * Generates realistic "normal player" sessions for training the autoencoder.
 * Normal players have:
 * - RTP between 75-125% (natural variance)
 * - Consistent shot patterns
 * - Random wagers (no correlation with outcomes)
 * - Gradual skill progression
 * - Stable confidence growth
 */

import { extractFeatures, featuresToArray, type ShotData } from './featureExtractor';

/**
 * Generate a single normal player session
 */
function generateNormalSession(): number[] {
  const numShots = 20 + Math.floor(Math.random() * 80); // 20-100 shots
  const baseHandicap = 5 + Math.random() * 15; // Handicap 5-20
  const baseSigma = 3 + baseHandicap * 0.5; // Base skill level

  const shots: ShotData[] = [];
  const confidenceHistory: Array<{ shotNumber: number; confidence: number }> = [];

  // Simulate gradual skill improvement
  const improvementRate = -0.001 * (Math.random() * 0.5 + 0.5); // -0.05% to -0.1% per shot

  for (let i = 0; i < numShots; i++) {
    // Skill improves slightly over time
    const currentSigma = baseSigma * (1 + improvementRate * i);

    // Generate miss distance using Rayleigh-like distribution
    const u1 = Math.random();
    const rayleighSample = currentSigma * Math.sqrt(-2 * Math.log(u1));
    const missDistance = Math.abs(rayleighSample) * 3; // Convert to feet

    // Random wager (no correlation with outcome)
    const wager = 5 + Math.random() * 15; // $5-$20

    // Calculate multiplier based on miss distance (simplified payout curve)
    const targetRadius = 6.0; // yards
    const breakeven = 2.5; // yards
    let multiplier = 0;

    if (missDistance / 3 < breakeven) {
      // Winner
      const distanceYards = missDistance / 3;
      const maxMult = 10.0;
      multiplier = maxMult * Math.exp(-(distanceYards * distanceYards) / (2 * targetRadius * targetRadius));
    }

    shots.push({
      missDistance,
      wager,
      multiplier,
      shotNumber: i + 1,
    });

    // Track confidence (grows with shots, plateaus around 50-70%)
    if (i % 5 === 0) {
      const confidence = Math.min(50 + (i / numShots) * 30, 70);
      confidenceHistory.push({ shotNumber: i + 1, confidence });
    }
  }

  // Extract features
  const sessionDuration = numShots / (1 + Math.random() * 2); // 0.5-2 shots per minute
  const features = extractFeatures(shots, confidenceHistory, sessionDuration);

  return featuresToArray(features);
}

/**
 * Generate a cheating session (for validation, not training)
 */
function generateCheatingSession(): number[] {
  const cheatType = Math.random();

  if (cheatType < 0.25) {
    // Type 1: Bot/Fixed shots - ALL shots 0-2 ft (EXTREME)
    return generateBotSession();
  } else if (cheatType < 0.5) {
    // Type 2: Perfect exploit - ALL shots exactly 0 ft
    return generateFixedValueSession();
  } else if (cheatType < 0.75) {
    // Type 3: Cherry-picking with high correlation
    return generateCherryPickingSession();
  } else {
    // Type 4: Sandbagging with bimodal performance
    return generateSandbaggingSession();
  }
}

/**
 * Bot session: All shots within 0-2 ft, realistic RTP (300-600%)
 */
function generateBotSession(): number[] {
  const numShots = 20 + Math.floor(Math.random() * 80); // 20-100 shots like real games
  const shots: ShotData[] = [];
  const confidenceHistory: Array<{ shotNumber: number; confidence: number }> = [];

  for (let i = 0; i < numShots; i++) {
    const missDistance = Math.random() * 2.0; // 0-2 ft
    const wager = 10 + Math.random() * 10;
    // Realistic multipliers: 3.0-5.0 (matches P_max clamping in real game)
    const multiplier = 3.0 + Math.random() * 2.0;

    shots.push({ missDistance, wager, multiplier, shotNumber: i + 1 });

    if (i % 5 === 0) {
      confidenceHistory.push({ shotNumber: i + 1, confidence: 60 + Math.random() * 20 });
    }
  }

  return featuresToArray(extractFeatures(shots, confidenceHistory, 10));
}

/**
 * Fixed value exploit: ALL shots exactly 0 ft (realistic game conditions)
 */
function generateFixedValueSession(): number[] {
  const numShots = 20 + Math.floor(Math.random() * 80); // 20-100 shots like real games
  const shots: ShotData[] = [];
  const confidenceHistory: Array<{ shotNumber: number; confidence: number }> = [];

  for (let i = 0; i < numShots; i++) {
    const missDistance = 0.0; // EXACTLY 0 ft - impossible naturally
    const wager = 10 + Math.random() * 10;
    // Realistic P_max clamping: game clamps to 5.0 max in practice
    const multiplier = 5.0;

    shots.push({ missDistance, wager, multiplier, shotNumber: i + 1 });

    if (i % 5 === 0) {
      confidenceHistory.push({ shotNumber: i + 1, confidence: 70 + Math.random() * 10 });
    }
  }

  return featuresToArray(extractFeatures(shots, confidenceHistory, 10));
}

/**
 * Cherry-picking: High wager correlation with realistic multipliers
 */
function generateCherryPickingSession(): number[] {
  const numShots = 20 + Math.floor(Math.random() * 80);
  const shots: ShotData[] = [];
  const confidenceHistory: Array<{ shotNumber: number; confidence: number }> = [];

  for (let i = 0; i < numShots; i++) {
    const isGoodShot = Math.random() < 0.5;
    const missDistance = isGoodShot ? Math.random() * 5 : 50 + Math.random() * 50;
    const wager = isGoodShot ? 15 + Math.random() * 10 : 5 + Math.random() * 5;
    // Realistic multipliers: 2.0-5.0 for wins, 0 for losses
    const multiplier = missDistance < 10 ? 2.0 + Math.random() * 3.0 : 0;

    shots.push({ missDistance, wager, multiplier, shotNumber: i + 1 });

    if (i % 5 === 0) {
      confidenceHistory.push({ shotNumber: i + 1, confidence: 40 + Math.random() * 40 });
    }
  }

  return featuresToArray(extractFeatures(shots, confidenceHistory, 10));
}

/**
 * Sandbagging: Bimodal performance with realistic multipliers
 */
function generateSandbaggingSession(): number[] {
  const numShots = 20 + Math.floor(Math.random() * 80);
  const shots: ShotData[] = [];
  const confidenceHistory: Array<{ shotNumber: number; confidence: number }> = [];

  for (let i = 0; i < numShots; i++) {
    const isGoodShot = Math.random() < 0.3;
    const missDistance = isGoodShot ? Math.random() * 5 : 50 + Math.random() * 50;
    const wager = 10 + Math.random() * 10;
    // Realistic multipliers: 3.0-5.0 for wins, 0 for losses
    const multiplier = missDistance < 10 ? 3.0 + Math.random() * 2.0 : 0;

    shots.push({ missDistance, wager, multiplier, shotNumber: i + 1 });

    if (i % 5 === 0) {
      confidenceHistory.push({ shotNumber: i + 1, confidence: 40 + Math.random() * 40 });
    }
  }

  return featuresToArray(extractFeatures(shots, confidenceHistory, 10));
}

/**
 * Generate training dataset (normal players only)
 */
export function generateTrainingData(numSessions: number = 1000): number[][] {
  console.log(`Generating ${numSessions} normal player sessions...`);
  const trainingData: number[][] = [];

  for (let i = 0; i < numSessions; i++) {
    trainingData.push(generateNormalSession());
  }

  console.log('✅ Training data generated');
  return trainingData;
}

/**
 * Generate validation dataset (mix of normal and cheating)
 */
export function generateValidationData(numNormal: number = 100, numCheating: number = 50): {
  normal: number[][];
  cheating: number[][];
} {
  console.log(`Generating validation data: ${numNormal} normal, ${numCheating} cheating...`);

  const normal: number[][] = [];
  const cheating: number[][] = [];

  for (let i = 0; i < numNormal; i++) {
    normal.push(generateNormalSession());
  }

  for (let i = 0; i < numCheating; i++) {
    cheating.push(generateCheatingSession());
  }

  console.log('✅ Validation data generated');
  return { normal, cheating };
}

/**
 * Calculate statistics of generated data
 */
export function analyzeDataset(data: number[][]): void {
  const featureNames = [
    'RTP',
    'Consistency',
    'Wager Correlation',
    'Skill Progression',
    'P_max Utilization',
    'Confidence Stability',
    'Session Intensity',
    'Extreme Outcomes',
    'Suspiciously Perfect',
  ];

  console.log('\n📊 Dataset Statistics:');
  console.log(`Total sessions: ${data.length}`);

  for (let i = 0; i < 9; i++) {
    const values = data.map(session => session[i]);
    const mean = values.reduce((sum, v) => sum + v, 0) / values.length;
    const min = Math.min(...values);
    const max = Math.max(...values);

    console.log(`${featureNames[i]}: mean=${mean.toFixed(3)}, range=[${min.toFixed(3)}, ${max.toFixed(3)}]`);
  }
}
