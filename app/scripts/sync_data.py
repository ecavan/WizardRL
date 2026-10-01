"""Copy the models and the charts the app needs into public/ (run after retraining or
regenerating the charts):

    npm run data          # from app/

- public/models/ppo5.wznet: the best bot (a two-brain file: ppo5 chooses, arche2 scores)
- public/models/winprob.wzwp: the win-probability model
- public/data/*.json: the bid chart, play chart, example decisions and mistake chart for Learn
  (the mistake chart trimmed to what the page shows)
"""

import json
import os
import shutil

HERE = os.path.dirname(os.path.abspath(__file__))
APP = os.path.dirname(HERE)
RL = os.path.join(os.path.dirname(APP), "rl")

for name in ("ppo5.wznet", "winprob.wzwp"):
    shutil.copy(os.path.join(RL, "models", name), os.path.join(APP, "public", "models", name))

charts = os.path.join(RL, "charts")
out = os.path.join(APP, "public", "data")
os.makedirs(out, exist_ok=True)
for name in ("bid_chart.json", "play_chart.json", "play_situations.json"):
    shutil.copy(os.path.join(charts, name), os.path.join(out, name))

with open(os.path.join(charts, "mistake_chart.json")) as f:
    m = json.load(f)
for t in m["tables"].values():
    rows = [r for r in t["mistakes"] if r["count"] >= 200]
    keep = set()
    for kind in ("bid", "play"):
        mine = [i for i, r in enumerate(rows) if r["kind"] == kind]
        keep.update(sorted(mine, key=lambda i: -rows[i]["per_game"])[:60])
        keep.update(sorted(mine, key=lambda i: -rows[i]["points"])[:60])
    t["mistakes"] = [rows[i] for i in sorted(keep)]
with open(os.path.join(out, "mistake_chart.json"), "w") as f:
    json.dump(m, f, separators=(",", ":"))
print("copied models and chart data into public/")
