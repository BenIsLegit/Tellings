// A stand-in for ElevenLabs, for testing the voice path without spending credits.
//
// It answers the calls the tool makes (list voices, models and pronunciation
// dictionaries, credits left, record speech). The speech comes from the voice
// built into Windows, so it sounds nothing like the real thing, but it is real
// speech at a natural pace: good enough to check timing, pace, pauses and the
// credit guard.
//
// It charges what each model charges (the flash model is half price) and says
// so the way ElevenLabs does, in a "character-cost" header. It prints what each
// request asked for, so you can see that a setting reached it.
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
let credits = 0;

// A made-up price list in the shape ElevenLabs sends. "eleven_dear" costs double, to test the guard.
const MODELS = [
  { model_id: "eleven_v4", name: "Eleven v4", description: "Stand-in for the newest model. Takes direction.", rate: 1, max: 100000, style: false, boost: false },
  { model_id: "eleven_v3", name: "Eleven v3", description: "Stand-in for the model before it. Takes direction.", rate: 1, max: 5000, style: false, boost: false },
  { model_id: "eleven_multilingual_v2", name: "Eleven Multilingual v2", description: "Stand-in for the model the first video used.", rate: 1, max: 10000, style: true, boost: true },
  { model_id: "eleven_flash_v2_5", name: "Eleven Flash v2.5", description: "Stand-in for the fast, half-price model.", rate: 0.5, max: 40000, style: false, boost: false },
  { model_id: "eleven_dear", name: "Eleven Dear", description: "Not a real model. Charges double.", rate: 2, max: 10000, style: true, boost: true },
  { model_id: "eleven_multilingual_sts_v2", name: "Voice changer", description: "Does not turn text into speech.", rate: 1, max: 10000, speech: false },
].map((m) => ({
  model_id: m.model_id,
  name: m.name,
  description: m.description,
  can_do_text_to_speech: m.speech !== false,
  can_use_style: !!m.style,
  can_use_speaker_boost: !!m.boost,
  maximum_text_length_per_request: m.max,
  languages: [{ language_id: "en", name: "English" }],
  model_rates: { character_cost_multiplier: m.rate },
}));

/** A direction like "[reverent] " in front is not spoken, as with Eleven v3 and v4. */
const spokenPart = (text) => text.replace(/^(\s*\[[^\]]*\])+\s*/, "");

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
          { voice_id: "imp", name: "Imperial - Male", category: "generated", settings: null },
          { voice_id: "ans", name: "Answer", category: "premade", settings: null },
        ],
        has_more: false,
      });
    }
    // How one voice is set up. The real list of voices does not carry this, so the tool asks voice by voice.
    if (/^\/v1\/voices\/[^/]+\/settings$/.test(url.pathname)) {
      const own = url.pathname.includes("/ans/") ? { stability: 0.35, similarity_boost: 0.8, style: 0.1, use_speaker_boost: false, speed: 1 } : { stability: 0.5, similarity_boost: 0.75, style: 0, use_speaker_boost: true, speed: 1 };
      return send(200, own);
    }
    if (url.pathname === "/v1/models") return send(200, MODELS);
    if (url.pathname === "/v1/pronunciation-dictionaries") {
      return send(200, { pronunciation_dictionaries: [{ id: "dict1", name: "Prayers", latest_version_id: "ver3", latest_version_rules_num: 2 }], has_more: false });
    }
    if (url.pathname === "/v1/user/subscription") {
      const nextMonth = Math.floor(Date.now() / 1000) + 30 * 86400;
      return send(200, { character_count: credits, character_limit: 100000, tier: "stand-in", next_character_count_reset_unix: nextMonth });
    }
    const m = url.pathname.match(/^\/v1\/text-to-speech\/([^/]+)\/with-timestamps$/);
    if (m && req.method === "POST") {
      let body = "";
      req.on("data", (c) => (body += c));
      req.on("end", () => {
        const { text, model_id, ...asked } = JSON.parse(body);
        const model = MODELS.find((x) => x.model_id === (model_id ?? "eleven_multilingual_v2") && x.can_do_text_to_speech);
        if (!model) return send(400, { detail: { status: "model_not_found", message: `There is no speech model called ${model_id}.` } });
        const count = [...text].length;
        const cost = Math.ceil(count * model.model_rates.character_cost_multiplier);
        calls += 1;
        letters += count;
        credits += cost;
        const extras = { format: url.searchParams.get("output_format"), ...asked };
        console.log(`recorded piece ${calls}: voice ${m[1]}, ${model.model_id}, ${count} letters, ${cost} credits (${credits} so far)  ${JSON.stringify(extras)}`);
        res.writeHead(200, { "content-type": "application/json", "character-cost": String(cost), "request-id": `stand-in-${calls}` });
        res.end(JSON.stringify(speak(spokenPart(text), m[1])));
      });
      return;
    }
    send(404, { detail: "not found" });
  })
  .listen(PORT, "127.0.0.1", () => console.log(`Stand-in ElevenLabs on http://127.0.0.1:${PORT}  (key: ${KEY})`));
