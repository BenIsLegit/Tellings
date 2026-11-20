<script lang="ts">
  // Pick the ElevenLabs model that turns the words into speech. Listing models is free.
  import { Loader2, RefreshCw } from "lucide-svelte";
  import { api } from "$lib/api";
  import type { ModelInfo } from "$lib/types";
  import { commas, rateWords } from "$lib/utils/format";
  import Badge from "./Badge.svelte";
  import Button from "./Button.svelte";
  import Modal from "./Modal.svelte";

  let { open, current = "", onpick, onclose }: { open: boolean; current?: string; onpick: (model: ModelInfo) => void; onclose: () => void } = $props();

  let list = $state<ModelInfo[]>([]);
  let loading = $state(false);
  let problem = $state<string | null>(null);

  async function load(refresh: boolean) {
    loading = true;
    problem = null;
    try {
      list = await api.models(refresh);
    } catch (e) {
      problem = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (open && list.length === 0 && !loading && !problem) load(false);
  });

  /** "Up to 10,000 letters a prayer · 29 languages" */
  function facts(m: ModelInfo): string {
    const parts: string[] = [];
    if (m.maxLetters > 0) parts.push(`Up to ${commas(m.maxLetters)} letters a prayer`);
    if (m.languages.length > 0) parts.push(m.languages.length === 1 ? "1 language" : `${m.languages.length} languages`);
    return parts.join(" · ");
  }
</script>

<Modal {open} title="Choose a model" wide testid="model-picker" {onclose}>
  <div class="mb-3 flex items-start gap-3">
    <p class="min-w-0 flex-1 text-sm leading-relaxed text-ink-muted">The model is what turns the words into speech. A prayer recorded with one model is recorded again, and paid for again, with another.</p>
    <Button title="Fetch the list again" testid="model-picker-refresh-btn" onclick={() => load(true)}><RefreshCw size={16} /> Refresh</Button>
  </div>
  {#if loading}
    <div class="flex items-center justify-center py-12" data-testid="model-picker-loading">
      <Loader2 size={24} class="animate-spin text-ink-muted" />
      <span class="ml-3 text-sm text-ink-muted">Asking ElevenLabs for its models…</span>
    </div>
  {:else if problem}
    <div class="rounded-lg border border-red-900/50 bg-red-900/20 p-4" data-testid="model-picker-error">
      <p class="text-sm text-red-400">{problem}</p>
    </div>
  {:else if list.length === 0}
    <p class="py-8 text-center text-sm text-ink-muted" data-testid="model-picker-empty-state">ElevenLabs listed no models.</p>
  {:else}
    <ul class="divide-y divide-edge overflow-hidden rounded-lg border border-edge" data-testid="model-picker-list">
      {#each list as model (model.id)}
        <li>
          <button
            type="button"
            data-testid="model-picker-item"
            onclick={() => onpick(model)}
            class="flex w-full items-start gap-3 px-4 py-3 text-left transition-colors hover:bg-well {model.id === current ? 'bg-brand/10' : ''}"
          >
            <span class="min-w-0 flex-1">
              <span class="block text-sm font-medium text-ink">{model.name}</span>
              {#if model.description}<span class="mt-0.5 block text-xs leading-relaxed text-ink-muted">{model.description}</span>{/if}
              {#if facts(model)}<span class="mt-0.5 block text-xs text-ink-faint">{facts(model)}</span>{/if}
            </span>
            <span class="flex shrink-0 flex-col items-end gap-1">
              <Badge tone={model.creditsPerLetter < 1 ? "good" : model.creditsPerLetter > 1 ? "warn" : "quiet"}>{rateWords(model.creditsPerLetter)}</Badge>
              {#if model.takesDirection}<Badge tone="quiet">Takes direction</Badge>{/if}
              {#if model.id === current}<Badge>In use</Badge>{/if}
            </span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</Modal>
