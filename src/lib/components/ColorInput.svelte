<script lang="ts">
  import TextInput from "./TextInput.svelte";

  let { value, fallback = "#888888", testid = "color-input", onchange }: { value: string | null; fallback?: string; testid?: string; onchange: (value: string) => void } = $props();

  // The color picker only speaks #rrggbb. Anything else is typed in the box.
  let swatch = $derived(/^#[0-9a-fA-F]{6}$/.test(value ?? "") ? (value as string) : fallback);
</script>

<div class="flex min-w-0 items-center gap-2" data-testid={testid}>
  <input
    type="color"
    value={swatch}
    data-testid="{testid}-picker"
    onchange={(e) => onchange(e.currentTarget.value.toUpperCase())}
    class="h-9 w-11 shrink-0 cursor-pointer rounded-md border border-frame bg-well p-1"
  />
  <TextInput value={value ?? ""} placeholder="like {fallback}" mono testid="{testid}-text" {onchange} />
</div>
