<script lang="ts">
  // Pick a voice from the ElevenLabs account. Listing voices is free.
  import { Loader2, RefreshCw } from "lucide-svelte";
  import { api } from "$lib/api";
  import type { VoiceInfo } from "$lib/types";
  import Badge from "./Badge.svelte";
  import Button from "./Button.svelte";
  import Modal from "./Modal.svelte";

  let { open, current = "", onpick, onclose }: { open: boolean; current?: string; onpick: (voice: VoiceInfo) => void; onclose: () => void } = $props();

  let list = $state<VoiceInfo[]>([]);
  let loading = $state(false);
  let problem = $state<string | null>(null);
  let search = $state("");

  let shown = $derived(list.filter((v) => v.name.toLowerCase().includes(search.trim().toLowerCase())));

  async function load(refresh: boolean) {
    loading = true;
    problem = null;
    try {
      list = await api.voices(refresh);
    } catch (e) {
      problem = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (open && list.length === 0 && !loading && !problem) load(false);
  });
</script>

<Modal {open} title="Choose a voice" wide testid="voice-picker" {onclose}>
  <div class="mb-3 flex gap-2">
    <input
      type="search"
      placeholder="Search your voices"
      bind:value={search}
      data-testid="voice-picker-search-input"
      class="min-w-0 flex-1 rounded-md border border-frame bg-well px-3 py-2 text-sm text-ink placeholder-ink-faint focus:border-brand focus:outline-none"
    />
    <Button title="Fetch the list again" testid="voice-picker-refresh-btn" onclick={() => load(true)}><RefreshCw size={16} /> Refresh</Button>
  </div>
  {#if loading}
    <div class="flex items-center justify-center py-12" data-testid="voice-picker-loading">
      <Loader2 size={24} class="animate-spin text-ink-muted" />
      <span class="ml-3 text-sm text-ink-muted">Asking ElevenLabs for your voices…</span>
    </div>
  {:else if problem}
    <div class="rounded-lg border border-red-900/50 bg-red-900/20 p-4" data-testid="voice-picker-error">
      <p class="text-sm text-red-400">{problem}</p>
    </div>
  {:else if shown.length === 0}
    <p class="py-8 text-center text-sm text-ink-muted" data-testid="voice-picker-empty-state">No voice matches that.</p>
  {:else}
    <ul class="divide-y divide-edge overflow-hidden rounded-lg border border-edge" data-testid="voice-picker-list">
      {#each shown as voice (voice.id)}
        <li>
          <button
            type="button"
            data-testid="voice-picker-item"
            onclick={() => onpick(voice)}
            class="flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-well {voice.name === current ? 'bg-brand/10' : ''}"
          >
            <span class="min-w-0 flex-1">
              <span class="block truncate text-sm font-medium text-ink">{voice.name}</span>
              {#if voice.description}<span class="block truncate text-xs text-ink-muted">{voice.description}</span>{/if}
            </span>
            {#if voice.category}<Badge tone="quiet">{voice.category}</Badge>{/if}
            {#if voice.name === current}<Badge>In use</Badge>{/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</Modal>
