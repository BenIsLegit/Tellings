<script lang="ts">
  let {
    value,
    options,
    testid = "select",
    onchange,
  }: {
    value: string;
    /** Pairs of the stored value and the words shown for it. */
    options: [string, string][];
    testid?: string;
    onchange: (value: string) => void;
  } = $props();
</script>

<select
  data-testid={testid}
  value={value ?? ""}
  onchange={(e) => onchange(e.currentTarget.value)}
  class="w-full min-w-0 rounded-md border border-frame bg-well px-3 py-2 text-sm text-ink focus:border-brand focus:outline-none"
>
  {#each options as [key, label] (key)}
    <option value={key}>{label}</option>
  {/each}
</select>
