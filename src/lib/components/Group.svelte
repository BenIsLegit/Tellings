<script lang="ts">
  // A titled group of settings inside an editor panel. Can start folded away.
  import type { Snippet } from "svelte";
  import { ChevronDown, ChevronRight } from "lucide-svelte";

  let { title, note = "", folded = false, testid = "group", children }: { title: string; note?: string; folded?: boolean; testid?: string; children: Snippet } = $props();

  // `folded` only says how the group starts. After that the person is in charge.
  // svelte-ignore state_referenced_locally
  let open = $state(!folded);
</script>

<section class="border-t border-edge py-3 first:border-t-0" data-testid={testid}>
  <button type="button" class="flex w-full items-center gap-1.5 text-left" data-testid="{testid}-toggle-btn" onclick={() => (open = !open)}>
    {#if open}<ChevronDown size={16} class="text-ink-muted" />{:else}<ChevronRight size={16} class="text-ink-muted" />{/if}
    <span class="text-xs font-semibold tracking-wide text-ink-tertiary uppercase">{title}</span>
  </button>
  {#if open}
    {#if note}<p class="mt-2 text-xs leading-relaxed text-ink-muted">{note}</p>{/if}
    <div class="mt-1">{@render children()}</div>
  {/if}
</section>
