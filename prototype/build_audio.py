"""Cut the voice files into prayers, speed them up, and lay out the soundtrack scene by scene.

Usage: python3 build_audio.py <set>
Reads audio/short.mp3, audio/long.mp3, audio/<set>.mp3 and steps.json.
Writes clips/*.wav, build/<set>/full.wav and build/<set>/scenes.json.
"""
import json
import math
import pathlib
import re
import subprocess
import sys
import wave

import numpy as np

SR, FPS = 48000, 30
SPF = SR // FPS            # samples per video frame
TEMPO = 1.75               # Ben's chosen speed
LEAD, TAIL = 0.30, 0.70    # silence before and after each prayer, seconds
ANN_GAP = 0.45             # pause between naming the mystery and the Our Father
INTRO, OUTRO = 4.0, 4.0
TARGET_LUFS, PEAK_DB = -17.0, -1.5

here = pathlib.Path(__file__).resolve().parent
set_key = sys.argv[1]
clips_dir = here / "clips"
out_dir = here / "build" / set_key
clips_dir.mkdir(exist_ok=True)
out_dir.mkdir(parents=True, exist_ok=True)


def run(cmd):
    return subprocess.run(cmd, capture_output=True, text=True)


def duration(path):
    return float(run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", str(path)]).stdout)


def silences(path):
    err = run(["ffmpeg", "-hide_banner", "-nostats", "-i", str(path), "-af", "silencedetect=noise=-38dB:d=0.25", "-f", "null", "-"]).stderr
    starts = [float(x) for x in re.findall(r"silence_start: ([\d.]+)", err)]
    ends = [float(x) for x in re.findall(r"silence_end: ([\d.]+)", err)]
    return list(zip(starts, ends))


def split(path, names, texts):
    """The prayers in one file are separated by the longest pauses. Cut there and check the result."""
    dur = duration(path)
    inner = [s for s in silences(path) if s[0] > 0.05 and s[1] < dur - 0.05]
    by_len = sorted(inner, key=lambda s: s[1] - s[0], reverse=True)
    k = len(names) - 1
    cuts = sorted(by_len[:k])
    shortest_cut = by_len[k - 1][1] - by_len[k - 1][0]
    longest_other = by_len[k][1] - by_len[k][0]
    if shortest_cut < 1.25 * longest_other:
        raise SystemExit(f"{path.name}: pauses between prayers are not clearly longer than pauses inside them "
                         f"({shortest_cut:.2f}s vs {longest_other:.2f}s)")
    bounds = [0.0] + [t for c in cuts for t in c] + [dur]
    rates = []
    for i, name in enumerate(names):
        a, b = bounds[2 * i], bounds[2 * i + 1]
        a2, b2 = max(0.0, a - 0.04), min(dur, b + 0.10)
        out = clips_dir / f"{name}.wav"
        af = (f"atrim=start={a2:.4f}:end={b2:.4f},asetpts=PTS-STARTPTS,atempo={TEMPO},"
              f"afade=t=in:d=0.01,areverse,afade=t=in:d=0.03,areverse,aresample={SR}")
        r = run(["ffmpeg", "-y", "-hide_banner", "-loglevel", "error", "-i", str(path), "-af", af, "-ac", "1", "-c:a", "pcm_s16le", str(out)])
        if r.returncode:
            raise SystemExit(r.stderr)
        rate = len(texts[i]) / (b - a)
        rates.append(rate)
        print(f"  {name:10s} {a:7.2f} -> {b:7.2f}  {b - a:6.2f}s  -> {(b2 - a2) / TEMPO:5.2f}s at {TEMPO}x   {rate:4.1f} chars/s")
    if max(rates) > 1.5 * min(rates):
        raise SystemExit(f"{path.name}: speaking rate varies too much between pieces, so a cut is probably wrong")
    print(f"  pauses used as cuts >= {shortest_cut:.2f}s; longest pause inside a prayer {longest_other:.2f}s")


def load(name):
    with wave.open(str(clips_dir / f"{name}.wav")) as w:
        assert w.getframerate() == SR and w.getnchannels() == 1 and w.getsampwidth() == 2
        return np.frombuffer(w.readframes(w.getnframes()), dtype=np.int16).astype(np.float32) / 32768.0


core = json.loads((here / "steps.json").read_text())
P, steps, mysteries = core["prayers"], core["steps"], core["sets"][set_key]

print("short prayers")
split(here / "audio/short.mp3", ["sign", "ourfather", "hailmary", "glorybe", "fatima"],
      [" ".join(P[k]) for k in ["sign", "ourfather", "hailmary", "glorybe", "fatima"]])
print("long prayers")
split(here / "audio/long.mp3", ["creed", "hhq", "closing"], [" ".join(P[k]) for k in ["creed", "hhq", "closing"]])
print(f"{set_key} mysteries")
split(here / f"audio/{set_key}.mp3", [f"{set_key}-{d}" for d in range(1, 6)], mysteries)

clip = {k: load(k) for k in ["sign", "ourfather", "hailmary", "glorybe", "fatima", "creed", "hhq", "closing"]}
ann = {d: load(f"{set_key}-{d}") for d in range(1, 6)}
sil = lambda secs: np.zeros(int(round(secs * SR)), dtype=np.float32)


def to_frames(x):
    n = math.ceil(len(x) / SPF)
    return np.concatenate([x, np.zeros(n * SPF - len(x), dtype=np.float32)]), n


scenes, parts, frame = [], [], 0


def add(name, audio):
    global frame
    a, n = to_frames(audio)
    scenes.append({"name": name, "frames": n, "start_frame": frame, "start": round(frame / FPS, 3)})
    parts.append(a)
    frame += n


add("intro", sil(INTRO))
for i, st in enumerate(steps):
    body = [sil(LEAD)]
    if st.get("announce"):
        body += [ann[st["decade"]], sil(ANN_GAP)]
    body += [clip[st["prayer"]], sil(TAIL)]
    add(f"{i:03d}", np.concatenate(body))
add("outro", sil(OUTRO))

full = np.concatenate(parts)
tmp = out_dir / "measure.wav"
with wave.open(str(tmp), "wb") as w:
    w.setnchannels(1); w.setsampwidth(2); w.setframerate(SR)
    w.writeframes((np.clip(full, -1, 1) * 32767).astype(np.int16).tobytes())
err = run(["ffmpeg", "-hide_banner", "-nostats", "-i", str(tmp), "-af", "loudnorm=print_format=json", "-f", "null", "-"]).stderr
m = json.loads(err[err.rindex("{"):err.rindex("}") + 1])
# Raise the voice to a normal listening level; a limiter holds the few loud peaks under the ceiling.
gain_db = min(TARGET_LUFS - float(m["input_i"]), 8.0)
r = run(["ffmpeg", "-y", "-hide_banner", "-loglevel", "error", "-i", str(tmp), "-af",
         f"volume={gain_db:.2f}dB,alimiter=limit={10 ** (PEAK_DB / 20):.3f}:attack=5:release=60:level=false",
         "-ar", str(SR), "-ac", "1", "-c:a", "pcm_s16le", str(out_dir / "full.wav")])
if r.returncode:
    raise SystemExit(r.stderr)
tmp.unlink()
with wave.open(str(out_dir / "full.wav")) as w:
    assert w.getnframes() == frame * SPF, (w.getnframes(), frame * SPF)
(out_dir / "scenes.json").write_text(json.dumps(scenes))

total = frame / FPS
print(f"loudness {m['input_i']} LUFS, peak {m['input_tp']} dB -> gain {gain_db:+.1f} dB")
print(f"scenes {len(scenes)} | frames {frame} | length {int(total // 60)}:{total % 60:04.1f}")
