// Small helpers for showing numbers to a person.

/** 790 seconds becomes "13:10". */
export function clock(seconds: number): string {
  const s = Math.max(0, Math.round(seconds));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

/** 2573 becomes "2,573". */
export function commas(n: number): string {
  return Math.round(n).toLocaleString("en-US");
}

/** What a model charges, in words: 1 becomes "1 credit a letter", 0.5 "half a credit a letter". */
export function rateWords(rate: number): string {
  if (Math.abs(rate - 1) < 0.001) return "1 credit a letter";
  if (Math.abs(rate - 0.5) < 0.001) return "half a credit a letter";
  return `${Math.round(rate * 100) / 100} credits a letter`;
}

export function megabytes(bytes: number): string {
  return `${(bytes / 1_048_576).toFixed(1)} MB`;
}

/** The last part of a path: "C:\\a\\b.png" becomes "b.png". */
export function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

/** Several voices share one setting, joined like this: "Bella + George". They take turns. */
const VOICE_JOIN = " + ";

export function splitVoices(name: unknown): string[] {
  return String(name ?? "")
    .split(VOICE_JOIN)
    .map((n) => n.trim())
    .filter((n) => n !== "");
}

export function joinVoices(names: string[]): string {
  return names.join(VOICE_JOIN);
}

const SECTION_NAMES: Record<string, string> = {
  intro: "Opening title",
  opening: "Opening prayers",
  closing: "Closing prayers",
  outro: "Ending",
};

export function sectionName(section: string): string {
  if (section.startsWith("decade-")) return `Decade ${section.slice(7)}`;
  return SECTION_NAMES[section] ?? section;
}

const PRAYER_NAMES: Record<string, string> = {
  sign: "Sign of the Cross",
  creed: "Apostles' Creed",
  ourfather: "Our Father",
  hailmary: "Hail Mary",
  glorybe: "Glory Be",
  fatima: "O My Jesus",
  hhq: "Hail, Holy Queen",
  closing: "Closing Prayer",
  reading: "reading",
  extra: "extra prayer",
};

export function prayerName(key: string): string {
  return PRAYER_NAMES[key] ?? key;
}

/** A bead id in plain words: "d3b10" becomes "Decade 3, bead 10". */
export function beadName(id: string): string {
  const decade = /^d(\d)b(\d+)$/.exec(id);
  if (decade) return `Decade ${decade[1]}, bead ${decade[2]}`;
  if (id === "cross") return "The crucifix";
  if (id === "medal") return "The medal";
  if (id === "p1") return "First big bead";
  if (id === "p5") return "Second big bead";
  if (/^p[234]$/.test(id)) return `Small bead ${Number(id[1]) - 1} of 3`;
  if (/^L\d$/.test(id)) return `Big bead before decade ${id[1]}`;
  if (id === "g0") return "Chain before the first decade";
  if (/^g\d$/.test(id)) return `Chain after decade ${id[1]}`;
  return id;
}

const ROMAN = ["I", "II", "III", "IV", "V"];

/** A part of the video in plain words, for the list of rules. */
export function targetName(t: { section?: string; mystery?: number; prayer?: string; bead?: string }): string {
  const parts: string[] = [];
  if (t.bead) parts.push(beadName(t.bead));
  if (t.prayer) parts.push(`every ${prayerName(t.prayer)}`);
  if (t.mystery) parts.push(`mystery ${ROMAN[t.mystery - 1] ?? t.mystery}`);
  if (t.section) parts.push(sectionName(t.section));
  if (parts.length === 0) return "The whole video";
  const text = parts.join(", in ");
  return text.charAt(0).toUpperCase() + text.slice(1);
}
