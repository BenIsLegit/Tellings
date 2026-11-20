<script lang="ts">
  // One setting: its label, its control, and a way to put it back.
  //
  // A narrowing setting (voice, pace, words, picture, colors) is changed for
  // whatever the "This choice covers" bar says, unless `target` pins it.
  import { open } from "@tauri-apps/plugin-dialog";
  import { RotateCcw } from "lucide-svelte";
  import { editor } from "$lib/stores/editor.svelte";
  import type { Target } from "$lib/types";
  import { fileName } from "$lib/utils/format";
  import Button from "./Button.svelte";
  import ColorInput from "./ColorInput.svelte";
  import NumberInput from "./NumberInput.svelte";
  import Select from "./Select.svelte";
  import TextInput from "./TextInput.svelte";
  import Toggle from "./Toggle.svelte";

  let {
    path,
    label,
    help = "",
    kind,
    options = [],
    min = 0,
    max = 100,
    step = 1,
    unit = "",
    slider = true,
    placeholder = "",
    picks = "picture",
    startAt = 1,
    fallback = "#888888",
    target = undefined,
  }: {
    path: string;
    label: string;
    help?: string;
    kind: "toggle" | "number" | "optional-number" | "text" | "select" | "color" | "file";
    options?: [string, string][];
    min?: number;
    max?: number;
    step?: number;
    unit?: string;
    slider?: boolean;
    placeholder?: string;
    /** For kind "file": what the chooser offers. */
    picks?: "picture" | "sound" | "folder";
    /** For kind "optional-number": the value it starts at when first turned on. */
    startAt?: number;
    fallback?: string;
    /** undefined = follow the bar. null = the whole video. An object = that part. */
    target?: Target | null;
  } = $props();

  let at = $derived(target === undefined ? (editor.narrows(path) ? editor.activeTarget : null) : target);
  let scene = $derived(target ? editor.sceneOf(target) : editor.scene);
  let value = $derived(editor.valueFor(path, at, scene));
  let own = $derived(editor.isSetFor(path, at));
  let id = $derived(path.replace(/\./g, "-"));

  const set = (v: unknown) => editor.setFor(path, at, v);
  const reset = () => editor.resetFor(path, at);
  /** Empty text means "no choice", not a choice of nothing. */
  const setText = (v: string) => (v.trim() === "" ? reset() : set(v));

  const FILTERS = {
    picture: [{ name: "Pictures", extensions: ["png", "jpg", "jpeg", "webp", "avif"] }],
    sound: [{ name: "Sound", extensions: ["mp3", "wav", "m4a", "ogg", "flac", "aac"] }],
  };

  async function choose() {
    const picked = picks === "folder" ? await open({ directory: true }) : await open({ multiple: false, filters: FILTERS[picks] });
    if (typeof picked === "string") set(picked);
  }
</script>

<div class="py-2.5" data-testid="setting-{id}">
  <div class="mb-1.5 flex items-center gap-2">
    <span class="text-sm font-medium text-ink">{label}</span>
    {#if own}
      <span class="h-1.5 w-1.5 rounded-full bg-brand" title="Set here" data-testid="setting-{id}-own"></span>
      <button type="button" class="ml-auto flex items-center gap-1 rounded px-1.5 py-0.5 text-xs text-ink-muted hover:bg-well hover:text-ink" title="Put this back to what it was" data-testid="setting-{id}-reset-btn" onclick={reset}>
        <RotateCcw size={12} /> Put back
      </button>
    {/if}
    {#if kind === "toggle"}
      <span class={own ? "" : "ml-auto"}><Toggle checked={!!value} {label} testid="setting-{id}-toggle" onchange={set} /></span>
    {/if}
  </div>

  {#if kind === "number"}
    <NumberInput value={Number(value ?? min)} {min} {max} {step} {unit} {slider} testid="setting-{id}-input" onchange={set} />
  {:else if kind === "optional-number"}
    {#if value === null || value === undefined}
      <Button size="sm" testid="setting-{id}-set-btn" onclick={() => set(startAt)}>Set this</Button>
    {:else}
      <NumberInput value={Number(value)} {min} {max} {step} {unit} {slider} testid="setting-{id}-input" onchange={set} />
    {/if}
  {:else if kind === "text"}
    <TextInput value={value ?? ""} {placeholder} testid="setting-{id}-input" onchange={setText} />
  {:else if kind === "select"}
    <Select value={String(value ?? "")} {options} testid="setting-{id}-select" onchange={set} />
  {:else if kind === "color"}
    <ColorInput value={value ?? null} {fallback} testid="setting-{id}-input" onchange={setText} />
  {:else if kind === "file"}
    <div class="flex min-w-0 items-center gap-2">
      <span class="min-w-0 flex-1 truncate rounded-md border border-edge bg-well px-3 py-2 text-sm {value ? 'text-ink' : 'text-ink-faint'}" title={value ?? ""} data-testid="setting-{id}-file">
        {value ? fileName(String(value)) : placeholder || "None chosen"}
      </span>
      <Button testid="setting-{id}-choose-btn" onclick={choose}>Choose…</Button>
    </div>
  {/if}

  {#if help}<p class="mt-1.5 text-xs leading-relaxed text-ink-muted">{help}</p>{/if}
</div>
