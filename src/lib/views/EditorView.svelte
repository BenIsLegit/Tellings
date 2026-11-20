<script lang="ts">
  import { save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { ChevronLeft, ChevronRight, Clapperboard, Save, Share2 } from "lucide-svelte";
  import type { Component } from "svelte";
  import { api } from "$lib/api";
  import Badge from "$lib/components/Badge.svelte";
  import Button from "$lib/components/Button.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import Preview from "$lib/components/Preview.svelte";
  import RenderDialog from "$lib/components/RenderDialog.svelte";
  import Select from "$lib/components/Select.svelte";
  import { editor } from "$lib/stores/editor.svelte";
  import type { Doc } from "$lib/types";
  import { invokeWithToast } from "$lib/utils/errors";
  import { clock } from "$lib/utils/format";
  import BeadsPanel from "./panels/BeadsPanel.svelte";
  import ColorsPanel from "./panels/ColorsPanel.svelte";
  import OutputPanel from "./panels/OutputPanel.svelte";
  import PacePanel from "./panels/PacePanel.svelte";
  import PartsPanel from "./panels/PartsPanel.svelte";
  import PicturesPanel from "./panels/PicturesPanel.svelte";
  import PrayersPanel from "./panels/PrayersPanel.svelte";
  import ScreenPanel from "./panels/ScreenPanel.svelte";
  import VoicePanel from "./panels/VoicePanel.svelte";
  import WordsPanel from "./panels/WordsPanel.svelte";

  const tabs: { key: string; label: string; panel: Component }[] = [
    { key: "prayers", label: "Prayers", panel: PrayersPanel },
    { key: "voice", label: "Voice", panel: VoicePanel },
    { key: "pictures", label: "Pictures", panel: PicturesPanel },
    { key: "beads", label: "Beads", panel: BeadsPanel },
    { key: "words", label: "Words", panel: WordsPanel },
    { key: "colors", label: "Colors", panel: ColorsPanel },
    { key: "screen", label: "Screen", panel: ScreenPanel },
    { key: "pace", label: "Pace", panel: PacePanel },
    { key: "output", label: "What comes out", panel: OutputPanel },
    { key: "parts", label: "Part choices", panel: PartsPanel },
  ];
  let tab = $state("prayers");
  let Panel = $derived(tabs.find((t) => t.key === tab)?.panel ?? PrayersPanel);

  let scene = $derived(editor.scene);
  let count = $derived(editor.plan?.scenes.length ?? 0);
  let chapters = $derived<[string, string][]>((editor.plan?.scenes ?? []).flatMap((s, i) => (s.chapter ? [[String(i), s.chapter] as [string, string]] : [])));
  let chapterNow = $derived.by(() => {
    let last = "0";
    for (const [i] of chapters) if (Number(i) <= editor.index) last = i;
    return last;
  });
  let ownRules = $derived(Array.isArray(editor.doc.rules) ? editor.doc.rules.length : 0);

  // Saving
  let naming = $state(false);
  let newName = $state("");
  let isPreset = $derived(editor.entry?.kind === "preset");

  function askName() {
    newName = isPreset ? `My ${editor.entry?.name ?? "setup"}` : editor.name;
    naming = true;
  }
  async function saveAs() {
    const saved = await invokeWithToast(() => editor.save(newName), `Saved “${newName.trim()}”`);
    if (saved) naming = false;
  }
  async function save() {
    if (isPreset) return askName();
    await invokeWithToast(() => editor.save(editor.name), "Saved");
  }
  async function share() {
    const path = await saveDialog({ defaultPath: `${editor.name || "setup"}.json`, filters: [{ name: "Setup", extensions: ["json"] }] });
    if (typeof path === "string") await invokeWithToast(() => api.setupExport($state.snapshot(editor.doc), path), "Setup written. Send that file to share it.");
  }

  // Making
  let making = $state<Doc | null>(null);

  function keys(e: KeyboardEvent) {
    const el = e.target as HTMLElement | null;
    if (el && ["INPUT", "TEXTAREA", "SELECT"].includes(el.tagName)) return;
    if (making || naming) return;
    if (e.key === "ArrowRight") editor.go(editor.index + 1);
    if (e.key === "ArrowLeft") editor.go(editor.index - 1);
  }
</script>

<svelte:window onkeydown={keys} />

<div class="flex h-full flex-col" data-testid="editor-view">
  <header class="flex items-center gap-3 border-b border-edge bg-panel px-5 py-3" data-testid="editor-header">
    <div class="min-w-0 flex-1">
      <div class="flex items-center gap-2">
        <h1 class="truncate text-xl font-bold text-ink" data-testid="editor-title">{editor.name || "Untitled"}</h1>
        {#if isPreset}<Badge tone="quiet" testid="editor-preset-badge">Preset</Badge>{/if}
        {#if editor.dirty}<Badge tone="warn" testid="editor-dirty-badge">Not saved</Badge>{/if}
      </div>
      <p class="truncate text-xs text-ink-muted" data-testid="editor-subtitle">
        {#if editor.plan}{editor.plan.setName} Mysteries · {editor.plan.prayerCount} prayers · about {clock(editor.plan.seconds)}{/if}
        {#if editor.startsFrom && !isPreset} · starts from “{editor.startsFrom}”{/if}
      </p>
    </div>
    <Button title="Write this setup to a file to share" testid="editor-share-btn" onclick={share}><Share2 size={16} /> Share</Button>
    {#if !isPreset}<Button testid="editor-save-as-btn" onclick={askName}>Save as…</Button>{/if}
    <Button testid="editor-save-btn" onclick={save}><Save size={16} /> {isPreset ? "Save as my own…" : "Save"}</Button>
    <Button variant="primary" size="lg" disabled={!!editor.problem || !editor.plan} testid="editor-make-btn" onclick={() => (making = $state.snapshot(editor.doc))}>
      <Clapperboard size={18} /> Make the video
    </Button>
  </header>

  <div class="flex min-h-0 flex-1">
    <nav class="w-40 shrink-0 space-y-1 overflow-y-auto border-r border-edge bg-panel p-2" data-testid="editor-tabs">
      {#each tabs as t (t.key)}
        <button
          type="button"
          data-testid="editor-tab-{t.key}-btn"
          onclick={() => (tab = t.key)}
          class="flex w-full items-center justify-between rounded-md px-3 py-2 text-left text-sm transition-colors {tab === t.key ? 'bg-brand/10 font-medium text-brand' : 'text-ink-muted hover:bg-well/50 hover:text-ink-secondary'}"
        >
          {t.label}
          {#if t.key === "parts" && ownRules > 0}<span class="rounded-full bg-brand/15 px-1.5 text-xs text-brand">{ownRules}</span>{/if}
        </button>
      {/each}
    </nav>

    <div class="w-[26rem] shrink-0 overflow-y-auto border-r border-edge bg-panel px-4 py-3" data-testid="editor-panel">
      {#if editor.problem}
        <div class="mb-3 rounded-lg border border-red-900/50 bg-red-900/20 p-3" data-testid="editor-problem">
          <p class="text-sm leading-relaxed text-red-400">{editor.problem}</p>
        </div>
      {/if}
      {#if editor.setup}
        <Panel />
      {/if}
    </div>

    <div class="flex min-w-0 flex-1 flex-col bg-canvas" data-testid="editor-preview">
      <div class="min-h-0 flex-1 p-5">
        {#if editor.plan}
          <Preview plan={editor.plan} index={editor.index} onbead={(bead) => editor.goToBead(bead)} />
        {/if}
      </div>
      <div class="border-t border-edge bg-panel px-5 py-3" data-testid="editor-navigator">
        <div class="flex items-center gap-3">
          <Button title="Back one bead (left arrow)" testid="editor-prev-btn" onclick={() => editor.go(editor.index - 1)}><ChevronLeft size={18} /></Button>
          <input
            type="range"
            min="0"
            max={Math.max(0, count - 1)}
            value={editor.index}
            data-testid="editor-scene-slider"
            oninput={(e) => editor.go(Number(e.currentTarget.value))}
            class="h-1.5 min-w-0 flex-1 cursor-pointer appearance-none rounded-full bg-frame accent-brand"
          />
          <Button title="On one bead (right arrow)" testid="editor-next-btn" onclick={() => editor.go(editor.index + 1)}><ChevronRight size={18} /></Button>
          <div class="w-52 shrink-0"><Select value={chapterNow} options={chapters} testid="editor-chapter-select" onchange={(v) => editor.go(Number(v))} /></div>
        </div>
        <p class="mt-2 truncate text-sm text-ink-secondary" data-testid="editor-scene-label">
          {#if scene}
            {scene.number ? `Prayer ${scene.number} of ${editor.plan?.prayerCount}` : scene.kind === "intro" ? "Opening title" : "Ending"} · {scene.title}
            {#if scene.voice.on && scene.kind === "prayer"}<span class="text-ink-muted"> · voice: {scene.muted ? "dropped out" : scene.voice.recording ? "your recording" : scene.voice.name}</span>{/if}
          {/if}
          <span class="text-ink-faint"> · click a bead to jump to it</span>
        </p>
      </div>
    </div>
  </div>
</div>

<Modal open={naming} title="Save this setup" testid="save-dialog" onclose={() => (naming = false)}>
  <p class="mb-3 text-base">Give it a name. You can open it again from Setups.</p>
  <input
    type="text"
    bind:value={newName}
    placeholder="A name"
    spellcheck="false"
    data-testid="save-name-input"
    onkeydown={(e) => {
      if (e.key === "Enter" && newName.trim()) saveAs();
    }}
    class="w-full rounded-md border border-frame bg-well px-3 py-2 text-base text-ink placeholder-ink-faint focus:border-brand focus:outline-none"
  />
  {#if isPreset}<p class="mt-3 text-sm text-ink-muted">It keeps only what you changed, on top of the “{editor.entry?.name}” preset.</p>{/if}
  {#snippet actions()}
    <Button testid="save-cancel-btn" onclick={() => (naming = false)}>Cancel</Button>
    <Button variant="primary" disabled={!newName.trim()} testid="save-submit-btn" onclick={saveAs}>Save</Button>
  {/snippet}
</Modal>

<RenderDialog open={making !== null} doc={making} onclose={() => (making = null)} />
