"""Screenshot rosary video frames from frame.html.

Usage: python3 shoot.py <set> <outdir> [scene ...]
A scene is "intro", "outro" or a step number. With no scenes, every scene is shot.
Writes <outdir>/<name>.jpg and <outdir>/fit.json (the prayer font size used per scene).
"""
import json
import pathlib
import sys

from playwright.sync_api import sync_playwright

here = pathlib.Path(__file__).resolve().parent
set_key, outdir = sys.argv[1], pathlib.Path(sys.argv[2])
outdir.mkdir(parents=True, exist_ok=True)
wanted = sys.argv[3:]

with sync_playwright() as p:
    browser = p.chromium.launch(executable_path="/usr/bin/google-chrome", args=["--no-sandbox", "--force-color-profile=srgb"])
    page = browser.new_page(viewport={"width": 2076, "height": 2152}, device_scale_factor=1)
    page.goto((here / "frame.html").as_uri())
    n = page.evaluate("steps.length")
    # make sure every face is loaded before any text is measured
    page.evaluate("""async () => {
        await Promise.all(['400 60px Cardo', '700 60px Cardo', '400 60px "Fira Sans"', '500 60px "Fira Sans"'].map(f => document.fonts.load(f)));
        await document.fonts.ready;
        await window.ready;
    }""")
    scenes = wanted or (["intro"] + [str(i) for i in range(n)] + ["outro"])
    fits = {}
    for s in scenes:
        scene = {"kind": s, "set": set_key} if s in ("intro", "outro") else {"kind": "step", "i": int(s), "set": set_key}
        fits[s] = page.evaluate("s => show(s)", scene)
        over = page.evaluate("document.getElementById('col').scrollHeight > document.getElementById('col').clientHeight")
        if over:
            raise SystemExit(f"scene {s}: text does not fit the column")
        name = s if s in ("intro", "outro") else f"{int(s):03d}"
        # Stills, not filmed motion: PNG keeps the type and the paintings exact for the encoder.
        page.locator("#stage").screenshot(path=str(outdir / f"{name}.png"), type="png")
    (outdir / "fit.json").write_text(json.dumps(fits))
    fonts = page.evaluate("[...document.fonts].filter(f => f.status === 'loaded').map(f => f.family)")
    print("scenes", len(scenes), "| smallest prayer size", min(fits.values()), "| web fonts loaded", fonts)
    browser.close()
