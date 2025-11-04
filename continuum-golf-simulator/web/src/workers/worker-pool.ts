// Worker pool manager for parallel venue simulations
import * as Comlink from 'comlink';
import type { BaySimulationRequest, BaySimulationResult } from './venue-worker';

export class WorkerPool {
  private workers: Array<{ worker: Worker; api: any; busy: boolean }> = [];
  private readonly poolSize: number;

  constructor(poolSize?: number) {
    // Use number of logical processors or default to 8 for M4 Max
    this.poolSize = poolSize || Math.min(navigator.hardwareConcurrency || 8, 16);
    console.log(`🚀 Creating worker pool with ${this.poolSize} workers`);
    this.initializeWorkers();
  }

  private initializeWorkers() {
    for (let i = 0; i < this.poolSize; i++) {
      const worker = new Worker(
        new URL('./venue-worker.ts', import.meta.url),
        { type: 'module' }
      );

      const api = Comlink.wrap(worker);

      this.workers.push({
        worker,
        api,
        busy: false,
      });
    }
  }

  async simulateBays(
    requests: BaySimulationRequest[],
    onProgress?: (completed: number, total: number) => void
  ): Promise<BaySimulationResult[]> {
    const total = requests.length;
    let completed = 0;
    const results: BaySimulationResult[] = new Array(total);

    // Process requests in parallel using available workers
    await Promise.all(
      requests.map(async (request, index) => {
        // Get an available worker
        const workerSlot = await this.getAvailableWorker();

        try {
          workerSlot.busy = true;

          // Simulate the bay
          const result = await workerSlot.api.simulateBay(request);
          results[index] = result;

          completed++;
          if (onProgress) {
            onProgress(completed, total);
          }
        } finally {
          workerSlot.busy = false;
        }
      })
    );

    return results;
  }

  private async getAvailableWorker(): Promise<{ worker: Worker; api: any; busy: boolean }> {
    // Wait for an available worker
    while (true) {
      const available = this.workers.find(w => !w.busy);
      if (available) {
        return available;
      }
      // Wait a bit before checking again
      await new Promise(resolve => setTimeout(resolve, 10));
    }
  }

  terminate() {
    console.log('🛑 Terminating worker pool');
    this.workers.forEach(({ worker }) => worker.terminate());
    this.workers = [];
  }
}
