<script lang="ts">
  // A box for the words of a prayer: one line of the prayer on each line of the box.
  let { lines, placeholder = "", rows = 6, testid = "lines-input", onchange }: { lines: string[]; placeholder?: string; rows?: number; testid?: string; onchange: (lines: string[]) => void } = $props();

  let draft = $state("");
  let editing = $state(false);

  $effect(() => {
    if (!editing) draft = (lines ?? []).join("\n");
  });

  function commit() {
    const next = draft
      .split("\n")
      .map((l) => l.trim())
      .filter((l) => l !== "");
    if (next.join("\n") !== (lines ?? []).join("\n")) onchange(next);
  }
</script>

<textarea
  {rows}
  {placeholder}
  data-testid={testid}
  value={draft}
  spellcheck="false"
  oninput={(e) => (draft = e.currentTarget.value)}
  onfocus={() => (editing = true)}
  onblur={() => {
    editing = false;
    commit();
  }}
  class="w-full resize-y rounded-md border border-frame bg-well px-3 py-2 text-sm leading-relaxed text-ink placeholder-ink-faint focus:border-brand focus:outline-none"
></textarea>
