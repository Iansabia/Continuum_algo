export default function Hero() {
  return (
    <div className="relative overflow-hidden">
      {/* Background Animation */}
      <div className="absolute inset-0 bg-gradient-to-r from-golf-green/20 to-golf-gold/20 animate-pulse"></div>

      <div className="relative container mx-auto px-4 py-20 text-center">
        {/* Logo/Title */}
        <h1 className="text-6xl md:text-7xl font-montserrat font-bold text-golf-gold mb-4">
          CONTINUUM GOLF
        </h1>

        {/* Tagline */}
        <p className="text-2xl md:text-3xl text-gray-300 mb-8">
          Fair • Dynamic • Profitable
        </p>

        <p className="text-xl text-gray-400 max-w-3xl mx-auto mb-12">
          Golf wagering reimagined with adaptive AI, guaranteed fairness, and proven profitability
        </p>

        {/* Key Stats */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-4xl mx-auto">
          <div className="bg-gray-800/50 backdrop-blur-sm rounded-lg p-6 border border-golf-gold/20">
            <div className="text-4xl font-bold text-golf-gold mb-2">85%</div>
            <div className="text-gray-400">Return to Player</div>
          </div>
          <div className="bg-gray-800/50 backdrop-blur-sm rounded-lg p-6 border border-golf-gold/20">
            <div className="text-4xl font-bold text-golf-gold mb-2">100%</div>
            <div className="text-gray-400">Fairness Guarantee</div>
          </div>
          <div className="bg-gray-800/50 backdrop-blur-sm rounded-lg p-6 border border-golf-gold/20">
            <div className="text-4xl font-bold text-golf-gold mb-2">Real-time</div>
            <div className="text-gray-400">Kalman Adaptation</div>
          </div>
        </div>
      </div>
    </div>
  );
}
