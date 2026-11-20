<script lang="ts">
  // "This choice covers…" — the five levels from docs/what-you-can-change.md.
  // Whatever is picked here is what the settings below it change.
  import { editor } from "$lib/stores/editor.svelte";
</script>

<div class="rounded-lg border border-edge bg-canvas p-3" data-testid="scope-bar">
  <p class="mb-2 text-xs font-medium tracking-wide text-ink-muted uppercase">This choice covers</p>
  <div class="flex flex-wrap gap-1.5">
    {#each editor.scopes as s (s.kind)}
      <button
        type="button"
        disabled={!s.available}
        title={s.available ? s.hint : "Move the preview to a prayer to use this"}
        data-testid="scope-{s.kind}-btn"
        onclick={() => (editor.scope = s.kind)}
        class="rounded-md px-2.5 py-1.5 text-sm transition-colors disabled:cursor-not-allowed disabled:opacity-40
          {editor.activeScope === s.kind ? 'bg-brand font-medium text-on-brand' : 'border border-frame text-ink-secondary hover:bg-well'}"
      >
        {s.label}
      </button>
    {/each}
  </div>
  <p class="mt-2 text-xs leading-relaxed text-ink-muted" data-testid="scope-hint">
    {#if editor.activeScope === "video"}
      Changes below apply everywhere. The narrowest choice always wins.
    {:else}
      Changes below apply only here. Move the preview to change a different part.
    {/if}
  </p>
</div>
