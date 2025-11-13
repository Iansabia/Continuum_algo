import { useState } from 'react';
import PlayerSimulator from './components/PlayerSimulator';
import PatternDrawingDemo from './components/PatternDrawingDemo';
import VenueSimulator from './components/VenueSimulator';
import { MeshGradient } from '@paper-design/shaders-react';

type TabType = 'simulator' | 'pattern-demo' | 'venue-sim';

function App() {
  const [activeTab, setActiveTab] = useState<TabType>('simulator');

  return (
    <div className="h-screen w-screen overflow-hidden bg-transparent flex flex-col relative">
      {/* Animated MeshGradient Background with Company Colors */}
      <div className="fixed top-0 left-0 w-full h-full -z-10">
        <MeshGradient
          className="w-full h-full"
          colors={["#493b7c", "#604c9c", "#9e8cb4", "#dfc9ad"]}
          speed={0.5}
          distortion={0.5}
          swirl={0.3}
          grainMixer={0.2}
          grainOverlay={0.1}
        />
      </div>

      {/* Light Glass Header */}
      <header className="relative bg-gradient-to-br from-black/40 to-black/20 backdrop-blur-2xl border-b border-white/10 shadow-lg flex-shrink-0 z-10">
        <div className="absolute inset-0 bg-gradient-to-b from-white/5 to-transparent pointer-events-none"></div>
        <div className="relative flex items-center justify-between py-3 px-6">
          <div className="flex items-center gap-4">
            <h1 className="text-2xl font-semibold text-black dark:text-white tracking-tight">
              Continuum Golf
            </h1>
            <div className="hidden md:flex items-center gap-2 text-xs">
              <span className="px-2.5 py-1 bg-black/30 backdrop-blur-sm rounded-lg border border-white/20 text-black dark:text-white font-semibold">85% RTP</span>
              <span className="px-2.5 py-1 bg-black/30 backdrop-blur-sm rounded-lg border border-white/20 text-black dark:text-white font-semibold">Fair Play</span>
              <span className="px-2.5 py-1 bg-black/30 backdrop-blur-sm rounded-lg border border-white/20 text-black dark:text-white font-semibold">Adaptive</span>
            </div>
          </div>
          <div className="text-[10px] text-black/60 dark:text-white/60">
            Rust + WebAssembly
          </div>
        </div>

        {/* Tab Navigation */}
        <div className="relative flex gap-2 px-6 pb-3">
          <button
            onClick={() => setActiveTab('pattern-demo')}
            className={`px-4 py-2 rounded-xl text-sm font-semibold transition-all ${
              activeTab === 'pattern-demo'
                ? 'bg-gradient-to-br from-black/60 to-black/40 text-black shadow-lg border border-white/50'
                : 'bg-black/20 text-white/70 hover:bg-black/30 border border-white/20'
            }`}
          >
            Pattern Drawing Demo
          </button>
          <button
            onClick={() => setActiveTab('simulator')}
            className={`px-4 py-2 rounded-xl text-sm font-semibold transition-all ${
              activeTab === 'simulator'
                ? 'bg-gradient-to-br from-black/60 to-black/40 text-black shadow-lg border border-white/50'
                : 'bg-black/20 text-white/70 hover:bg-black/30 border border-white/20'
            }`}
          >
            Player Simulator
          </button>
          <button
            onClick={() => setActiveTab('venue-sim')}
            className={`px-4 py-2 rounded-xl text-sm font-semibold transition-all ${
              activeTab === 'venue-sim'
                ? 'bg-gradient-to-br from-black/60 to-black/40 text-black shadow-lg border border-white/50'
                : 'bg-black/20 text-white/70 hover:bg-black/30 border border-white/20'
            }`}
          >
            Venue Simulation
          </button>
        </div>
      </header>

      {/* Main Content - Responsive container */}
      <main className="flex-1 min-h-0 p-4 overflow-auto relative z-10">
        <div className="h-full max-w-[2000px] mx-auto">
          {activeTab === 'pattern-demo' && <PatternDrawingDemo />}
          {activeTab === 'simulator' && <PlayerSimulator />}
          {activeTab === 'venue-sim' && <VenueSimulator />}
        </div>
      </main>
    </div>
  );
}

export default App;
