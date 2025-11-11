# Deploying Continuum Golf Simulator to Vercel

## Quick Deploy (Recommended)

### Option 1: Deploy via Vercel CLI (Fastest)

1. Install Vercel CLI globally:
```bash
npm install -g vercel
```

2. Navigate to the web directory:
```bash
cd /Users/iansabia/projects/OOCProjects/Continuum_algo/continuum-golf-simulator/web
```

3. Deploy:
```bash
vercel
```

Follow the prompts:
- Set up and deploy? **Y**
- Which scope? (select your account)
- Link to existing project? **N**
- What's your project's name? **continuum-golf-simulator**
- In which directory is your code located? **./**
- Want to override settings? **N**

4. For production deployment:
```bash
vercel --prod
```

### Option 2: Deploy via Vercel Dashboard

1. Go to [vercel.com](https://vercel.com)
2. Sign up/Login with GitHub
3. Click "Add New Project"
4. Import your Git repository OR:
   - Select "Import Third-Party Git Repository"
   - Connect your GitHub repo

5. Configure:
   - **Framework Preset**: Vite
   - **Root Directory**: `continuum-golf-simulator/web`
   - **Build Command**: `npm run build`
   - **Output Directory**: `dist`
   - **Install Command**: `npm install`

6. Click "Deploy"

## Environment Variables (if needed)

If you need any environment variables, add them in:
- Vercel Dashboard → Project Settings → Environment Variables

## Custom Domain

1. Go to Project Settings → Domains
2. Add your custom domain (e.g., `simulator.continuum.golf`)
3. Follow DNS configuration instructions

## Important Notes

- The `vercel.json` file is already configured with necessary WASM headers
- Build time: ~2-3 minutes
- Your app will be live at: `https://[project-name].vercel.app`
- Automatic deployments on every git push (if connected to GitHub)

## Post-Deployment

Your simulator will be available at:
- **Preview URL**: `https://continuum-golf-simulator-[hash].vercel.app`
- **Production URL**: `https://continuum-golf-simulator.vercel.app`

Share this link with YC and investors!

## Troubleshooting

If build fails:
1. Check that all dependencies are in `package.json`
2. Ensure WASM files are built: `npm run build`
3. Check Vercel build logs for specific errors

## Analytics

Vercel provides built-in analytics:
- Go to your project dashboard
- Click "Analytics" tab
- See visitor data, page views, etc.
