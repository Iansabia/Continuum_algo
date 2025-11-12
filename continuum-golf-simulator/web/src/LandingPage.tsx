import { MeshGradient } from '@paper-design/shaders-react'
import { Button3D } from '@/components/ui/3d-button'
import { motion, useInView } from 'framer-motion'
import { useRef } from 'react'
import { Link } from 'react-scroll'

// Scroll animation wrapper component
function ScrollReveal({ children, delay = 0 }: { children: React.ReactNode; delay?: number }) {
  const ref = useRef(null)
  const isInView = useInView(ref, { once: true, amount: 0.3 })

  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 50 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 50 }}
      transition={{ duration: 0.6, delay }}
    >
      {children}
    </motion.div>
  )
}

export default function LandingPage() {
  return (
    <div className="relative">
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

      {/* Hero Section */}
      <section className="relative min-h-screen flex flex-col justify-center items-center px-10 py-32">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
          className="text-6xl font-black tracking-tighter mb-6 text-center text-black dark:text-white"
        >
          CONTINUUM
        </motion.div>
        <motion.h1
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.2 }}
          className="text-3xl font-light text-center max-w-3xl mb-5 leading-snug text-black dark:text-white"
        >
          The First Skill-Based Wagering Platform for Real-World Golf
        </motion.h1>
        <motion.p
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.4 }}
          className="text-lg text-center max-w-2xl mb-12 leading-relaxed text-black dark:text-white"
        >
          Continuum Technologies is building the software platform that transforms any golf simulator into a competitive,
          real-money gameplay experience. Integrated with Trackman, Foresight, and Uneekor systems, we're creating an entirely new market
          at the intersection of gaming, golf, and sports wagering.
        </motion.p>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.6 }}
          className="flex gap-5 mb-16 flex-wrap justify-center"
        >
          <Button3D href="/app.html" variant="secondary">
            Try Live Demo
          </Button3D>
          <Link to="how-it-works" smooth={true} duration={800} offset={-50}>
            <Button3D variant="secondary">
              Learn More
            </Button3D>
          </Link>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.8 }}
          className="flex gap-16 flex-wrap justify-center"
        >
          <div className="text-center">
            <div className="text-5xl font-black leading-none mb-2 text-black dark:text-white">
              $115B+
            </div>
            <div className="text-sm uppercase tracking-wider font-semibold text-black dark:text-white">
              Sports Betting Market
            </div>
          </div>
          <div className="text-center">
            <div className="text-5xl font-black leading-none mb-2 text-black dark:text-white">
              20M+
            </div>
            <div className="text-sm uppercase tracking-wider font-semibold text-black dark:text-white">
              Golfers in US
            </div>
          </div>
          <div className="text-center">
            <div className="text-5xl font-black leading-none mb-2 text-black dark:text-white">
              New
            </div>
            <div className="text-sm uppercase tracking-wider font-semibold text-black dark:text-white">
              Market Category
            </div>
          </div>
        </motion.div>
      </section>

      {/* Features Section */}
      <section className="relative px-10 py-32">
        <ScrollReveal>
          <h2 className="text-5xl font-extrabold text-center mb-5 tracking-tight text-black dark:text-white">
            What We're Building
          </h2>
          <p className="text-lg text-center max-w-2xl mx-auto mb-20 leading-relaxed text-black dark:text-white">
            The first-ever platform to unlock skill-based sports wagering for a real-world, physical sport with measurable outcomes.
          </p>
        </ScrollReveal>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-10 max-w-6xl mx-auto">
          {[
            {
              title: 'Mobile App for Players',
              description: 'Enter challenges, track performance, and compete for real money. Personalized payout curves adapt to each golfer\'s accuracy and consistency—fair for every skill level.'
            },
            {
              title: 'Simulator Integration',
              description: 'Direct integration with Trackman, Foresight, and Uneekor. Real-time shot data feeds into our backend to generate instant, skill-based payouts.'
            },
            {
              title: 'Venue Backend Portal',
              description: 'Operators can configure contests, monitor gameplay, and track revenue in real-time. Turn every bay into a competitive gaming experience.'
            },
            {
              title: 'Dynamic & Fair',
              description: 'Every player gets a personalized experience. Better golfers see higher payouts, while beginners get accessible entry points. It\'s skill-based, not luck-based.'
            },
            {
              title: 'New Revenue Streams',
              description: 'Drive engagement and generate income for simulator venues. Players stay longer, compete more, and return for the competitive experience.'
            },
            {
              title: 'Creating a New Market',
              description: 'Not just riding the sports betting wave—we\'re opening up an entirely new category at the intersection of golf, gaming, and immersive tech.'
            }
          ].map((feature, i) => (
            <ScrollReveal key={i} delay={i * 0.1}>
              <div className="bg-gradient-to-br from-white/40 to-white/20 backdrop-blur-2xl border border-white/30 rounded-3xl p-10 hover:border-white/50 hover:bg-white/30 transition-all h-full flex flex-col">
                <h3 className="text-2xl font-bold mb-3 text-black dark:text-white">{feature.title}</h3>
                <p className="text-base leading-relaxed text-black dark:text-white flex-grow">{feature.description}</p>
              </div>
            </ScrollReveal>
          ))}
        </div>
      </section>

      {/* How It Works */}
      <section className="relative px-10 py-32" id="how-it-works">
        <ScrollReveal>
          <h2 className="text-5xl font-extrabold text-center mb-5 tracking-tight text-black dark:text-white">
            How It Works
          </h2>
          <p className="text-lg text-center max-w-2xl mx-auto mb-20 leading-relaxed text-black dark:text-white">
            From simulator to payout in seconds—here's the player experience.
          </p>
        </ScrollReveal>

        <div className="max-w-4xl mx-auto">
          {[
            {
              num: 1,
              title: 'Open the App & Enter a Challenge',
              description: 'Players open our mobile app, select a challenge, and place their wager. They can compete solo or against others in multiplayer formats.'
            },
            {
              num: 2,
              title: 'Take Your Shot',
              description: 'Hit the ball on a Trackman, Foresight, or Uneekor simulator. Our platform captures real-time shot data—distance, accuracy, and consistency.'
            },
            {
              num: 3,
              title: 'Skill-Based Payout Generated',
              description: 'Our backend instantly calculates a personalized payout based on shot accuracy relative to your skill profile. Fair, dynamic, and tailored to you.'
            },
            {
              num: 4,
              title: 'Win & Track Your Progress',
              description: 'Receive your payout immediately. Track your performance over time, compare with others, and climb the leaderboard.'
            }
          ].map((step, i) => (
            <ScrollReveal key={i} delay={i * 0.15}>
              <div className="bg-gradient-to-br from-white/40 to-white/20 backdrop-blur-2xl border border-white/30 rounded-3xl p-10 mb-10 hover:border-white/50 hover:bg-white/30 transition-all">
                <div className="flex items-start gap-6">
                  <div className="min-w-[60px] h-16 flex items-center justify-center text-3xl font-black text-black dark:text-white">
                    {step.num}
                  </div>
                  <div className="flex-1">
                    <h3 className="text-3xl font-bold mb-3 text-black dark:text-white">{step.title}</h3>
                    <p className="text-base leading-relaxed text-black dark:text-white">{step.description}</p>
                  </div>
                </div>
              </div>
            </ScrollReveal>
          ))}
        </div>
      </section>

      {/* Technology Section */}
      <section className="relative px-10 py-32">
        <ScrollReveal>
          <h2 className="text-5xl font-extrabold text-center mb-5 tracking-tight text-black dark:text-white">
            Why This Is Unique
          </h2>
          <p className="text-lg text-center max-w-2xl mx-auto mb-20 leading-relaxed text-black dark:text-white">
            This is the first time skill-based sports wagering has been unlocked for a real-world, physical sport with measurable outcomes.
          </p>
        </ScrollReveal>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8 max-w-6xl mx-auto">
          {[
            { label: 'Market', value: 'First-of-Its-Kind' },
            { label: 'Integration', value: 'Major Simulators' },
            { label: 'Payouts', value: 'Personalized & Fair' },
            { label: 'Revenue', value: 'New Streams for Venues' }
          ].map((tech, i) => (
            <ScrollReveal key={i} delay={i * 0.1}>
              <div className="bg-gradient-to-br from-white/40 to-white/20 backdrop-blur-2xl border border-white/30 rounded-2xl p-8 text-center hover:border-white/50 hover:bg-white/30 transition-all h-full flex flex-col justify-center min-h-[140px]">
                <div className="text-sm uppercase tracking-wider mb-2 font-semibold text-black dark:text-white">
                  {tech.label}
                </div>
                <div className="text-2xl font-extrabold text-black dark:text-white">{tech.value}</div>
              </div>
            </ScrollReveal>
          ))}
        </div>
      </section>

      {/* CTA Section */}
      <section className="relative px-10 py-32 text-center">
        <ScrollReveal>
          <h2 className="text-6xl font-black mb-6 tracking-tighter text-black dark:text-white">
            See the Platform in Action
          </h2>
          <p className="text-xl mb-10 max-w-2xl mx-auto text-black dark:text-white">
            Explore our interactive demo showcasing real-time gameplay simulation, venue analytics, and dynamic payout calculations.
          </p>
          <div className="flex gap-5 justify-center">
            <Button3D href="/app.html" variant="secondary">
              Launch Interactive Demo
            </Button3D>
          </div>
        </ScrollReveal>
      </section>

      {/* Footer */}
      <footer className="relative px-10 py-16 text-center">
        <div className="text-3xl font-black mb-5 text-black dark:text-white">
          CONTINUUM
        </div>
        <p className="text-sm mb-3 text-black dark:text-white">
          Creating a New Market at the Intersection of Golf, Gaming, and Sports Wagering
        </p>
        <p className="text-sm mb-3 text-black dark:text-white">Continuum Technologies</p>
        <p className="text-xs mt-5 text-black/60 dark:text-white/60">
          &copy; 2024 Continuum Technologies. All rights reserved.
        </p>
      </footer>
    </div>
  )
}
