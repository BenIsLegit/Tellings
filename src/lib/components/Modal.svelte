<script lang="ts">
  import type { Snippet } from "svelte";
  import { X } from "lucide-svelte";

  let {
    open,
    title,
    wide = false,
    closable = true,
    testid = "dialog",
    onclose,
    children,
    actions,
  }: {
    open: boolean;
    title: string;
    wide?: boolean;
    /** false while something is running that should not be walked away from. */
    closable?: boolean;
    testid?: string;
    onclose: () => void;
    children: Snippet;
    actions?: Snippet;
  } = $props();
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    role="dialog"
    tabindex="-1"
    aria-modal="true"
    aria-label={title}
    data-testid={testid}
    onclick={(e) => {
      if (closable && e.target === e.currentTarget) onclose();
    }}
    onkeydown={(e) => {
      if (closable && e.key === "Escape") onclose();
    }}
  >
    <div class="mx-4 flex max-h-[88vh] w-full flex-col rounded-lg border border-edge bg-panel shadow-xl {wide ? 'max-w-2xl' : 'max-w-md'}">
      <div class="flex items-center justify-between border-b border-edge px-6 py-4" data-testid="{testid}-header">
        <h3 class="text-lg font-semibold text-ink">{title}</h3>
        {#if closable}
          <button type="button" class="rounded-md p-1 text-ink-muted hover:bg-well hover:text-ink" aria-label="Close" data-testid="{testid}-close-btn" onclick={onclose}>
            <X size={18} />
          </button>
        {/if}
      </div>
      <div class="overflow-y-auto px-6 py-5 text-sm text-ink-secondary" data-testid="{testid}-body">
        {@render children()}
      </div>
      {#if actions}
        <div class="flex justify-end gap-3 border-t border-edge px-6 py-4" data-testid="{testid}-footer">
          {@render actions()}
        </div>
      {/if}
    </div>
  </div>
{/if}
