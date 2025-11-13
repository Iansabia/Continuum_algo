import { motion } from 'framer-motion'

export function PhoneMockApp() {
  return (
    <div className="relative w-full h-full flex items-center justify-center">
      {/* 3D Phone Frame */}
      <motion.div
        initial={{ opacity: 0, rotateY: -20 }}
        animate={{ opacity: 1, rotateY: 0 }}
        transition={{ duration: 0.8, ease: "easeOut" }}
        className="relative"
        style={{
          perspective: '1000px',
          transformStyle: 'preserve-3d',
        }}
      >
        {/* Phone Device */}
        <div
          className="relative w-[320px] h-[640px] rounded-[40px] overflow-hidden"
          style={{
            background: `
              linear-gradient(135deg,
                #fcfcfd 0%,
                #f8f8fa 15%,
                #f3f4f6 30%,
                #eeeff2 45%,
                #e9eaed 60%,
                #e4e5e8 75%,
                #dee0e3 90%,
                #e2e3e6 100%
              )
            `,
            boxShadow: `
              0 10px 30px rgba(0, 0, 0, 0.3),
              0 20px 60px rgba(0, 0, 0, 0.2),
              0 30px 80px rgba(0, 0, 0, 0.15),
              inset 0 2px 4px rgba(255, 255, 255, 0.8),
              inset 0 -4px 12px rgba(0, 0, 0, 0.15),
              inset 4px 4px 12px rgba(0, 0, 0, 0.12),
              inset -4px 4px 12px rgba(0, 0, 0, 0.10)
            `,
            transform: 'rotateY(-5deg) rotateX(5deg)',
          }}
        >
          {/* Top edge highlight */}
          <div
            className="absolute inset-x-0 top-0 pointer-events-none"
            style={{
              height: '3px',
              background: 'linear-gradient(90deg, rgba(255, 255, 255, 0) 0%, rgba(255, 255, 255, 0.95) 5%, rgba(255, 255, 255, 1) 50%, rgba(255, 255, 255, 0.95) 95%, rgba(255, 255, 255, 0) 100%)',
              filter: 'blur(0.5px)',
            }}
          />

          {/* Phone notch */}
          <div className="absolute top-0 left-1/2 transform -translate-x-1/2 w-32 h-6 bg-black/90 rounded-b-3xl z-20"></div>

          {/* Screen Content */}
          <div className="absolute inset-4 top-8 bottom-4 rounded-[32px] overflow-hidden bg-gradient-to-br from-purple-900 via-purple-800 to-indigo-900">
            {/* Gradient Background matching website */}
            <div className="absolute inset-0 bg-gradient-to-br from-[#493b7c] via-[#604c9c] to-[#9e8cb4] opacity-90"></div>

            {/* App Content */}
            <div className="relative z-10 p-6 h-full flex flex-col">
              {/* Status Bar */}
              <div className="flex justify-between items-center text-white text-xs mb-6">
                <span>9:41</span>
                <div className="flex gap-1">
                  <div className="w-4 h-3 border border-white rounded-sm"></div>
                  <div className="w-4 h-3 border border-white rounded-sm"></div>
                  <div className="w-4 h-3 border border-white rounded-sm bg-white"></div>
                </div>
              </div>

              {/* App Header */}
              <div className="mb-8">
                <h1 className="text-white text-2xl font-bold mb-2">Continuum Golf</h1>
                <p className="text-white/80 text-sm">Skill-Based Wagering</p>
              </div>

              {/* Featured Challenge Card */}
              <motion.div
                initial={{ y: 20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.3 }}
                className="bg-gradient-to-br from-white/40 to-white/20 backdrop-blur-xl border border-white/30 rounded-2xl p-4 mb-4"
              >
                <div className="flex justify-between items-start mb-3">
                  <div>
                    <h3 className="text-white font-bold text-lg">Target Challenge</h3>
                    <p className="text-white/70 text-xs">150 yards precision</p>
                  </div>
                  <div className="bg-white/30 rounded-lg px-3 py-1">
                    <span className="text-white text-sm font-bold">$50</span>
                  </div>
                </div>
                <div className="flex gap-2">
                  <div className="flex-1 bg-white/20 rounded-lg p-2 text-center">
                    <p className="text-white/60 text-xs">Shots</p>
                    <p className="text-white font-bold">5</p>
                  </div>
                  <div className="flex-1 bg-white/20 rounded-lg p-2 text-center">
                    <p className="text-white/60 text-xs">Payout</p>
                    <p className="text-white font-bold">$125</p>
                  </div>
                </div>
              </motion.div>

              {/* Stats Cards */}
              <div className="grid grid-cols-2 gap-3 mb-4">
                <motion.div
                  initial={{ x: -20, opacity: 0 }}
                  animate={{ x: 0, opacity: 1 }}
                  transition={{ delay: 0.4 }}
                  className="bg-white/20 backdrop-blur-lg border border-white/20 rounded-xl p-3"
                >
                  <p className="text-white/60 text-xs mb-1">Win Rate</p>
                  <p className="text-white text-xl font-bold">68%</p>
                </motion.div>
                <motion.div
                  initial={{ x: 20, opacity: 0 }}
                  animate={{ x: 0, opacity: 1 }}
                  transition={{ delay: 0.4 }}
                  className="bg-white/20 backdrop-blur-lg border border-white/20 rounded-xl p-3"
                >
                  <p className="text-white/60 text-xs mb-1">Earnings</p>
                  <p className="text-white text-xl font-bold">$2,340</p>
                </motion.div>
              </div>

              {/* Action Button */}
              <motion.button
                initial={{ y: 20, opacity: 0 }}
                animate={{ y: 0, opacity: 1 }}
                transition={{ delay: 0.5 }}
                className="w-full py-4 rounded-xl font-bold text-black mt-auto"
                style={{
                  background: `
                    linear-gradient(135deg,
                      #fcfcfd 0%,
                      #f8f8fa 15%,
                      #f3f4f6 30%,
                      #eeeff2 45%,
                      #e9eaed 60%,
                      #e4e5e8 75%,
                      #dee0e3 90%,
                      #e2e3e6 100%
                    )
                  `,
                  boxShadow: `
                    0 4px 8px rgba(0, 0, 0, 0.2),
                    inset 0 1px 2px rgba(255, 255, 255, 0.8),
                    inset 0 -2px 4px rgba(0, 0, 0, 0.1)
                  `,
                }}
              >
                Enter Challenge
              </motion.button>
            </div>
          </div>

          {/* Screen gloss */}
          <div
            className="absolute inset-4 top-8 bottom-4 rounded-[32px] pointer-events-none"
            style={{
              background: 'linear-gradient(180deg, rgba(255, 255, 255, 0.15) 0%, rgba(255, 255, 255, 0.05) 30%, rgba(255, 255, 255, 0) 50%)',
            }}
          ></div>

          {/* Side buttons */}
          <div className="absolute right-0 top-24 w-1 h-12 bg-white/30 rounded-l-sm"></div>
          <div className="absolute right-0 top-40 w-1 h-16 bg-white/30 rounded-l-sm"></div>
          <div className="absolute left-0 top-32 w-1 h-12 bg-white/30 rounded-r-sm"></div>
        </div>

        {/* Phone shadow */}
        <div
          className="absolute inset-0 -z-10 rounded-[40px]"
          style={{
            filter: 'blur(40px)',
            background: 'rgba(0, 0, 0, 0.5)',
            transform: 'translateY(20px) scale(0.95)',
          }}
        ></div>
      </motion.div>
    </div>
  )
}
