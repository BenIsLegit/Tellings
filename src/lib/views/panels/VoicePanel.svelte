<script lang="ts">
  import { X } from "lucide-svelte";
  import { api } from "$lib/api";
  import Button from "$lib/components/Button.svelte";
  import Group from "$lib/components/Group.svelte";
  import ScopeBar from "$lib/components/ScopeBar.svelte";
  import Setting from "$lib/components/Setting.svelte";
  import VoicePicker from "$lib/components/VoicePicker.svelte";
  import { confirmDialog } from "$lib/stores/confirm";
  import { editor } from "$lib/stores/editor.svelte";
  import type { Heard } from "$lib/types";
  import { invokeWithToast } from "$lib/utils/errors";
  import { commas, joinVoices, splitVoices } from "$lib/utils/format";

  /** What the voice chooser is open for: the voice, one more to take turns, or the one that answers. */
  let picking = $state<"voice.name" | "voice.respond" | "turn" | null>(null);
  let est = $derived(editor.look?.estimate);

  // Several voices can share the one setting, joined with " + ". Then they take turns.
  let names = $derived(splitVoices(editor.valueFor("voice.name", editor.activeTarget)));
  let current = $derived(picking === "voice.respond" ? String(editor.valueFor("voice.respond", editor.activeTarget) ?? "") : picking === "voice.name" ? (names[0] ?? "") : "");
  let fresh = $derived(!!editor.valueFor("voice.freshEachTime", editor.activeTarget));

  function setNames(list: string[]) {
    if (list.length === 0) editor.resetFor("voice.name", editor.activeTarget);
    else editor.setFor("voice.name", editor.activeTarget, joinVoices(list));
  }

  // What is heard on the bead the preview is on.
  let scene = $derived(editor.scene);
  let heard = $derived(editor.look?.speech?.[editor.index] ?? []);
  let redoable = $derived(heard.filter((h) => !h.ownRecording && h.kept));

  /** One piece of speech in plain words: "Hail Mary · Bella · take 2 of 3 · on 18 beads · recorded". */
  function line(h: Heard): string {
    const parts = [h.part === "announce" ? "The mystery's name" : (scene?.title ?? "The prayer"), h.voice];
    if (h.takes > 1) parts.push(`take ${h.take} of ${h.takes}`);
    if (h.uses > 1) parts.push(`on ${h.uses} beads`);
    parts.push(h.ownRecording ? "your recording" : h.kept ? "recorded" : "not recorded yet");
    return parts.join(" · ");
  }

  async function recordAgain() {
    const letters = redoable.reduce((n, h) => n + h.letters, 0);
    const one = redoable.length === 1;
    const most = Math.max(...redoable.map((h) => h.uses));
    const ok = await confirmDialog({
      title: "Record this again?",
      message:
        (one ? "This recording will be thrown away." : `The ${redoable.length} recordings heard on this bead will be thrown away.`) +
        (most > 1 ? (one ? ` It plays on ${most} beads in this video.` : " Some of them play on other beads too.") : "") +
        ` Nothing is spent now. The next time you make the video ${one ? "it is" : "they are"} recorded afresh, for about ${commas(letters)} credits.`,
      confirmLabel: one ? "Throw it away" : "Throw them away",
      variant: "danger",
    });
    if (!ok) return;
    await invokeWithToast(() => api.recordAgain($state.snapshot(editor.doc), editor.index), "It will be recorded afresh next time");
    await editor.refresh();
  }
</script>

<div data-testid="voice-panel">
  <ScopeBar />

  <Group title="The voice" testid="voice-main-group">
    <Setting path="voice.on" label="Use a voice" kind="toggle" help="Off means words on screen only. No credits are spent." />
    <Setting path="voice.name" label="Voice" kind="text" placeholder="The name of a voice in your ElevenLabs account" />
    <div class="-mt-1 pb-2">
      <Button size="sm" testid="voice-choose-btn" onclick={() => (picking = "voice.name")}>Choose from my voices…</Button>
    </div>
    <Setting path="voice.speed" label="Speed" kind="number" min={0.6} max={1.6} step={0.05} unit="x" help="1 is the pace of the first video. Lower is slower." />
    <Setting path="voice.announce" label="Say the name of each mystery" kind="toggle" />
  </Group>

  <Group title="Variety" testid="voice-variety-group" note="A prayer said many times does not have to sound the same every time. Pick what a choice covers in the bar above: “Every Hail Mary” is a good place to start.">
    <Setting
      path="voice.takes"
      label="Recordings of each prayer"
      kind="number"
      min={1}
      max={10}
      step={1}
      unit="takes"
      help="1 is one recording, used every time, as in the first video. With 5, each prayer is recorded five times and the five take turns. Extra takes cost credits once, then are kept."
    />
    <Setting path="voice.freshEachTime" label="A new recording every time" kind="toggle" help="No two prayers sound the same. This costs the most: a whole rosary is about 16,000 credits." />
    {#if !fresh}
      <Setting path="voice.shuffle" label="Mix up the order" kind="toggle" help="Takes and voices come in a mixed order instead of in turn. The same one never comes twice running." />
    {/if}

    <div class="py-2.5" data-testid="voice-turns">
      <span class="text-sm font-medium text-ink">Voices that take turns</span>
      {#if names.length > 1}
        <ul class="mt-1.5 divide-y divide-edge overflow-hidden rounded-md border border-edge" data-testid="voice-turns-list">
          {#each names as name, i (i)}
            <li class="flex items-center gap-2 px-3 py-1.5" data-testid="voice-turns-item">
              <span class="w-4 text-xs text-ink-faint tabular-nums">{i + 1}</span>
              <span class="min-w-0 flex-1 truncate text-sm text-ink">{name}</span>
              <button
                type="button"
                class="rounded p-1 text-ink-muted hover:bg-well hover:text-ink"
                title="Take {name} out"
                data-testid="voice-turns-remove-btn"
                onclick={() => setNames(names.filter((_, k) => k !== i))}
              >
                <X size={14} />
              </button>
            </li>
          {/each}
        </ul>
      {:else}
        <p class="mt-1 text-xs leading-relaxed text-ink-muted" data-testid="voice-turns-empty-state">One voice says everything here. Add another and they take turns.</p>
      {/if}
      <div class="mt-2">
        <Button size="sm" testid="voice-turns-add-btn" onclick={() => (picking = "turn")}>Add a voice…</Button>
      </div>
    </div>
    {#if names.length > 1}
      <Setting
        path="voice.turnEvery"
        label="The next voice takes over"
        kind="select"
        options={[
          ["prayer", "On every prayer"],
          ["decade", "On every decade"],
        ]}
        help="By the decade, the opening prayers and the closing prayers count as a turn each."
      />
    {/if}
  </Group>

  <Group title="Praying together" testid="voice-together-group">
    <Setting path="voice.respond" label="A second voice that answers" kind="text" placeholder="None" help="It says the second half of the Our Father, Hail Mary and Glory Be, the way a group prays." />
    <div class="-mt-1 pb-2">
      <Button size="sm" testid="voice-respond-choose-btn" onclick={() => (picking = "voice.respond")}>Choose from my voices…</Button>
    </div>
    <Setting path="voice.dropAfter" label="The voice drops out after" kind="number" min={0} max={9} slider={false} unit="Hail Marys" help="0 means it says them all. 3 means it says the first three of each decade, then leaves the rest to you." />
  </Group>

  <Group title="Your own recordings" folded testid="voice-recordings-group" note="To use your own recording of one prayer, pick “Every Hail Mary” (or another prayer) in the bar above first, then choose the file.">
    <Setting path="voice.recording" label="Recording of the prayer" kind="file" picks="sound" />
    <Setting path="voice.announceRecording" label="Recording of the mystery's name" kind="file" picks="sound" />
    <Setting path="voice.recordingSpeed" label="Speed of your recordings" kind="number" min={0.5} max={2.5} step={0.05} unit="x" help="1 leaves them as they are. The first video's recordings were played at 1.75." />
  </Group>

  <Group title="Fine tuning" folded testid="voice-fine-group">
    <Setting path="voice.exactSpeedup" label="Exact speed-up" kind="optional-number" min={0.5} max={4} step={0.05} startAt={1.75} unit="x" help="Skips the measuring and speeds the recording up by exactly this much." />
    <Setting path="voice.stability" label="Stability" kind="optional-number" min={0} max={1} step={0.05} startAt={0.5} help="An ElevenLabs setting. Leave unset to use the voice's own. Lower makes one take differ more from the next." />
    <Setting path="voice.similarity" label="Similarity" kind="optional-number" min={0} max={1} step={0.05} startAt={0.75} />
    <Setting path="voice.style" label="Style" kind="optional-number" min={0} max={1} step={0.05} startAt={0} />
    <Setting path="voice.model" label="ElevenLabs model" kind="text" />
    <Setting path="voice.id" label="Exact voice id" kind="text" placeholder="None. The name is used." help="One exact voice. It leads alone, even when several names are listed above." />
  </Group>

  {#if heard.length > 0}
    <div class="mt-2 rounded-md border border-edge bg-canvas p-3 text-sm leading-relaxed" data-testid="voice-heard">
      <p class="font-medium text-ink">Heard on this bead</p>
      <ul class="mt-1 space-y-0.5">
        {#each heard as h, i (i)}
          <li class="text-ink-muted" data-testid="voice-heard-item">{line(h)}</li>
        {/each}
      </ul>
      <div class="mt-2">
        <Button size="sm" disabled={redoable.length === 0} testid="voice-record-again-btn" onclick={recordAgain}>Record this again…</Button>
      </div>
      <p class="mt-1.5 text-xs text-ink-muted">For a recording that came out wrong. It is thrown away and made afresh the next time.</p>
    </div>
  {/if}

  {#if est}
    <div class="mt-2 rounded-md border border-edge bg-canvas p-3 text-sm leading-relaxed" data-testid="voice-cost">
      {#if est.lettersToRecord > 0}
        <p class="font-medium text-ink">New speech to record: {commas(est.lettersToRecord)} letters</p>
        <p class="text-ink-muted">About {commas(est.lettersToRecord)} ElevenLabs credits. You will be asked before anything is spent.</p>
      {:else}
        <p class="font-medium text-ink">Nothing new to record</p>
        <p class="text-ink-muted">{est.lettersKept > 0 ? "Every piece of speech is already recorded and kept." : "This video has no generated voice."}</p>
      {/if}
    </div>
  {/if}
</div>

<VoicePicker
  open={picking !== null}
  {current}
  onclose={() => (picking = null)}
  onpick={(voice) => {
    if (picking === "turn") setNames(names.includes(voice.name) ? names : [...names, voice.name]);
    else if (picking === "voice.name") setNames([voice.name, ...names.slice(1).filter((n) => n !== voice.name)]);
    else if (picking) editor.setFor(picking, editor.activeTarget, voice.name);
    picking = null;
  }}
/>
