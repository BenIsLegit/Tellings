<script lang="ts">
  import type { Snippet } from "svelte";

  let { tone = "brand", testid = "badge", children }: { tone?: "brand" | "good" | "warn" | "bad" | "quiet"; testid?: string; children: Snippet } = $props();

  const tones = {
    brand: "bg-brand/15 text-brand",
    good: "bg-emerald-900/30 text-emerald-400",
    warn: "bg-amber-900/30 text-amber-400",
    bad: "bg-red-900/40 text-red-400",
    quiet: "bg-well text-ink-muted",
  };
</script>

<span class="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs {tones[tone]}" data-testid={testid}>{@render children()}</span>
