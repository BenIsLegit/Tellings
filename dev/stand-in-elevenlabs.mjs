// A stand-in for ElevenLabs, for testing the voice path without spending credits.
//
// It answers the three calls the tool makes (list voices, credits left, record
// speech). The speech comes from the voice built into Windows, so it sounds
// nothing like the real thing, but it is real speech at a natural pace: good
// enough to check timing, pace, pauses and the credit guard.
//
// Windows only. Run it, then point the tool at it in another terminal:
//
//   node dev/stand-in-elevenlabs.mjs
//
//   set PVC_ELEVENLABS_URL=http://127.0.0.1:8765
//   set ELEVENLABS_API_KEY=fake-key
//   set PVC_DATA_DIR=C:\some\scratch\folder      (so real kept speech is not touched)
//   pvc.bat make luminous --to C:\some\scratch\folder --approve-credits 2573
//
// Always set PVC_DATA_DIR. Without it, stand-in speech would be kept as if it
// were the real voice, and later videos would use it.
import http from "node:http";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";

const PORT = 8765;
const KEY = "fake-key";
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "stand-in-elevenlabs-"));
let calls = 0;
let letters = 0;

function speak(text, voiceId) {
  const input = path.join(tmp, "in.txt");
  const out = path.join(tmp, "out.wav");
  fs.writeFileSync(input, text, "utf8");
  // "ans" uses a second installed voice, so two voices taking turns sound different.
  const pick = voiceId === "ans" ? "$v = $s.GetInstalledVoices() | Select-Object -Last 1; $s.SelectVoice($v.VoiceInfo.Name);" : "";
  const ps = `Add-Type -AssemblyName System.Speech; $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; ${pick} $s.SetOutputToWaveFile('${out}'); $s.Speak([IO.File]::ReadAllText('${input}')); $s.Dispose()`;
  execFileSync("powershell", ["-NoProfile", "-Command", ps]);
  const wav = fs.readFileSync(out);
  const seconds = (wav.length - 44) / wav.readUInt32LE(28);
  // Real ElevenLabs says when each letter is spoken. Here the letters are spread evenly.
  const chars = [...text];
  const at = (i) => +(0.05 + (seconds - 0.1) * (i / chars.length)).toFixed(3);
  return {
    audio_base64: wav.toString("base64"),
    alignment: { characters: chars, character_start_times_seconds: chars.map((_, i) => at(i)), character_end_times_seconds: chars.map((_, i) => at(i + 1)) },
  };
}

http
  .createServer((req, res) => {
    const url = new URL(req.url, "http://x");
    const send = (code, body) => {
      res.writeHead(code, { "content-type": "application/json" });
      res.end(JSON.stringify(body));
    };
    if (req.headers["xi-api-key"] !== KEY) return send(401, { detail: { status: "invalid_api_key", message: "Invalid API key" } });
    if (url.pathname === "/v2/voices") {
      return send(200, {
        voices: [
          { voice_id: "imp", name: "Imperial - Male", category: "generated" },
          { voice_id: "ans", name: "Answer", category: "premade" },
        ],
        has_more: false,
      });
    }
    if (url.pathname === "/v1/user/subscription") return send(200, { character_count: letters, character_limit: 100000 });
    const m = url.pathname.match(/^\/v1\/text-to-speech\/([^/]+)\/with-timestamps$/);
    if (m && req.method === "POST") {
      let body = "";
      req.on("data", (c) => (body += c));
      req.on("end", () => {
        const { text } = JSON.parse(body);
        calls += 1;
        letters += [...text].length;
        console.log(`recorded piece ${calls}: voice ${m[1]}, ${[...text].length} letters (${letters} so far)`);
        send(200, speak(text, m[1]));
      });
      return;
    }
    send(404, { detail: "not found" });
  })
  .listen(PORT, "127.0.0.1", () => console.log(`Stand-in ElevenLabs on http://127.0.0.1:${PORT}  (key: ${KEY})`));
