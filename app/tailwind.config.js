/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,jsx}'],
  theme: {
    extend: {
      colors: {
        ink: Object.fromEntries(['950', '900', '850', '800', '700', '600', '500', '400', '300', '200', '100'].map(k => [k, `rgb(var(--ink-${k}) / <alpha-value>)`])),
        white: 'rgb(var(--fg) / <alpha-value>)',
        paper: '#ffffff',
      },
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', '"SF Pro Text"', 'Inter', '"Segoe UI"', 'Roboto', 'sans-serif'],
        mono: ['"SF Mono"', 'ui-monospace', 'Menlo', 'monospace'],
        display: ['"Young Serif"', 'Georgia', '"Times New Roman"', 'serif'],
      },
      boxShadow: {
        card: '0 1px 2px rgba(0,0,0,.35), 0 4px 12px rgba(0,0,0,.25)',
        lift: '0 8px 30px rgba(0,0,0,.35)',
      },
    },
  },
  plugins: [],
};
