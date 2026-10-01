# Wizard app

Learn, Watch and Play, with the Wizard RL bots running in the browser. See
[The app](../README.md#the-app) in the main README.

```sh
npm install
npm run dev      # http://localhost:5173
npm run build    # static site in dist/
npm run wasm     # rebuild src/wasm after changing the Rust engine
npm run data     # copy models and chart data from ../rl into public/
```

```
src/App.jsx        header, tabs, settings
src/modes/         Learn, Watch, Play
src/ui/            cards, the table, small controls
src/lib/engine.js  loads the WebAssembly engine and the models
src/lib/players.js the bots: levels, habits, random tables
src/wasm/          built engine (from crates/wizard-wasm; committed)
public/models/     ppo5.wznet (the bots) and winprob.wzwp (chance to win)
public/data/       chart data for Learn
```

Deploy on Vercel with **Root Directory** set to `app`.
