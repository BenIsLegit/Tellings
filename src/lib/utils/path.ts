// Reading and changing one setting inside a setup, by its dotted name.
// Pure functions, so they can be tested without the app (path.test.ts).

type Obj = Record<string, any>;

export interface Target {
  section?: string;
  mystery?: number;
  prayer?: string;
  bead?: string;
}

export function getPath(obj: Obj | null | undefined, path: string): any {
  let cur: any = obj;
  for (const key of path.split(".")) {
    if (cur === null || cur === undefined || typeof cur !== "object") return undefined;
    cur = cur[key];
  }
  return cur;
}

export function hasPath(obj: Obj | null | undefined, path: string): boolean {
  return getPath(obj, path) !== undefined;
}

/** Set one value, making the groups above it as needed. */
export function setPath(obj: Obj, path: string, value: any): void {
  const keys = path.split(".");
  let cur = obj;
  for (const key of keys.slice(0, -1)) {
    if (cur[key] === null || typeof cur[key] !== "object" || Array.isArray(cur[key])) cur[key] = {};
    cur = cur[key];
  }
  cur[keys[keys.length - 1]] = value;
}

/** Remove one value, and any group left empty by removing it. */
export function deletePath(obj: Obj, path: string): void {
  const keys = path.split(".");
  const trail: Obj[] = [obj];
  for (const key of keys.slice(0, -1)) {
    const next = trail[trail.length - 1][key];
    if (next === null || typeof next !== "object") return;
    trail.push(next);
  }
  delete trail[trail.length - 1][keys[keys.length - 1]];
  for (let i = trail.length - 1; i > 0; i--) {
    if (Object.keys(trail[i]).length === 0) delete trail[i - 1][keys[i - 1]];
    else break;
  }
}

const TARGET_KEYS = ["section", "mystery", "prayer", "bead"] as const;

export function sameTarget(a: Target | null | undefined, b: Target | null | undefined): boolean {
  return TARGET_KEYS.every((k) => (a?.[k] ?? null) === (b?.[k] ?? null));
}

/** The rule in a setup that covers exactly this part of the video, if there is one. */
export function findRule(doc: Obj, target: Target): Obj | null {
  const rules: Obj[] = Array.isArray(doc.rules) ? doc.rules : [];
  return rules.find((r) => sameTarget(r.for, target)) ?? null;
}

/** Set one value for part of the video, adding the rule if it is new. */
export function setInRule(doc: Obj, target: Target, path: string, value: any): void {
  if (!Array.isArray(doc.rules)) doc.rules = [];
  let rule = findRule(doc, target);
  if (!rule) {
    doc.rules.push({ for: { ...target }, set: {} });
    // Read it back from the list, so a reactive list hands us its own copy.
    rule = doc.rules[doc.rules.length - 1] as Obj;
  }
  if (rule.set === null || typeof rule.set !== "object") rule.set = {};
  setPath(rule.set, path, value);
}

/** Remove one value from a rule. A rule left with nothing to say is removed too. */
export function deleteInRule(doc: Obj, target: Target, path: string): void {
  const rule = findRule(doc, target);
  if (!rule || !rule.set) return;
  deletePath(rule.set, path);
  if (Object.keys(rule.set).length === 0) {
    doc.rules = doc.rules.filter((r: Obj) => !sameTarget(r.for, target));
    if (doc.rules.length === 0) delete doc.rules;
  }
}

/** Every setting a rule changes, as dotted names with their values. */
export function flatten(set: Obj, prefix = ""): [string, any][] {
  const out: [string, any][] = [];
  for (const [key, value] of Object.entries(set ?? {})) {
    const path = prefix ? `${prefix}.${key}` : key;
    if (value !== null && typeof value === "object" && !Array.isArray(value)) out.push(...flatten(value, path));
    else out.push([path, value]);
  }
  return out;
}
