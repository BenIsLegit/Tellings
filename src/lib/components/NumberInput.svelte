<script lang="ts">
  // A number with a slider beside it. The slider is for feel, the box for exact values.
  let {
    value,
    min,
    max,
    step = 1,
    unit = "",
    slider = true,
    testid = "number-input",
    onchange,
  }: {
    value: number;
    min: number;
    max: number;
    step?: number;
    unit?: string;
    slider?: boolean;
    testid?: string;
    onchange: (value: number) => void;
  } = $props();

  function commit(raw: string) {
    const n = Number(raw);
    if (raw.trim() === "" || Number.isNaN(n)) return;
    const clamped = Math.min(max, Math.max(min, n));
    if (clamped !== value) onchange(Math.round(clamped * 1000) / 1000);
  }
</script>

<div class="flex min-w-0 items-center gap-3" data-testid={testid}>
  {#if slider}
    <input
      type="range"
      {min}
      {max}
      {step}
      value={value ?? min}
      data-testid="{testid}-slider"
      onchange={(e) => commit(e.currentTarget.value)}
      class="h-1.5 min-w-0 flex-1 cursor-pointer appearance-none rounded-full bg-frame accent-brand"
    />
  {/if}
  <div class="flex items-center gap-1.5">
    <input
      type="number"
      {min}
      {max}
      {step}
      value={value ?? ""}
      data-testid="{testid}-box"
      onchange={(e) => commit(e.currentTarget.value)}
      class="w-20 rounded-md border border-frame bg-well px-2 py-1.5 text-right text-sm text-ink tabular-nums focus:border-brand focus:outline-none"
    />
    {#if unit}<span class="min-w-8 text-xs whitespace-nowrap text-ink-muted">{unit}</span>{/if}
  </div>
</div>
