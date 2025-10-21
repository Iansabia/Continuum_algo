import Hero from './components/Hero';
import PlayerSimulator from './components/PlayerSimulator';

function App() {
  return (
    <div className="min-h-screen bg-gradient-to-b from-brand-deep-purple via-gray-900 to-black">
      <Hero />

      <div className="container mx-auto px-4 py-8">
        <PlayerSimulator />
      </div>

      {/* Footer */}
      <footer className="py-8 text-center text-gray-500">
        <p>&copy; 2025 Continuum Golf. All rights reserved.</p>
        <p className="text-sm mt-2">Powered by Rust + WebAssembly</p>
      </footer>
    </div>
  );
}

export default App;
