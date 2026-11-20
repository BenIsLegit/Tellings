<script lang="ts">
  import type { Snippet } from "svelte";

  let { title = "", testid = "card", children }: { title?: string; testid?: string; children: Snippet } = $props();
</script>

<section class="rounded-lg border border-edge bg-panel p-5" data-testid={testid}>
  {#if title}<h3 class="mb-3 text-lg font-semibold text-ink">{title}</h3>{/if}
  {@render children()}
</section>
