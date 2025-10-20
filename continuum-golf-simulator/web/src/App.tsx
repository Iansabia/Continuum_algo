import { useState } from 'react';
import Hero from './components/Hero';
import PlayerSimulator from './components/PlayerSimulator';
import VenueSimulator from './components/VenueSimulator';
import FairnessValidator from './components/FairnessValidator';

function App() {
  const [activeTab, setActiveTab] = useState<'player' | 'venue' | 'fairness'>('player');

  return (
    <div className="min-h-screen bg-gradient-to-b from-golf-navy via-gray-900 to-black">
      <Hero />

      <div className="container mx-auto px-4 py-8">
        {/* Navigation Tabs */}
        <div className="flex justify-center gap-4 mb-8">
          <button
            onClick={() => setActiveTab('player')}
            className={`px-6 py-3 rounded-lg font-montserrat font-semibold transition-all ${
              activeTab === 'player'
                ? 'bg-golf-gold text-golf-navy shadow-lg'
                : 'bg-gray-800 text-gray-300 hover:bg-gray-700'
            }`}
          >
            Player Session
          </button>
          <button
            onClick={() => setActiveTab('venue')}
            className={`px-6 py-3 rounded-lg font-montserrat font-semibold transition-all ${
              activeTab === 'venue'
                ? 'bg-golf-gold text-golf-navy shadow-lg'
                : 'bg-gray-800 text-gray-300 hover:bg-gray-700'
            }`}
          >
            Venue Economics
          </button>
          <button
            onClick={() => setActiveTab('fairness')}
            className={`px-6 py-3 rounded-lg font-montserrat font-semibold transition-all ${
              activeTab === 'fairness'
                ? 'bg-golf-gold text-golf-navy shadow-lg'
                : 'bg-gray-800 text-gray-300 hover:bg-gray-700'
            }`}
          >
            Fairness Proof
          </button>
        </div>

        {/* Active Component */}
        {activeTab === 'player' && <PlayerSimulator />}
        {activeTab === 'venue' && <VenueSimulator />}
        {activeTab === 'fairness' && <FairnessValidator />}
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
