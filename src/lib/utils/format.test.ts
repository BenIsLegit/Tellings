import assert from "node:assert/strict";
import { test } from "node:test";
import { joinVoices, splitVoices } from "./format.ts";

test("several voices share one setting and take turns", () => {
  assert.deepEqual(splitVoices("Imperial - Male"), ["Imperial - Male"]);
  assert.deepEqual(splitVoices("Imperial - Male + Bella"), ["Imperial - Male", "Bella"]);
  // A comma or a dash inside a name is part of the name.
  assert.deepEqual(splitVoices("Roger - Laid-Back, Casual +  Bella "), ["Roger - Laid-Back, Casual", "Bella"]);
  assert.deepEqual(splitVoices(""), []);
  assert.deepEqual(splitVoices(null), []);
  assert.equal(joinVoices(["Ann", "Ben", "Cat"]), "Ann + Ben + Cat");
  assert.deepEqual(splitVoices(joinVoices(["Ann", "Ben"])), ["Ann", "Ben"]);
});
