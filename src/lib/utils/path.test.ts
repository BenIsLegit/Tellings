// Run with: npm test
import assert from "node:assert/strict";
import { test } from "node:test";
import { deleteInRule, deletePath, findRule, flatten, getPath, hasPath, sameTarget, setInRule, setPath } from "./path.ts";

test("changing one thing touches one thing", () => {
  const doc: Record<string, any> = { voice: { name: "A", speed: 1 } };
  setPath(doc, "voice.name", "B");
  setPath(doc, "screen.safe.top", 120);
  assert.deepEqual(doc, { voice: { name: "B", speed: 1 }, screen: { safe: { top: 120 } } });
  assert.equal(getPath(doc, "screen.safe.top"), 120);
  assert.equal(getPath(doc, "screen.nothing.here"), undefined);
  assert.equal(hasPath(doc, "voice.speed"), true);
  assert.equal(hasPath(doc, "voice.id"), false);
});

test("putting a setting back leaves no empty groups behind", () => {
  const doc: Record<string, any> = { voice: { name: "B" }, screen: { safe: { top: 120 } } };
  deletePath(doc, "screen.safe.top");
  assert.deepEqual(doc, { voice: { name: "B" } });
  deletePath(doc, "not.there");
  assert.deepEqual(doc, { voice: { name: "B" } });
});

test("a setting for part of the video becomes a rule", () => {
  const doc: Record<string, any> = {};
  setInRule(doc, { mystery: 3 }, "voice.name", "Bella");
  setInRule(doc, { mystery: 3 }, "voice.speed", 1.1);
  setInRule(doc, { prayer: "ourfather" }, "voice.name", "George");
  assert.equal(doc.rules.length, 2, "the same part of the video reuses its rule");
  assert.deepEqual(findRule(doc, { mystery: 3 })?.set, { voice: { name: "Bella", speed: 1.1 } });

  deleteInRule(doc, { mystery: 3 }, "voice.speed");
  assert.deepEqual(findRule(doc, { mystery: 3 })?.set, { voice: { name: "Bella" } });
  deleteInRule(doc, { mystery: 3 }, "voice.name");
  assert.equal(findRule(doc, { mystery: 3 }), null, "an empty rule is removed");
  deleteInRule(doc, { prayer: "ourfather" }, "voice.name");
  assert.equal("rules" in doc, false, "and so is an empty list");
});

test("parts of the video are told apart exactly", () => {
  assert.equal(sameTarget({ mystery: 3 }, { mystery: 3 }), true);
  assert.equal(sameTarget({ mystery: 3 }, { mystery: 3, prayer: "hailmary" }), false);
  assert.equal(sameTarget({}, null), true);
});

test("a rule's changes can be listed", () => {
  assert.deepEqual(flatten({ voice: { name: "Bella" }, pace: { after: 1 } }), [
    ["voice.name", "Bella"],
    ["pace.after", 1],
  ]);
});
