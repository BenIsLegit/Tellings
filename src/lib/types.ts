// The shapes the engine hands to the window. They mirror the Rust types in
// crates/engine (setup.rs, plan.rs, make.rs). Field names are camelCase on both sides.

/** A setup as a person wrote it: only what differs from the defaults. */
export type Doc = Record<string, any>;

export interface Entry {
  id: string;
  name: string;
  description: string;
  kind: "preset" | "saved" | "file";
  extends: string | null;
}

/** What part of the video a rule covers. */
export interface Target {
  section?: string;
  mystery?: number;
  prayer?: string;
  bead?: string;
}

export interface Rule {
  for: Target;
  set: Doc;
  note?: string;
}

/** A setup with every default filled in. Groups are read by dotted path. */
export interface Setup {
  name: string;
  description: string;
  beads: Record<string, any>;
  prayers: Record<string, any>;
  voice: Record<string, any>;
  pace: Record<string, any>;
  words: Record<string, any>;
  picture: Record<string, any>;
  colors: Record<string, any>;
  screen: Record<string, any>;
  music: Record<string, any>;
  output: Record<string, any>;
  rules: Rule[];
}

export interface Scene {
  id: string;
  kind: "intro" | "prayer" | "outro";
  number: number | null;
  section: string;
  mystery: number | null;
  prayer: string | null;
  bead: string | null;
  place: string;
  title: string;
  lines: string[];
  showLines: boolean;
  picture: { file: string; focus: string; zoom: number; credit: string; place: string } | null;
  words: Record<string, any>;
  voice: Record<string, any>;
  /** When several voices take turns, the one that leads here. */
  speaker: string | null;
  pace: Record<string, any>;
  pictureChoice: Record<string, any>;
  colorChoice: Record<string, any>;
  sayFirst: string | null;
  muted: boolean;
  chapter: string | null;
  seconds: number;
}

export interface Plan {
  name: string;
  setKey: string;
  setName: string;
  stage: { width: number; height: number; fps: number };
  scenes: Scene[];
  prayerCount: number;
  seconds: number;
  warnings: string[];
}

export interface Estimate {
  lettersToRecord: number;
  /** What recording them is expected to cost. This is the number a person agrees to. */
  creditsToRecord: number;
  lettersKept: number;
  piecesToRecord: number;
  byVoice: { voice: string; letters: number; credits: number; pieces: number }[];
  /** The ElevenLabs models that would do the recording. */
  models: string[];
  /** True when what a model charges had to be guessed. */
  ratesGuessed: boolean;
  missingRecordings: string[];
}

export type KeySource = "environment" | "password-store";

/** One piece of speech as it is heard on one scene. */
export interface Heard {
  part: "announce" | "prayer";
  /** The voice, or the name of the person's own sound file. */
  voice: string;
  ownRecording: boolean;
  /** Which recording of these words by this voice, counting from 1. */
  take: number;
  letters: number;
  /** What recording it costs, or would cost again. */
  credits: number;
  /** The ElevenLabs model that speaks it. Empty for the person's own recording. */
  model: string;
  /** Already recorded and kept. */
  kept: boolean;
  /** How many times this same recording is heard in the whole video. */
  uses: number;
  /** How many different recordings of these words by this voice the video uses. */
  takes: number;
}

/** Music ElevenLabs would be asked to make, and roughly what that would cost. */
export interface MusicCost {
  /** True when the setup asks ElevenLabs for music and the video would use it. */
  wanted: boolean;
  /** Already made and kept: it costs nothing now. */
  kept: boolean;
  minutes: number;
  /** Roughly what making it would cost. A rough figure: ElevenLabs does not give the price beforehand. */
  credits: number;
}

export interface Look {
  plan: Plan;
  /** The voice: what there is to record and what that would cost. */
  estimate: Estimate;
  music: MusicCost;
  /** The one number a person agrees to: the voice and the music together. */
  credits: number;
  /** For each scene of the plan, the speech heard on it. */
  speech: Heard[][];
  key: KeySource | null;
  files: string[];
}

export interface Checked {
  setup: Setup;
  look: Look;
}

export interface Progress {
  stage: "voice" | "sound" | "stills" | "video" | "finishing";
  message: string;
  done: number;
  total: number;
  fraction: number;
}

export interface Made {
  name: string;
  files: { kind: string; path: string }[];
  seconds: number;
  lettersRecorded: number;
  /** Credits known to have been spent: the voice, and the music where its cost could be told. */
  creditsSpent: number;
  /** The music made this time, and what is known of its cost. Never a guess. */
  music: { minutes: number; charged: number | null; knownBy: "reported" | "counted" | "unmoved" | "unknown"; estimate: number } | null;
  /** The same in a sentence. */
  musicSaid: string | null;
  pace: { voice: string; natural: number; speedup: number }[];
  hailMarySeconds: number | null;
  warnings: string[];
  took: number;
}

export interface VoiceInfo {
  id: string;
  name: string;
  category?: string | null;
  description?: string | null;
}

/** How a voice is set to speak at ElevenLabs: what is used wherever a setup leaves a setting unset. */
export interface OwnSettings {
  stability?: number | null;
  similarity?: number | null;
  style?: number | null;
  speakerBoost?: boolean | null;
  speed?: number | null;
}

/** One ElevenLabs model that turns text into speech. */
export interface ModelInfo {
  id: string;
  name: string;
  description: string;
  /** Credits for each letter. 1 for most models, 0.5 for the fast ones. */
  creditsPerLetter: number;
  /** The most letters one piece of speech may hold. 0 when ElevenLabs did not say. */
  maxLetters: number;
  canUseStyle: boolean;
  canUseSpeakerBoost: boolean;
  /** True when it reads a tag like [reverent] as direction, instead of saying it aloud. */
  takesDirection: boolean;
  /** True when it can record two voices together, as one conversation. */
  speaksDialogue: boolean;
  languages: { id: string; name: string }[];
}

export interface Credits {
  used: number;
  limit: number;
  /** The plan: "free", "starter", "creator" … */
  tier?: string | null;
  /** When the count starts again, in seconds since 1970. */
  resets?: number | null;
}

export interface Doctor {
  ffmpeg: string | null;
  ffmpegProblem: string | null;
  browser: string | null;
  browserProblem: string | null;
  key: KeySource | null;
  dataFolder: string;
  picturesFolder: string;
  videosFolder: string;
  keptSpeechBytes: number;
  version: string;
}

export interface Settings {
  outputFolder?: string | null;
  browser?: string | null;
  ffmpeg?: string | null;
}

export interface Options {
  screens: { key: string; name: string; width: number; height: number }[];
  prayers: { key: string; title: string; lines: string[] }[];
  sets: { key: string; name: string; days: string; items: { name: string; reference: string | null; note: string | null }[] }[];
}
