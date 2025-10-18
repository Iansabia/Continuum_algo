# Continuum Golf Simulator - Web Interface Plan

## 🎯 Goal
Create an **investor-ready web demo** showcasing the Continuum Golf simulator with interactive visualizations, deployable to Vercel or GitHub Pages.

## 🏗️ Architecture Overview

### Technology Stack
- **Backend**: Rust compiled to WebAssembly (WASM)
- **Frontend**: React + TypeScript + Vite
- **Styling**: Tailwind CSS
- **Charts**: Chart.js or Recharts
- **3D Graphics**: Three.js (optional, for golf ball animation)
- **Deployment**: Vercel (primary) or GitHub Pages

### Why This Stack?

**WebAssembly (WASM)**
- ✅ Rust simulator runs **directly in browser** (no backend server!)
- ✅ Near-native performance (10-100× faster than JavaScript)
- ✅ Secure: sandboxed execution
- ✅ Zero hosting costs (static site)

**React + TypeScript**
- ✅ Industry standard (investors recognize it)
- ✅ Rich ecosystem for charts, animations, UI components
- ✅ Type safety prevents bugs
- ✅ Easy to maintain and extend

**Vercel Deployment**
- ✅ One-click deploy from GitHub
- ✅ Automatic HTTPS, CDN, caching
- ✅ Custom domains
- ✅ Preview deployments for testing

---

## 📱 User Interface Design

### Landing Page

```
┌─────────────────────────────────────────────────────────┐
│                    CONTINUUM GOLF                       │
│       Fair, Dynamic, Profitable - Golf Reimagined       │
│                                                          │
│   [Animated Golf Ball Trajectory - 3D Canvas]          │
│                                                          │
│   ┌─────────┐  ┌─────────┐  ┌─────────┐               │
│   │   85%   │  │ Perfect  │  │ Kalman  │               │
│   │   RTP   │  │ Fairness │  │ Adaptive│               │
│   │  (15%   │  │  (Equal  │  │  (Bias  │               │
│   │  Edge)  │  │   EV)    │  │  Track) │               │
│   └─────────┘  └─────────┘  └─────────┘               │
│                                                          │
│            [Try Live Demo →] [Watch Video]              │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Interactive Dashboard

**3 Main Tabs:**

#### 1. Player Simulator
```
┌──────────────────────────────────────────────────────────┐
│  Controls                      │  Live Visualization     │
├────────────────────────────────┼─────────────────────────┤
│ Handicap: [====●====] 15       │                         │
│ Shots: [===●=======] 100       │  [P/L Line Chart]       │
│ Wager: $5 - $10                │  Running: -$45.20       │
│ Hole: [H4 - 150yds ▼]         │                         │
│                                 │  [Skill Confidence]     │
│ [▶ Start] [⏸ Pause] [↻ Reset] │  ████████░░ 82%        │
│                                 │                         │
│ Speed: [1x] [5x] [Max]         │  [Shot Scatter Plot]    │
│                                 │  • Hit within radius    │
└────────────────────────────────┴─────────────────────────┘
```

#### 2. Venue Economics
```
┌──────────────────────────────────────────────────────────┐
│  Venue Configuration           │  Financial Dashboard    │
├────────────────────────────────┼─────────────────────────┤
│ Bays: [=====●====] 50          │                         │
│ Hours: [====●=====] 8          │  Revenue: $128,450      │
│ Shots/hr: [===●====] 100       │  Payouts: $106,100      │
│ Archetype: [Bell Curve ▼]     │  Profit:  $22,350       │
│                                 │                         │
│ [▶ Run Simulation]             │  Hold %: [███] 12.4%    │
│                                 │                         │
│                                 │  [Hourly Revenue Bar]   │
│                                 │  [Player Distribution]  │
└────────────────────────────────┴─────────────────────────┘
```

#### 3. Fairness Validator
```
┌──────────────────────────────────────────────────────────┐
│  Interactive Fairness Proof                              │
├──────────────────────────────────────────────────────────┤
│  Selected Hole: [H4 - 150 yards ▼]                      │
│                                                           │
│  Running 10,000 shots for each handicap level...         │
│                                                           │
│  Handicap 0:   EV = -15.02% ✓                           │
│  Handicap 10:  EV = -15.00% ✓                           │
│  Handicap 20:  EV = -14.98% ✓                           │
│  Handicap 30:  EV = -15.01% ✓                           │
│                                                           │
│  ✅ All handicaps within ±0.5% (15% house edge)         │
│  ✅ FAIRNESS PROVEN: Equal opportunity regardless of    │
│      skill level - everyone faces same 15% edge         │
│                                                           │
│  [Animated Visualization: Equal Opportunity Circle]      │
└──────────────────────────────────────────────────────────┘
```

---

## 📊 Advanced Visualizations

### 1. Shot Trajectory Viewer (Enhanced with BVN)
- **Type**: 2D scatter plot with bias overlay
- **X-axis**: Lateral position (feet from pin, negative = left, positive = right)
- **Y-axis**: Distance position (feet from pin, negative = short, positive = long)
- **Color**: Payout multiplier (green = high, red = low)
- **NEW: Bias Vector**: Arrow showing systematic miss tendency (μ_x, μ_y)
- **NEW: Elliptical Confidence**: 1σ and 2σ ellipses showing dispersion (σ_x vs σ_y)
- **Interactive**: Hover for details, click to highlight shot
- **Animation**: Shots appear one-by-one with fade-in, bias updates in real-time

### 2. Kalman Filter Evolution (Enhanced for 4D)
- **Type**: Multi-line time series chart
- **Legacy (1D)**: Single line showing σ over time
- **NEW (4D)**:
  - **Line 1**: μ_x (lateral bias) - Red line
  - **Line 2**: μ_y (distance bias) - Blue line
  - **Line 3**: σ_x (lateral precision) - Orange area
  - **Line 4**: σ_y (distance precision) - Green area
- **Shaded areas**: Confidence bands for each parameter
- **Annotations**: Key events (batch updates, high-stakes shots)
- **Toggle**: Switch between 1D (legacy) and 4D (BVN) views
- **Controls**: Scrub through time, zoom in/out

### 2b. Bias Detection Dashboard (NEW)
- **Purpose**: Show systematic shooting tendencies
- **Components**:
  - **Bias Magnitude**: `sqrt(μ_x² + μ_y²)` - How far off-center on average
  - **Bias Direction**: Compass showing direction (e.g., "4 ft right, 2 ft long")
  - **Precision Ratio**: `σ_x / σ_y` - Are you better at distance or lateral control?
  - **Coaching Tip**: "Work on lateral control - you're 2× more precise at distance"
- **Visual**: Polar plot with bias vector and elliptical confidence region

### 3. Profitability Heatmap (Updated for 15% Edge)
- **Rows**: 8 holes (75-250 yds)
- **Columns**: 7 handicap ranges (0-4, 5-9, ..., 25-30)
- **Cell color**: Hold percentage (green gradient, all ~15% theoretical)
- **Hover**: Detailed tooltip (RTP=85%, actual hold %, sample size, variance)
- **Note**: Cells show actual hold (15-17%) due to variance, fat-tails, Kalman learning
- **Insight**: "All holes maintain 15% house edge - uniform fairness across all distances"

### 4. Revenue Projection Calculator
- **Inputs**: Venue size, pricing, location, demographics
- **Output**: 5-year financial model
- **Chart**: Area chart with best/worst/expected scenarios
- **Download**: Export as PDF or Excel

### 5. Monte Carlo Risk Analysis
- **Run**: 1,000 venue simulations with random variations
- **Display**: Histogram of outcomes
- **Highlight**: Percentiles (10th, 50th, 90th)
- **Insight**: "95% chance of $X+ profit"

### 6. Camera System Visualizations (NEW - Phase 9)
- **Purpose**: Show camera integration and (x,y) coordinate capture
- **Components**:
  - **Live Camera Feed**: Real-time view with ball detection overlay
  - **Homography Calibration**: Visual guide for setting up 4 corner markers
  - **Ball ID Recognition**: Show detected ball ID and matched player
  - **Coordinate Accuracy**: Display measured (x,y) vs. expected position
  - **System Health**: Camera uptime, calibration drift, accuracy metrics

### 7. Player Bias Heatmap (NEW - Phase 9)
- **Type**: 2D density plot
- **Shows**: Where player's shots tend to land (aggregated over all shots)
- **Color**: Frequency (red = often misses here, blue = rarely)
- **Overlay**: Pin location, breakeven radius, 1σ/2σ ellipses
- **Use Case**: Identify if player consistently misses in specific direction

---

## 🎨 Design System

### Color Palette
```
Primary:   #2D5016 (Golf Green)
Secondary: #D4AF37 (Gold)
Dark:      #1A1D29 (Navy)
Success:   #10B981 (Emerald)
Warning:   #F59E0B (Amber)
Error:     #EF4444 (Red)
Gray:      #6B7280 (Neutral)
```

### Typography
- **Headings**: Montserrat (Bold, 600-700 weight)
- **Body**: Inter (Regular, 400 weight)
- **Monospace**: JetBrains Mono (for numbers, code)

### Component Library
- **Buttons**: Rounded corners, hover effects, loading states
- **Inputs**: Floating labels, validation feedback
- **Cards**: Subtle shadows, hover lift
- **Charts**: Consistent colors, tooltips, legends
- **Animations**: Smooth transitions (200-300ms), spring physics

---

## 🚀 Implementation Roadmap

### Phase 8.1: WASM Bridge (Week 1)
```bash
# Add dependencies
cargo add wasm-bindgen serde-wasm-bindgen

# Create WASM module
touch src/wasm.rs

# Implement exports
- simulate_player_session()
- simulate_venue()
- validate_fairness()
- get_hole_configs()

# Build
wasm-pack build --target web --out-dir web/wasm
```

### Phase 8.2: React Setup (Week 1)
```bash
# Create frontend
npm create vite@latest web -- --template react-ts
cd web
npm install

# Add dependencies
npm install chart.js react-chartjs-2
npm install @tanstack/react-table
npm install tailwindcss postcss autoprefixer
npm install three @react-three/fiber
```

### Phase 8.3: Core Components (Week 2)
- [ ] SimulatorControls.tsx (sliders, buttons)
- [ ] LiveCharts.tsx (Chart.js integration)
- [ ] ShotScatter.tsx (scatter plot)
- [ ] KalmanChart.tsx (time series)
- [ ] Heatmap.tsx (profitability matrix)
- [ ] MetricsCard.tsx (KPI displays)

### Phase 8.4: WASM Integration (Week 2)
- [ ] useSimulator.ts (React hook)
- [ ] wasmLoader.ts (initialization)
- [ ] dataTransform.ts (JSON ↔ UI state)

### Phase 8.5: Advanced Features (Week 3)
- [ ] Scenario Builder
- [ ] Revenue Calculator
- [ ] Monte Carlo Visualizer
- [ ] Export functionality (PDF, CSV)

### Phase 8.6: Polish & Deploy (Week 3)
- [ ] Responsive design testing
- [ ] Accessibility audit
- [ ] Performance optimization
- [ ] Deploy to Vercel
- [ ] Custom domain setup

---

## 📈 Performance Optimization

### WASM Binary Size
```bash
# Optimize Cargo.toml
[profile.release]
opt-level = "z"           # Optimize for size
lto = true                # Link-time optimization
codegen-units = 1         # Single codegen unit
strip = true              # Strip debug symbols

# Result: ~300-500 KB (gzipped)
```

### Code Splitting
```typescript
// Lazy load heavy components
const MonteCarloViz = lazy(() => import('./MonteCarloViz'));
const ThreeDGolfBall = lazy(() => import('./ThreeDGolfBall'));

// Only load when needed
<Suspense fallback={<Spinner />}>
  <MonteCarloViz />
</Suspense>
```

### Caching Strategy
- WASM binary: `Cache-Control: immutable, max-age=31536000`
- Assets (CSS, JS): Content-hash filenames
- API calls: None (everything runs locally!)

---

## 🔐 Security Considerations

- ✅ No user data stored (runs entirely in browser)
- ✅ No API keys exposed (static site)
- ✅ WASM sandboxed (can't access file system)
- ✅ HTTPS enforced (Vercel default)
- ✅ Content Security Policy headers

---

## 📱 Mobile Experience

### Responsive Breakpoints
- **Desktop**: 1280px+ (full dashboard)
- **Tablet**: 768-1279px (2-column layout)
- **Mobile**: <768px (stacked, simplified controls)

### Mobile-Specific Features
- Touch-friendly sliders
- Bottom sheet for controls
- Swipe between tabs
- Haptic feedback (iOS)
- Install as PWA (Add to Home Screen)

---

## 🎯 Investor Meeting Features

### Presenter Mode
- **Purpose**: Simplify for live demos
- **Features**:
  - Hide complexity toggles
  - Pre-loaded scenarios
  - Large fonts for projectors
  - Keyboard shortcuts (spacebar = run)

### Shareable Links
```
https://continuum-demo.vercel.app/?scenario=venue&bays=50&hours=8
```
- Parameters in URL
- Pre-configured demo
- One-click to reproduce

### Export Capabilities
- **PDF Report**: Full simulation results with charts
- **CSV Data**: Raw data for Excel analysis
- **Embed Code**: Iframe for pitch decks
- **QR Code**: Instant access on tablets

---

## 📊 Success Metrics

### Technical
- [ ] WASM loads in <500ms
- [ ] First Contentful Paint <1.5s
- [ ] Lighthouse score >90
- [ ] 0 accessibility errors
- [ ] Works offline (PWA)

### Business
- [ ] Non-technical person can run demo independently
- [ ] "Wow" factor in first 10 seconds
- [ ] Shareability: used in ≥3 investor meetings
- [ ] Feedback: "This looks professional"

---

## 🚢 Deployment Checklist

### Pre-Launch
- [ ] All simulations tested
- [ ] Mobile responsive verified
- [ ] Cross-browser compatible (Chrome, Safari, Firefox)
- [ ] Performance optimized
- [ ] SEO meta tags added
- [ ] Analytics integrated (Vercel Analytics)

### Launch
- [ ] Deploy to Vercel
- [ ] Custom domain configured
- [ ] SSL certificate active
- [ ] Social share images working
- [ ] README updated with demo link

### Post-Launch
- [ ] Monitor performance metrics
- [ ] Gather investor feedback
- [ ] Iterate on UX based on usage
- [ ] Add "Contact Us" for serious inquiries

---

## 💡 Future Enhancements (Phase 10+)

**Note:** Phase 9 is now dedicated to Camera Integration & BVN Migration (see `continuum_checklist.md`)

- **AI Assistant**: "Ask questions about the simulator"
- **Multi-language**: Spanish, Mandarin for global investors
- **Video Explainers**: Embedded Loom/YouTube tutorials
- **Live Data**: Connect to real venue once operational
- **Comparison Tool**: Side-by-side scenario comparison
- **Advanced Coaching**: AI-powered swing analysis using bias patterns
- **Tournament Brackets**: Visualize tournament gameplay with BVN stats

---

## 📞 Technical Support for Investors

Include in footer:
- GitHub repository link
- Technical white paper (PDF)
- Email: tech@continuum-golf.com
- Schedule demo meeting (Calendly)

---

**Created**: 2025-10-13
**Last Updated**: 2025-10-18
**Status**: Planning Phase (Updated for BVN & 15% house edge)
**Target Completion**: 3 weeks after Phase 6

---

## 📝 Changelog

### 2025-10-18: BVN Integration & House Edge Update
- ✅ Updated landing page to show 85% RTP (15% house edge) instead of variable 86-90%
- ✅ Updated fairness validator to show -15% EV across all handicaps
- ✅ Enhanced Shot Trajectory Viewer with bias vectors and elliptical confidence regions
- ✅ Upgraded Kalman Filter Evolution chart to show 4D state (μ_x, μ_y, σ_x, σ_y)
- ✅ Added Bias Detection Dashboard for systematic tendency analysis
- ✅ Updated Profitability Heatmap to reflect uniform 15% edge
- ✅ Added Camera System Visualizations (live feed, calibration, accuracy)
- ✅ Added Player Bias Heatmap for shot distribution analysis
- ✅ Renamed Future Enhancements to Phase 10+ (Phase 9 = Camera/BVN)

### 2025-10-13: Initial Planning
- Created comprehensive web interface plan for investor showcase
- Defined WASM architecture and React frontend
- Outlined 3 main interactive tabs: Player, Venue, Fairness
- Designed 5 advanced visualizations
- Established deployment strategy (Vercel/GitHub Pages)
