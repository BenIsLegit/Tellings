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
  lettersKept: number;
  piecesToRecord: number;
  byVoice: { voice: string; letters: number; pieces: number }[];
  missingRecordings: string[];
}

export type KeySource = "environment" | "password-store";

export interface Look {
  plan: Plan;
  estimate: Estimate;
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

export interface Credits {
  used: number;
  limit: number;
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
