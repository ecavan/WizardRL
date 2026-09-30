"""The bid chart as an interactive page: `bid_chart.html` (made by `bidchart`, or by hand):

    python -m wizard_rl.chartpage charts/bid_chart.json charts/bid_chart.html

Open it in a browser: pick the table size and trump, read the chart, and count a hand.
"""

from __future__ import annotations

import json
import os
import sys

TEMPLATE = os.path.join(os.path.dirname(__file__), "chartpage.html")


def render(json_path: str, full: bool = True) -> str:
    """The page with the chart data inlined. `full=False` leaves out the document skeleton."""
    with open(json_path) as f:
        data = json.load(f)
    # The play chart and the play situations, if they sit next to the bid chart.
    here = os.path.dirname(json_path)
    for key, name in (("play", "play_chart.json"), ("situations", "play_situations.json"),
                      ("mistakes", "mistake_chart.json")):
        path = os.path.join(here, name)
        if os.path.exists(path):
            with open(path) as f:
                data[key] = json.load(f)
    if "mistakes" in data:  # the page shows the top of each list; leave out the long tail
        for t in data["mistakes"]["tables"].values():
            rows = [r for r in t["mistakes"] if r["count"] >= 200]
            keep = set()
            for kind in ("bid", "play"):
                mine = [i for i, r in enumerate(rows) if r["kind"] == kind]
                keep.update(sorted(mine, key=lambda i: -rows[i]["per_game"])[:60])
                keep.update(sorted(mine, key=lambda i: -rows[i]["points"])[:60])
            t["mistakes"] = [rows[i] for i in sorted(keep)]
    with open(TEMPLATE) as f:
        page = f.read()
    page = page.replace("/*DATA*/null", json.dumps(data, separators=(",", ":")))
    if not full:
        return page
    return ('<!doctype html>\n<html lang="en">\n<head>\n<meta charset="utf-8">\n'
            '<meta name="viewport" content="width=device-width, initial-scale=1">\n'
            "<style>body{margin:0}[hidden]{display:none!important}</style>\n</head>\n<body>\n"
            + page + "\n</body>\n</html>\n")


if __name__ == "__main__":
    if len(sys.argv) not in (3, 4):
        print(__doc__)
        sys.exit(2)
    with open(sys.argv[2], "w") as f:
        f.write(render(sys.argv[1], full="--fragment" not in sys.argv))
    print(f"wrote {sys.argv[2]}")
