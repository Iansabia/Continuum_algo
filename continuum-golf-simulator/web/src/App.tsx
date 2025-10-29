import PlayerSimulator from './components/PlayerSimulator';
import AnimatedBackground from './components/AnimatedBackground';

function App() {
  return (
    <div className="h-screen w-screen overflow-hidden bg-black flex flex-col relative">
      {/* Animated Background - Global floating lines */}
      <AnimatedBackground />

      {/* Frosted Glass Purple Header */}
      <header className="relative bg-gradient-to-r from-[#493b7c]/30 via-[#604c9c]/30 to-[#493b7c]/30 backdrop-blur-2xl border-b border-[#604c9c]/30 shadow-lg flex-shrink-0 z-10">
        <div className="absolute inset-0 bg-gradient-to-b from-white/5 to-transparent pointer-events-none"></div>
        <div className="relative flex items-center justify-between py-3 px-6">
          <div className="flex items-center gap-4">
            <h1 className="text-2xl font-semibold text-[#dfc9ad] tracking-tight">
              Continuum Golf
            </h1>
            <div className="hidden md:flex items-center gap-2 text-xs">
              <span className="px-2.5 py-1 bg-[#604c9c]/20 backdrop-blur-sm rounded-lg border border-[#9e8cb4]/30 text-[#9e8cb4]">85% RTP</span>
              <span className="px-2.5 py-1 bg-[#604c9c]/20 backdrop-blur-sm rounded-lg border border-[#9e8cb4]/30 text-[#9e8cb4]">Fair Play</span>
              <span className="px-2.5 py-1 bg-[#604c9c]/20 backdrop-blur-sm rounded-lg border border-[#9e8cb4]/30 text-[#9e8cb4]">Adaptive</span>
            </div>
          </div>
          <div className="text-[10px] text-[#9e8cb4]/60">
            Rust + WebAssembly
          </div>
        </div>
      </header>

      {/* Main Simulator - Responsive container */}
      <main className="flex-1 min-h-0 p-4 overflow-auto relative z-10">
        <div className="h-full max-w-[2000px] mx-auto">
          <PlayerSimulator />
        </div>
      </main>
    </div>
  );
}

export default App;
