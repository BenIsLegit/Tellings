"""Encode one video file per scene from the still frames, then join them and add the soundtrack.

Usage: python3 render.py <set> [--join]
Each scene is its still held for the length of its audio, with a short crossfade from the previous still
inside the file. Finished scenes are skipped, so an interrupted run can simply be started again.
"""
import json
import pathlib
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

FPS, XF = 30, 9   # crossfade length in frames (0.3 s)
here = pathlib.Path(__file__).resolve().parent
set_key = sys.argv[1]
build = here / "build" / set_key
frames, out_dir = build / "frames", build / "scenes"
out_dir.mkdir(parents=True, exist_ok=True)
scenes = json.loads((build / "scenes.json").read_text())

TO_YUV = "format=rgb24,scale=out_color_matrix=bt709:out_range=tv:flags=accurate_rnd+full_chroma_int,format=yuv420p,loop=-1:1"
ENC = ["-r", str(FPS), "-c:v", "libx264", "-crf", "18", "-preset", "veryfast", "-pix_fmt", "yuv420p",
       "-colorspace", "bt709", "-color_primaries", "bt709", "-color_trc", "bt709", "-color_range", "tv",
       "-video_track_timescale", "15360", "-an"]


def render(i):
    sc = scenes[i]
    n = sc["frames"]
    out = out_dir / f"{i:03d}_{sc['name']}.mp4"
    if out.exists():
        return "skip"
    tmp = out_dir / f"{i:03d}_{sc['name']}.part.mp4"
    cur = frames / f"{sc['name']}.png"
    if i == 0:
        inputs = ["-framerate", str(FPS), "-i", str(cur)]
        graph = f"[0]{TO_YUV},trim=end_frame={n},setpts=PTS-STARTPTS,fade=t=in:st=0:d=0.8[v]"
    else:
        prev = frames / f"{scenes[i - 1]['name']}.png"
        inputs = ["-framerate", str(FPS), "-i", str(prev), "-framerate", str(FPS), "-i", str(cur)]
        tail = f",fade=t=out:st={n / FPS - 1.0:.3f}:d=1.0" if i == len(scenes) - 1 else ""
        graph = (f"[0]{TO_YUV},trim=end_frame={XF},setpts=PTS-STARTPTS[a];"
                 f"[1]{TO_YUV},trim=end_frame={n},setpts=PTS-STARTPTS[b];"
                 f"[a][b]xfade=transition=fade:duration={XF / FPS}:offset=0{tail}[v]")
    cmd = ["ffmpeg", "-y", "-hide_banner", "-loglevel", "error", *inputs, "-filter_complex", graph,
           "-map", "[v]", "-frames:v", str(n), *ENC, str(tmp)]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode:
        raise SystemExit(f"scene {i}: {r.stderr[-800:]}")
    tmp.rename(out)
    return "done"


if "--join" not in sys.argv:
    with ThreadPoolExecutor(max_workers=2) as pool:
        results = list(pool.map(render, range(len(scenes))))
    print("rendered", results.count("done"), "| already there", results.count("skip"))
else:
    lst = build / "list.txt"
    lst.write_text("".join(f"file '{out_dir / f'{i:03d}_{sc['name']}.mp4'}'\n" for i, sc in enumerate(scenes)))
    silent = build / "silent.part.mp4"
    final = build / sys.argv[sys.argv.index("--join") + 1]
    for cmd in (
        ["ffmpeg", "-y", "-hide_banner", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", str(lst), "-c", "copy", str(silent)],
        ["ffmpeg", "-y", "-hide_banner", "-loglevel", "error", "-i", str(silent), "-i", str(build / "full.wav"),
         "-map", "0:v", "-map", "1:a", "-c:v", "copy", "-c:a", "aac", "-b:a", "160k", "-movflags", "+faststart", str(final)],
    ):
        r = subprocess.run(cmd, capture_output=True, text=True)
        if r.returncode:
            raise SystemExit(r.stderr[-800:])
    silent.unlink()
    print("wrote", final)
