import assert from "node:assert/strict";
import { test } from "node:test";
import { joinVoices, rateWords, splitVoices } from "./format.ts";

test("what a model charges is said in words", () => {
  assert.equal(rateWords(1), "1 credit a letter");
  assert.equal(rateWords(0.5), "half a credit a letter");
  assert.equal(rateWords(2), "2 credits a letter");
  assert.equal(rateWords(0.3333), "0.33 credits a letter");
});

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
