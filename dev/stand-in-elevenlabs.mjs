// A stand-in for ElevenLabs, for testing the voice path without spending credits.
//
// It answers the calls the tool makes (list voices, models and pronunciation
// dictionaries, credits left, record speech, record two voices as one
// conversation, make music). The speech comes from the voices built into Windows, so it
// sounds nothing like the real thing, but it is real speech at a natural pace:
// good enough to check timing, pace, pauses and the credit guard.
//
// It charges what each model charges (the flash model is half price) and says
// so the way ElevenLabs does, in a "character-cost" header. It prints what each
// request asked for, so you can see that a setting reached it.
//
// Music is a quiet made-up chord of the length asked. It costs 15 credits a
// second, and like the real service the stand-in does not say so: the tool has
// to read the credits before and after. A prompt with "ave maria" in it is
// refused, the way the real service refuses the name of a song.
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

// ── two voices as one conversation ──

const RATE = 22050;

/** Windows speech as plain samples (16 bits, one channel), always at the same rate, so two voices can be joined. */
function speakSamples(text, voiceId) {
  const input = path.join(tmp, "in.txt");
  const out = path.join(tmp, "part.wav");
  fs.writeFileSync(input, text, "utf8");
  const pick = voiceId === "ans" ? "$v = $s.GetInstalledVoices() | Select-Object -Last 1; $s.SelectVoice($v.VoiceInfo.Name);" : "";
  const format = `$f = New-Object System.Speech.AudioFormat.SpeechAudioFormatInfo(${RATE}, [System.Speech.AudioFormat.AudioBitsPerSample]::Sixteen, [System.Speech.AudioFormat.AudioChannel]::Mono);`;
  const ps = `Add-Type -AssemblyName System.Speech; $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; ${pick} ${format} $s.SetOutputToWaveFile('${out}', $f); $s.Speak([IO.File]::ReadAllText('${input}')); $s.Dispose()`;
  execFileSync("powershell", ["-NoProfile", "-Command", ps]);
  const wav = fs.readFileSync(out);
  const data = wav.indexOf("data", 12, "latin1");
  return wav.subarray(data + 8, data + 8 + wav.readUInt32LE(data + 4));
}

function wavOf(samples) {
  const head = Buffer.alloc(44);
  head.write("RIFF", 0, "latin1");
  head.writeUInt32LE(36 + samples.length, 4);
  head.write("WAVEfmt ", 8, "latin1");
  head.writeUInt32LE(16, 16);
  head.writeUInt16LE(1, 20); // plain samples
  head.writeUInt16LE(1, 22); // one channel
  head.writeUInt32LE(RATE, 24);
  head.writeUInt32LE(RATE * 2, 28);
  head.writeUInt16LE(2, 32);
  head.writeUInt16LE(16, 34);
  head.write("data", 36, "latin1");
  head.writeUInt32LE(samples.length, 40);
  return Buffer.concat([head, samples]);
}

/**
 * Each part in its own voice, with a breath between, joined into one sound.
 *
 * The answer is shaped the way ElevenLabs' own example shows it: "characters"
 * is the parts' texts run together with nothing between them, and each voice
 * segment says where its part lies among those letters and in time. The real
 * service does not say whether a direction's letters are counted. Here they
 * are, and they take no time.
 */
function converse(inputs) {
  const breath = 0.25;
  const sound = [];
  const characters = [];
  const starts = [];
  const ends = [];
  const voice_segments = [];
  let now = 0;
  inputs.forEach((input, index) => {
    const said = spokenPart(input.text);
    const samples = speakSamples(said, input.voice_id);
    const seconds = samples.length / 2 / RATE;
    const all = [...input.text];
    const spoken = [...said].length;
    const unspoken = all.length - spoken;
    const at = (k) => +(now + 0.05 + (seconds - 0.1) * (Math.min(Math.max(k - unspoken, 0), spoken) / Math.max(spoken, 1))).toFixed(3);
    const from = characters.length;
    all.forEach((letter, k) => {
      characters.push(letter);
      starts.push(at(k));
      ends.push(at(k + 1));
    });
    voice_segments.push({
      voice_id: input.voice_id,
      start_time_seconds: +now.toFixed(3),
      end_time_seconds: +(now + seconds).toFixed(3),
      character_start_index: from,
      character_end_index: characters.length,
      dialogue_input_index: index,
    });
    sound.push(samples);
    now += seconds;
    if (index < inputs.length - 1) {
      sound.push(Buffer.alloc(Math.round(RATE * breath) * 2));
      now += breath;
    }
  });
  return {
    audio_base64: wavOf(Buffer.concat(sound)).toString("base64"),
    alignment: { characters, character_start_times_seconds: starts, character_end_times_seconds: ends },
    normalized_alignment: null,
    voice_segments,
  };
}

// ── music ──

/** Three soft notes that swell and fade: plainly not silence, and plainly not speech. */
function musicOf(ms) {
  const count = Math.round((RATE * ms) / 1000);
  const samples = Buffer.alloc(count * 2);
  const notes = [220, 277.18, 329.63];
  for (let i = 0; i < count; i++) {
    const t = i / RATE;
    const swell = 0.6 + 0.4 * Math.sin((2 * Math.PI * t) / 8);
    let v = 0;
    for (const f of notes) v += Math.sin(2 * Math.PI * f * t);
    samples.writeInt16LE(Math.round((v / notes.length) * swell * 6000), i * 2);
  }
  return wavOf(samples);
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
    if (url.pathname === "/v1/text-to-dialogue/with-timestamps" && req.method === "POST") {
      let body = "";
      req.on("data", (c) => (body += c));
      req.on("end", () => {
        const { inputs, model_id, ...asked } = JSON.parse(body);
        // Over plain requests, only these two record a conversation. Their quick kin need a live connection.
        const model = MODELS.find((x) => x.model_id === (model_id ?? "eleven_v3") && ["eleven_v3", "eleven_v4"].includes(x.model_id));
        if (!model) return send(400, { detail: { status: "unsupported_model", message: `The model ${model_id} cannot record a conversation.` } });
        if (!Array.isArray(inputs) || inputs.length === 0) return send(422, { detail: [{ loc: ["body", "inputs"], msg: "Field required.", type: "missing" }] });
        const count = inputs.reduce((n, input) => n + [...input.text].length, 0);
        if (count > 2000) return send(422, { detail: [{ loc: ["body", "inputs"], msg: `The conversation is ${count} characters long. 2,000 is the most.`, type: "value_error" }] });
        const cost = Math.ceil(count * model.model_rates.character_cost_multiplier);
        calls += 1;
        letters += count;
        credits += cost;
        const voices = inputs.map((input) => input.voice_id).join(" then ");
        const extras = { format: url.searchParams.get("output_format"), ...asked, parts: inputs.map((input) => [...input.text].slice(0, 28).join("")) };
        console.log(`recorded conversation ${calls}: voices ${voices}, ${model.model_id}, ${count} letters, ${cost} credits (${credits} so far)  ${JSON.stringify(extras)}`);
        res.writeHead(200, { "content-type": "application/json", "character-cost": String(cost), "request-id": `stand-in-${calls}` });
        res.end(JSON.stringify(converse(inputs)));
      });
      return;
    }
    if (url.pathname === "/v1/music" && req.method === "POST") {
      let body = "";
      req.on("data", (c) => (body += c));
      req.on("end", () => {
        const { prompt, music_length_ms, model_id, ...asked } = JSON.parse(body);
        // The real service refuses the name of a song or an artist, and often offers other words.
        if (/ave maria/i.test(prompt ?? "")) {
          console.log(`refused music: the prompt names a song  ${JSON.stringify({ prompt })}`);
          return send(400, { detail: { status: "bad_prompt", message: "The prompt names a song.", data: { prompt_suggestion: "A slow, tender sacred melody for solo voice and organ" } } });
        }
        const ms = Math.min(Math.max(Number(music_length_ms) || 30000, 3000), 600000);
        const cost = Math.ceil((ms / 1000) * 15);
        calls += 1;
        credits += cost;
        const extras = { format: url.searchParams.get("output_format"), prompt, ...asked };
        console.log(`made music ${calls}: ${ms / 1000} seconds, ${model_id}, ${cost} credits (${credits} so far)  ${JSON.stringify(extras)}`);
        // No "character-cost" here: the real service is not known to send one for music.
        res.writeHead(200, { "content-type": "audio/wav", "song-id": `stand-in-song-${calls}` });
        res.end(musicOf(ms));
      });
      return;
    }
    send(404, { detail: "not found" });
  })
  .listen(PORT, "127.0.0.1", () => console.log(`Stand-in ElevenLabs on http://127.0.0.1:${PORT}  (key: ${KEY})`));
