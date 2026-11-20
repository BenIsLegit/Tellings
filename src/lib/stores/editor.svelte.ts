// The setup being edited, and the one idea that makes editing easy:
//
//   every narrowing setting is changed "for" something — the whole video, this
//   section, this mystery, every prayer of this kind, or this one bead.
//
// The editor keeps the sparse setup (`doc`). After each change it asks the
// engine to fill in the defaults and plan the video, and shows that.

import { api } from "$lib/api";
import type { Doc, Entry, Look, Plan, Scene, Setup, Target } from "$lib/types";
import { prayerName, sectionName } from "$lib/utils/format";
import { deleteInRule, deletePath, findRule, getPath, hasPath, setInRule, setPath } from "$lib/utils/path";

export type ScopeKind = "video" | "section" | "mystery" | "prayer" | "bead";

/** The groups of settings that can be set for part of the video. */
export const NARROWING = ["voice", "pace", "words", "picture", "colors"];

const ROMAN = ["I", "II", "III", "IV", "V"];

/** Where a scene keeps the value it ended up with, for each narrowing group. */
const SCENE_FIELD: Record<string, keyof Scene> = { voice: "voice", pace: "pace", words: "words", picture: "pictureChoice", colors: "colorChoice" };

class Editor {
  /** What was opened from the library. */
  entry = $state<Entry | null>(null);
  /** The sparse setup: only what differs from what it starts from. */
  doc = $state<Doc>({});
  /** The same setup with every default filled in. */
  setup = $state<Setup | null>(null);
  look = $state<Look | null>(null);
  /** Which scene the preview shows. */
  index = $state(1);
  scope = $state<ScopeKind>("video");
  dirty = $state(false);
  problem = $state<string | null>(null);
  busy = $state(false);

  #timer: ReturnType<typeof setTimeout> | undefined;
  #serial = 0;

  get plan(): Plan | null {
    return this.look?.plan ?? null;
  }

  get scene(): Scene | null {
    const scenes = this.plan?.scenes;
    if (!scenes?.length) return null;
    return scenes[Math.max(0, Math.min(this.index, scenes.length - 1))];
  }

  get isOpen(): boolean {
    return this.entry !== null;
  }

  get name(): string {
    return (this.doc.name as string) ?? this.entry?.name ?? "";
  }

  /** The preset or setup this one starts from, if any. */
  get startsFrom(): string | null {
    return (this.doc.extends as string) ?? null;
  }

  async open(id: string) {
    this.busy = true;
    try {
      const opened = await api.setupOpen(id);
      this.entry = opened.entry;
      this.doc = opened.doc;
      this.dirty = false;
      this.index = 1;
      this.scope = "video";
      this.setup = null;
      this.look = null;
      await this.refresh();
    } finally {
      this.busy = false;
    }
  }

  /** Call after any change to `doc`. Re-plans shortly after the last change. */
  touch() {
    this.dirty = true;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => this.refresh(), 160);
  }

  async refresh() {
    const serial = ++this.#serial;
    try {
      const checked = await api.setupLook($state.snapshot(this.doc));
      if (serial !== this.#serial) return;
      this.setup = checked.setup;
      this.look = checked.look;
      this.problem = null;
      this.index = Math.min(this.index, checked.look.plan.scenes.length - 1);
    } catch (e) {
      if (serial === this.#serial) this.problem = String(e);
    }
  }

  go(index: number) {
    const n = this.plan?.scenes.length ?? 1;
    this.index = Math.max(0, Math.min(n - 1, index));
  }

  /** Jump to the first scene on a bead. Used when a bead is clicked in the preview. */
  goToBead(bead: string) {
    const i = this.plan?.scenes.findIndex((s) => s.bead === bead) ?? -1;
    if (i >= 0) this.index = i;
  }

  // ───────────── which part of the video a change covers ─────────────

  /** The part of the video a scope covers at the current scene. null = not possible here. */
  targetOf(kind: ScopeKind, scene: Scene | null = this.scene): Target | null {
    if (!scene) return null;
    switch (kind) {
      case "section":
        return { section: scene.section };
      case "mystery":
        return scene.mystery ? { mystery: scene.mystery } : null;
      case "prayer":
        return scene.prayer ? { prayer: scene.prayer } : null;
      case "bead":
        return scene.bead ? { bead: scene.bead } : null;
      default:
        return null;
    }
  }

  /** The five choices in the "This choice covers" bar, in order of narrowness. */
  get scopes(): { kind: ScopeKind; label: string; hint: string; available: boolean }[] {
    const s = this.scene;
    return [
      { kind: "video", label: "Whole video", hint: "Everything, unless a narrower choice says otherwise", available: true },
      { kind: "section", label: s ? sectionName(s.section) : "This section", hint: "Only this section", available: !!s },
      {
        kind: "mystery",
        label: s?.mystery ? `Mystery ${ROMAN[s.mystery - 1]}` : "This mystery",
        hint: "Only this mystery",
        available: !!s?.mystery,
      },
      {
        kind: "prayer",
        label: s?.prayer ? `Every ${prayerName(s.prayer)}` : "This kind of prayer",
        hint: "Every prayer of this kind, wherever it comes",
        available: !!s?.prayer,
      },
      { kind: "bead", label: "This bead only", hint: "Only the bead the preview is on", available: !!s?.bead },
    ];
  }

  /** The scope in force. Falls back to the whole video where the chosen one makes no sense. */
  get activeScope(): ScopeKind {
    return this.scope !== "video" && this.targetOf(this.scope) ? this.scope : "video";
  }

  /** The target a narrowing setting is being changed for. null = the whole video. */
  get activeTarget(): Target | null {
    return this.activeScope === "video" ? null : this.targetOf(this.activeScope);
  }

  // ───────────── reading and changing one setting ─────────────

  narrows(path: string): boolean {
    return NARROWING.includes(path.split(".")[0]);
  }

  /** The value in force for a setting, given what it is being changed for. */
  valueFor(path: string, target: Target | null, scene: Scene | null = this.scene): any {
    if (!target || !this.narrows(path)) return getPath(this.setup, path);
    const rule = findRule(this.doc, target);
    const own = rule ? getPath(rule.set, path) : undefined;
    if (own !== undefined) return own;
    // Not set at this level: show what the scene ends up with from wider choices.
    const [group, ...rest] = path.split(".");
    const from = scene?.[SCENE_FIELD[group]] as Doc | undefined;
    return getPath(from, rest.join("."));
  }

  /** True when this setup itself sets the value at exactly this level. */
  isSetFor(path: string, target: Target | null): boolean {
    if (!target || !this.narrows(path)) return hasPath(this.doc, path);
    const rule = findRule(this.doc, target);
    return !!rule && hasPath(rule.set, path);
  }

  setFor(path: string, target: Target | null, value: any) {
    if (!target || !this.narrows(path)) setPath(this.doc, path, value);
    else setInRule(this.doc, target, path, value);
    this.touch();
  }

  /** Put a setting back to what it would be without this choice. */
  resetFor(path: string, target: Target | null) {
    if (!target || !this.narrows(path)) deletePath(this.doc, path);
    else deleteInRule(this.doc, target, path);
    this.touch();
  }

  /** The first scene a target covers, to read its values from. */
  sceneOf(target: Target): Scene | null {
    return (
      this.plan?.scenes.find(
        (s) =>
          (target.section === undefined || s.section === target.section) &&
          (target.mystery === undefined || s.mystery === target.mystery) &&
          (target.prayer === undefined || s.prayer === target.prayer) &&
          (target.bead === undefined || s.bead === target.bead),
      ) ?? null
    );
  }

  removeRule(index: number) {
    if (!Array.isArray(this.doc.rules)) return;
    this.doc.rules.splice(index, 1);
    if (this.doc.rules.length === 0) delete this.doc.rules;
    this.touch();
  }

  // ───────────── saving ─────────────

  /** Save under a name. Saving a preset under a new name keeps only the changes. */
  async save(name: string) {
    const saved = await api.setupSave(name, $state.snapshot(this.doc));
    this.entry = saved;
    this.doc.name = saved.name;
    this.dirty = false;
    return saved;
  }

  close() {
    this.entry = null;
    this.doc = {};
    this.setup = null;
    this.look = null;
    this.dirty = false;
    this.problem = null;
  }
}

export const editor = new Editor();
