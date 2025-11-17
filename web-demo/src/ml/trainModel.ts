/**
 * Training Script for Anti-Cheat Neural Network
 *
 * Run this to retrain the model with new 9-feature architecture
 * Usage: node --loader ts-node/esm trainModel.ts
 */

import * as tf from '@tensorflow/tfjs';
import { trainModel, saveModel } from './antiCheatModel';

async function main() {
  console.log('🚀 Starting TensorFlow.js anti-cheat model training...\n');

  try {
    // Set backend (use CPU for Node.js, WebGL for browser)
    await tf.ready();
    console.log(`✅ TensorFlow.js backend: ${tf.getBackend()}\n`);

    // Train the model
    console.log('📚 Training autoencoder with:');
    console.log('  - 2000 normal player sessions');
    console.log('  - 9 features (including suspiciouslyPerfect)');
    console.log('  - 100 epochs');
    console.log('  - Batch size: 32\n');

    const model = await trainModel(100, 32);

    // Save the trained model
    console.log('\n💾 Saving model...');
    await saveModel(model, 'anti-cheat-autoencoder');

    console.log('\n✅ Training complete! Model saved to IndexedDB.');
    console.log('🔄 Refresh your browser to load the new model.\n');

    // Display model summary
    console.log('📊 Final Model Summary:');
    model.summary();

  } catch (error) {
    console.error('❌ Training failed:', error);
    process.exit(1);
  }
}

main();
