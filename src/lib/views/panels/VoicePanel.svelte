<script lang="ts">
  import Button from "$lib/components/Button.svelte";
  import Group from "$lib/components/Group.svelte";
  import ScopeBar from "$lib/components/ScopeBar.svelte";
  import Setting from "$lib/components/Setting.svelte";
  import VoicePicker from "$lib/components/VoicePicker.svelte";
  import { editor } from "$lib/stores/editor.svelte";
  import { commas } from "$lib/utils/format";

  let picking = $state<"voice.name" | "voice.respond" | null>(null);
  let est = $derived(editor.look?.estimate);
  let current = $derived(picking ? String(editor.valueFor(picking, editor.activeTarget) ?? "") : "");
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
    <Setting path="voice.stability" label="Stability" kind="optional-number" min={0} max={1} step={0.05} startAt={0.5} help="An ElevenLabs setting. Leave unset to use the voice's own." />
    <Setting path="voice.similarity" label="Similarity" kind="optional-number" min={0} max={1} step={0.05} startAt={0.75} />
    <Setting path="voice.style" label="Style" kind="optional-number" min={0} max={1} step={0.05} startAt={0} />
    <Setting path="voice.model" label="ElevenLabs model" kind="text" />
    <Setting path="voice.id" label="Exact voice id" kind="text" placeholder="None. The name is used." />
  </Group>

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
    if (picking) editor.setFor(picking, editor.activeTarget, voice.name);
    picking = null;
  }}
/>
