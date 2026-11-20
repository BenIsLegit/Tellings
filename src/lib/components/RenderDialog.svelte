<script lang="ts">
  // Making the video. Before anything starts, this says what will be made and
  // what it will cost. Credits are only spent after the person presses the
  // button that names the number.
  import { listen } from "@tauri-apps/api/event";
  import { CircleAlert, CircleCheck, FolderOpen, Loader2, Play } from "lucide-svelte";
  import { api } from "$lib/api";
  import type { Credits, Doc, Look, Made, Progress } from "$lib/types";
  import { clock, commas, fileName } from "$lib/utils/format";
  import { invokeWithToast } from "$lib/utils/errors";
  import Button from "./Button.svelte";
  import Modal from "./Modal.svelte";

  let { open, doc, onclose }: { open: boolean; doc: Doc | null; onclose: () => void } = $props();

  let phase = $state<"checking" | "ready" | "working" | "done" | "failed">("checking");
  let look = $state<Look | null>(null);
  let credits = $state<Credits | null>(null);
  let progress = $state<Progress | null>(null);
  let made = $state<Made | null>(null);
  let problem = $state("");

  let est = $derived(look?.estimate);
  let cost = $derived(est?.creditsToRecord ?? 0);
  let blocked = $derived(cost > 0 && !look?.key ? "No ElevenLabs API key is saved yet. Add it under Settings, or turn the voice off." : (est?.missingRecordings.length ?? 0) > 0 ? `These recordings could not be found: ${est?.missingRecordings.join(", ")}` : "");
  const KIND: Record<string, string> = { video: "Video", sound: "Sound only", captions: "Captions", guide: "Tap-through guide" };

  async function check() {
    if (!doc) return;
    phase = "checking";
    look = null;
    credits = null;
    made = null;
    progress = null;
    try {
      look = (await api.setupLook(doc)).look;
      if (look.estimate.lettersToRecord > 0 && look.key) {
        // What each model charges, asked of ElevenLabs (free, kept for a day), so
        // the number on the button is the number it will come to.
        const asked = await api.models().catch(() => null);
        if (asked) look = (await api.setupLook(doc)).look;
      }
      phase = "ready";
      if (look.estimate.lettersToRecord > 0 && look.key) credits = await api.credits().catch(() => null);
    } catch (e) {
      problem = String(e);
      phase = "failed";
    }
  }

  // Check afresh every time the dialog opens.
  let wasOpen = false;
  $effect(() => {
    if (open && !wasOpen) check();
    wasOpen = open;
  });

  $effect(() => {
    const unlisten = listen<Progress>("render-progress", (e) => {
      progress = e.payload;
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  });

  async function start() {
    if (!doc) return;
    phase = "working";
    progress = null;
    try {
      // The number approved is exactly the number shown on the button.
      made = await api.renderStart(doc, cost);
      phase = "done";
    } catch (e) {
      problem = String(e);
      phase = "failed";
    }
  }
</script>

<Modal {open} title={phase === "done" ? "Your video is ready" : "Make the video"} wide closable={phase !== "working"} testid="render-dialog" {onclose}>
  {#if phase === "checking"}
    <div class="flex items-center justify-center py-12" data-testid="render-loading">
      <Loader2 size={24} class="animate-spin text-ink-muted" />
      <span class="ml-3 text-sm text-ink-muted">Working out what this will take…</span>
    </div>
  {:else if phase === "ready" && look}
    <div class="space-y-4 text-base" data-testid="render-summary">
      <div>
        <p class="text-xl font-semibold text-ink">{look.plan.name}</p>
        <p class="mt-1 text-ink-secondary">{look.plan.prayerCount} prayers · about {clock(look.plan.seconds)} · {look.plan.stage.width} x {look.plan.stage.height}</p>
      </div>

      <div class="rounded-lg border p-4 {cost > 0 ? 'border-amber-700/60 bg-amber-900/20' : 'border-edge bg-canvas'}" data-testid="render-cost">
        {#if cost > 0}
          <p class="font-semibold text-ink">This will spend about {commas(cost)} ElevenLabs credits.</p>
          <p class="mt-1 text-sm text-ink-secondary">{est?.piecesToRecord === 1 ? "1 new piece" : `${est?.piecesToRecord} new pieces`} of speech to record. Each is recorded once and kept, so making this again is free.</p>
          {#each est?.byVoice ?? [] as v (v.voice)}
            <p class="mt-1 text-sm text-ink-secondary">{v.voice}: {commas(v.letters)} letters{v.credits === v.letters ? "" : `, ${commas(v.credits)} credits`}</p>
          {/each}
          {#if est?.models.length}<p class="mt-1 text-sm text-ink-secondary" data-testid="render-models">Model: {est.models.join(", ")}</p>{/if}
          {#if est?.ratesGuessed}<p class="mt-1 text-sm text-amber-400" data-testid="render-cost-guess">This number is a guess. ElevenLabs has not said what this model charges. The making stops before it goes over.</p>{/if}
          {#if credits}<p class="mt-2 text-sm text-ink-secondary" data-testid="render-credits-left">You have {commas(Math.max(0, credits.limit - credits.used))} credits left.</p>{/if}
        {:else}
          <p class="font-semibold text-ink">No credits will be spent.</p>
          <p class="mt-1 text-sm text-ink-secondary">{(est?.lettersKept ?? 0) > 0 ? "Every piece of speech is already recorded and kept." : "This one has no generated voice."}</p>
        {/if}
      </div>

      <div data-testid="render-files">
        <p class="text-sm font-medium text-ink">It will write</p>
        {#each look.files as f (f)}<p class="truncate text-sm text-ink-muted" title={f}>{f}</p>{/each}
      </div>

      {#each look.plan.warnings as w (w)}
        <p class="flex gap-2 text-sm text-amber-400" data-testid="render-warning"><CircleAlert size={16} class="mt-0.5 shrink-0" /> {w}</p>
      {/each}
      {#if blocked}
        <div class="rounded-lg border border-red-900/50 bg-red-900/20 p-4" data-testid="render-blocked"><p class="text-sm text-red-400">{blocked}</p></div>
      {/if}
    </div>
  {:else if phase === "working"}
    <div class="py-6" data-testid="render-progress">
      <p class="text-lg font-medium text-ink">{progress?.message ?? "Starting…"}</p>
      <div class="mt-4 h-3 overflow-hidden rounded-full bg-well">
        <div class="h-full rounded-full bg-brand transition-all duration-300" style="width: {Math.round((progress?.fraction ?? 0) * 100)}%"></div>
      </div>
      <p class="mt-2 text-sm text-ink-muted">{Math.round((progress?.fraction ?? 0) * 100)}% · A whole rosary takes a couple of minutes.</p>
    </div>
  {:else if phase === "done" && made}
    <div class="space-y-4 text-base" data-testid="render-done">
      <p class="flex items-center gap-2 text-ink"><CircleCheck size={20} class="text-emerald-400" /> {made.name}: {clock(made.seconds)} long, made in {clock(made.took)}.</p>
      <ul class="divide-y divide-edge overflow-hidden rounded-lg border border-edge">
        {#each made.files as f (f.path)}
          <li class="flex items-center gap-3 px-4 py-3" data-testid="render-file-item">
            <span class="min-w-0 flex-1">
              <span class="block text-sm font-medium text-ink">{KIND[f.kind] ?? f.kind}</span>
              <span class="block truncate text-xs text-ink-muted" title={f.path}>{fileName(f.path)}</span>
            </span>
            <Button size="sm" testid="render-open-btn" onclick={() => invokeWithToast(() => api.openPath(f.path))}><Play size={14} /> Open</Button>
            <Button size="sm" variant="ghost" title="Show in its folder" testid="render-reveal-btn" onclick={() => invokeWithToast(() => api.revealPath(f.path))}><FolderOpen size={14} /></Button>
          </li>
        {/each}
      </ul>
      {#if made.lettersRecorded > 0}<p class="text-sm text-ink-secondary" data-testid="render-spent">Recorded {commas(made.lettersRecorded)} letters of new speech. Spent {commas(made.creditsSpent)} credits.</p>{/if}
      {#if made.hailMarySeconds}<p class="text-sm text-ink-secondary" data-testid="render-pace">One Hail Mary takes {made.hailMarySeconds.toFixed(1)} seconds. In the first video it took about 9.2. Listen before you trust a new voice.</p>{/if}
      {#each made.warnings as w (w)}
        <p class="flex gap-2 text-sm text-amber-400"><CircleAlert size={16} class="mt-0.5 shrink-0" /> {w}</p>
      {/each}
    </div>
  {:else if phase === "failed"}
    <div class="rounded-lg border border-red-900/50 bg-red-900/20 p-4" data-testid="render-error"><p class="text-sm leading-relaxed whitespace-pre-wrap text-red-400">{problem}</p></div>
  {/if}

  {#snippet actions()}
    {#if phase === "ready"}
      <Button testid="render-cancel-btn" onclick={onclose}>Not now</Button>
      <Button variant="primary" disabled={!!blocked} testid="render-start-btn" onclick={start}>
        {cost > 0 ? `Spend about ${commas(cost)} credits and make it` : "Make it"}
      </Button>
    {:else if phase === "working"}
      <Button variant="danger" testid="render-stop-btn" onclick={() => api.renderCancel()}>Stop</Button>
    {:else if phase === "failed"}
      <Button testid="render-close-btn" onclick={onclose}>Close</Button>
      <Button variant="primary" testid="render-retry-btn" onclick={check}>Try again</Button>
    {:else if phase === "done"}
      <Button variant="primary" testid="render-close-btn" onclick={onclose}>Done</Button>
    {/if}
  {/snippet}
</Modal>
