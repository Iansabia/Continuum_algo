/**
 * Force Retrain Utility
 *
 * Deletes old model and trains a new one with 9-feature architecture
 */

import * as tf from '@tensorflow/tfjs';
import { trainModel, saveModel } from './antiCheatModel';

/**
 * Delete existing model from IndexedDB
 */
async function deleteOldModel(name: string = 'anti-cheat-autoencoder'): Promise<void> {
  try {
    await tf.io.removeModel(`indexeddb://${name}`);
    console.log(`✅ Deleted old model: ${name}`);
  } catch (error) {
    console.log('ℹ️ No existing model to delete');
  }
}

/**
 * Force retrain the model
 */
export async function forceRetrain(): Promise<tf.LayersModel> {
  console.log('🔄 Force retraining anti-cheat model...\n');

  // Delete old 8-feature model
  await deleteOldModel();

  // Train new 9-feature model
  console.log('📚 Training new model with 9 features...');
  const model = await trainModel(100, 32);

  // Save the new model
  await saveModel(model);

  console.log('✅ Retraining complete!\n');
  return model;
}

// Export for console use
if (typeof window !== 'undefined') {
  (window as any).forceRetrain = forceRetrain;
  console.log('💡 Use window.forceRetrain() to retrain the model');
}
