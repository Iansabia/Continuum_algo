/**
 * Anti-Cheat Autoencoder Neural Network
 *
 * Architecture:
 * - Input: 9 features (normalized 0-1)
 * - Encoder: 9 → 6 → 3 (compression)
 * - Decoder: 3 → 6 → 9 (reconstruction)
 * - Loss: MSE (mean squared error)
 *
 * Detection: High reconstruction error = anomaly (cheater)
 */

import * as tf from '@tensorflow/tfjs';
import { generateTrainingData, generateValidationData, analyzeDataset } from './syntheticDataGenerator';

export interface AnomalyResult {
  reconstructionError: number;  // MSE between input and output
  suspicionScore: number;        // 0-100% mapped from error
  threatLevel: 'CLEAN' | 'MONITOR' | 'WARNING' | 'CRITICAL';
  isAnomaly: boolean;
  confidence: number;             // How confident the model is
}

/**
 * Create autoencoder model
 */
export function createAutoencoderModel(): tf.LayersModel {
  // Encoder
  const input = tf.input({ shape: [9] });

  const encoded = tf.layers.dense({
    units: 6,
    activation: 'relu',
    kernelInitializer: 'heNormal',
    name: 'encoder_layer1',
  }).apply(input) as tf.SymbolicTensor;

  const bottleneck = tf.layers.dense({
    units: 3,
    activation: 'relu',
    kernelInitializer: 'heNormal',
    name: 'bottleneck',
  }).apply(encoded) as tf.SymbolicTensor;

  // Decoder
  const decoded = tf.layers.dense({
    units: 6,
    activation: 'relu',
    kernelInitializer: 'heNormal',
    name: 'decoder_layer1',
  }).apply(bottleneck) as tf.SymbolicTensor;

  const output = tf.layers.dense({
    units: 9,
    activation: 'sigmoid',
    kernelInitializer: 'glorotUniform',
    name: 'output',
  }).apply(decoded) as tf.SymbolicTensor;

  const model = tf.model({ inputs: input, outputs: output });

  model.compile({
    optimizer: tf.train.adam(0.001),
    loss: 'meanSquaredError',
    metrics: ['mse'],
  });

  return model;
}

/**
 * Train the autoencoder model
 */
export async function trainModel(
  numEpochs: number = 100,
  batchSize: number = 32
): Promise<tf.LayersModel> {
  console.log('🧠 Creating autoencoder model...');
  const model = createAutoencoderModel();

  console.log('\n📊 Model Architecture:');
  model.summary();

  // Generate training data (increased from 1000 to 2000)
  const trainingData = generateTrainingData(2000);
  analyzeDataset(trainingData);

  // Convert to tensors
  const xs = tf.tensor2d(trainingData);

  console.log('\n🏋️ Training model...');
  await model.fit(xs, xs, {
    epochs: numEpochs,
    batchSize,
    shuffle: true,
    validationSplit: 0.1,
    callbacks: {
      onEpochEnd: (epoch, logs) => {
        if ((epoch + 1) % 10 === 0) {
          console.log(
            `Epoch ${epoch + 1}/${numEpochs} - ` +
            `loss: ${logs?.loss.toFixed(4)} - ` +
            `val_loss: ${logs?.val_loss?.toFixed(4)}`
          );
        }
      },
    },
  });

  // Test on validation data (increased cheating samples from 50 to 200)
  console.log('\n🧪 Testing on validation data...');
  const validation = generateValidationData(200, 200);

  const normalXs = tf.tensor2d(validation.normal);
  const normalPreds = model.predict(normalXs) as tf.Tensor;
  const normalErrors = tf.losses.meanSquaredError(normalXs, normalPreds);
  const normalMeanError = await normalErrors.mean().data();

  const cheatingXs = tf.tensor2d(validation.cheating);
  const cheatingPreds = model.predict(cheatingXs) as tf.Tensor;
  const cheatingErrors = tf.losses.meanSquaredError(cheatingXs, cheatingPreds);
  const cheatingMeanError = await cheatingErrors.mean().data();

  console.log(`\n📈 Validation Results:`);
  console.log(`Normal players - Mean reconstruction error: ${normalMeanError[0].toFixed(4)}`);
  console.log(`Cheating players - Mean reconstruction error: ${cheatingMeanError[0].toFixed(4)}`);
  console.log(`Separation ratio: ${(cheatingMeanError[0] / normalMeanError[0]).toFixed(2)}x`);

  // Cleanup
  xs.dispose();
  normalXs.dispose();
  normalPreds.dispose();
  normalErrors.dispose();
  cheatingXs.dispose();
  cheatingPreds.dispose();
  cheatingErrors.dispose();

  console.log('\n✅ Training complete!');
  return model;
}

/**
 * Run inference on feature vector
 */
export function detectAnomaly(
  model: tf.LayersModel,
  features: number[]
): AnomalyResult {
  return tf.tidy(() => {
    // Convert to tensor
    const input = tf.tensor2d([features]);

    // Get reconstruction
    const output = model.predict(input) as tf.Tensor;

    // Calculate reconstruction error (MSE)
    const error = tf.losses.meanSquaredError(input, output);
    const reconstructionError = error.dataSync()[0];

    // Map error to suspicion score
    // Based on validation: normal ~0.01-0.02, cheating ~0.15-0.30
    let suspicionScore: number;
    let threatLevel: 'CLEAN' | 'MONITOR' | 'WARNING' | 'CRITICAL';

    if (reconstructionError < 0.05) {
      // Very low error → normal player
      suspicionScore = (reconstructionError / 0.05) * 20; // 0-20%
      threatLevel = 'CLEAN';
    } else if (reconstructionError < 0.15) {
      // Low-moderate error → monitor
      suspicionScore = 20 + ((reconstructionError - 0.05) / 0.10) * 30; // 20-50%
      threatLevel = 'MONITOR';
    } else if (reconstructionError < 0.30) {
      // Moderate-high error → warning
      suspicionScore = 50 + ((reconstructionError - 0.15) / 0.15) * 30; // 50-80%
      threatLevel = 'WARNING';
    } else {
      // Very high error → critical
      suspicionScore = Math.min(80 + ((reconstructionError - 0.30) / 0.20) * 20, 100); // 80-100%
      threatLevel = 'CRITICAL';
    }

    const isAnomaly = suspicionScore >= 50;
    const confidence = Math.min(suspicionScore / 100, 1.0);

    return {
      reconstructionError,
      suspicionScore,
      threatLevel,
      isAnomaly,
      confidence,
    };
  });
}

/**
 * Save model to IndexedDB for browser persistence
 */
export async function saveModel(model: tf.LayersModel, name: string = 'anti-cheat-autoencoder'): Promise<void> {
  await model.save(`indexeddb://${name}`);
  console.log(`✅ Model saved to IndexedDB as "${name}"`);
}

/**
 * Load model from IndexedDB
 */
export async function loadModel(name: string = 'anti-cheat-autoencoder'): Promise<tf.LayersModel | null> {
  try {
    const model = await tf.loadLayersModel(`indexeddb://${name}`);
    console.log(`✅ Model loaded from IndexedDB: "${name}"`);
    return model;
  } catch (error) {
    console.warn(`⚠️ Could not load model from IndexedDB:`, error);
    return null;
  }
}

/**
 * Initialize: Load existing model or train new one
 */
export async function initializeAntiCheatModel(): Promise<tf.LayersModel> {
  console.log('🚀 Initializing anti-cheat AI model...');

  // Try to load existing model
  let model = await loadModel();

  if (!model) {
    console.log('📦 No existing model found, training new model...');
    model = await trainModel(100, 32); // Train for 100 epochs with 2000 samples
    await saveModel(model);
  }

  return model;
}
