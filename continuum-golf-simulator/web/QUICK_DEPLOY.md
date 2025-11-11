# 🚀 Quick Deploy to Vercel - 5 Minutes

## Fastest Way: Vercel CLI

### Step 1: Install Vercel CLI
```bash
npm install -g vercel
```

### Step 2: Go to Web Directory
```bash
cd /Users/iansabia/projects/OOCProjects/Continuum_algo/continuum-golf-simulator/web
```

### Step 3: Deploy!
```bash
vercel
```

Answer the prompts:
- **Set up and deploy?** → `Y`
- **Which scope?** → (Choose your account)
- **Link to existing project?** → `N`
- **Project name?** → `continuum-golf-simulator` (or any name)
- **Directory?** → `./` (press Enter)
- **Override settings?** → `N`

### Step 4: Go to Production
```bash
vercel --prod
```

**Done!** 🎉 You'll get a URL like: `https://continuum-golf-simulator.vercel.app`

---

## Alternative: Vercel Dashboard (No CLI)

### Option A: If you have GitHub repo

1. Go to [vercel.com/new](https://vercel.com/new)
2. Import your GitHub repository
3. Set **Root Directory** to: `continuum-golf-simulator/web`
4. Click **Deploy**

### Option B: No GitHub (Drag & Drop)

1. Zip the `web` folder:
```bash
cd /Users/iansabia/projects/OOCProjects/Continuum_algo/continuum-golf-simulator
zip -r continuum-web.zip web/
```

2. Go to [vercel.com/new](https://vercel.com/new)
3. Click "Deploy without Git Provider"
4. Drag and drop the `continuum-web.zip`
5. Click **Deploy**

---

## What You Get

✅ Live URL for YC/investors
✅ Automatic HTTPS
✅ Global CDN (fast worldwide)
✅ Analytics dashboard
✅ Free tier (plenty for demos)

## Custom Domain (Optional)

In Vercel Dashboard:
1. Go to Project Settings → Domains
2. Add: `simulator.continuum.golf` (or your domain)
3. Update DNS records as shown

---

## Troubleshooting

**Build fails?**
```bash
cd web
npm run build
```
Check for errors, then redeploy.

**WASM not loading?**
- Already configured in `vercel.json` ✅
- CORS headers set automatically ✅

**Need help?**
Check Vercel logs: Project → Deployments → Click deployment → View logs
