export default function Hero() {
  return (
    <div className="relative overflow-hidden">
      {/* Background Animation */}
      <div className="absolute inset-0 bg-gradient-to-r from-brand-bright-purple/20 to-brand-rose-copper/20 animate-pulse"></div>

      <div className="relative container mx-auto px-4 py-20 text-center">
        {/* Logo/Title */}
        <h1 className="text-6xl md:text-7xl font-montserrat font-bold text-brand-tan mb-4">
          CONTINUUM GOLF
        </h1>

        {/* Tagline */}
        <p className="text-2xl md:text-3xl text-[var(--brand-lavender)] mb-8">
          Fair • Adaptive • Intelligent
        </p>

        <p className="text-xl text-[var(--brand-tan)] max-w-3xl mx-auto mb-12">
          Experience the future of golf wagering with real-time skill adaptation and guaranteed fairness
        </p>

        {/* Key Stats */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-4xl mx-auto">
          <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl p-6 border border-[var(--brand-bright-purple)]/40">
            <div className="text-4xl font-bold text-[var(--brand-tan)] mb-2">85%</div>
            <div className="text-[var(--brand-lavender)]">Return to Player</div>
          </div>
          <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl p-6 border border-[var(--brand-bright-purple)]/40">
            <div className="text-4xl font-bold text-[var(--brand-tan)] mb-2">100%</div>
            <div className="text-[var(--brand-lavender)]">Fairness Guarantee</div>
          </div>
          <div className="bg-[var(--brand-deep-purple)]/20 backdrop-blur-xl rounded-2xl p-6 border border-[var(--brand-bright-purple)]/40">
            <div className="text-4xl font-bold text-[var(--brand-tan)] mb-2">Real-time</div>
            <div className="text-[var(--brand-lavender)]">Kalman Adaptation</div>
          </div>
        </div>
      </div>
    </div>
  );
}
