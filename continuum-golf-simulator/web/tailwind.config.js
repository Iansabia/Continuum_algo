/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'golf-green': '#2D5016',
        'golf-gold': '#D4AF37',
        'golf-navy': '#1A1D29',
        // Company brand colors
        'brand-deep-purple': '#493b7c',
        'brand-bright-purple': '#604c9c',
        'brand-tan': '#dfc9ad',
        'brand-dark-gold': '#7e6649',
        'brand-lavender': '#9e8cb4',
        'brand-rose-copper': '#ac7c6c',
      },
      fontFamily: {
        'montserrat': ['Montserrat', 'sans-serif'],
        'inter': ['Inter', 'sans-serif'],
      },
    },
  },
  plugins: [],
}
