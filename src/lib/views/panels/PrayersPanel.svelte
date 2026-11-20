<script lang="ts">
  import { Plus, Trash2 } from "lucide-svelte";
  import Button from "$lib/components/Button.svelte";
  import Group from "$lib/components/Group.svelte";
  import LinesInput from "$lib/components/LinesInput.svelte";
  import Select from "$lib/components/Select.svelte";
  import Setting from "$lib/components/Setting.svelte";
  import TextInput from "$lib/components/TextInput.svelte";
  import { editor } from "$lib/stores/editor.svelte";
  import { options } from "$lib/stores/options.svelte";

  let prayers = $derived(editor.setup?.prayers ?? {});
  let custom = $derived(prayers.mysteries === "custom");
  let customItems = $derived<string[]>([0, 1, 2, 3, 4].map((i) => prayers.custom?.items?.[i] ?? ""));

  // Your own wording, one prayer at a time.
  let wordingKey = $state("hailmary");
  let builtIn = $derived(options.value?.prayers.find((p) => p.key === wordingKey)?.lines ?? []);
  let ownWording = $derived<string[] | undefined>(prayers.wording?.[wordingKey]);

  function setCustomItem(i: number, text: string) {
    const items = [...customItems];
    items[i] = text;
    editor.setFor("prayers.custom.items", null, items);
  }

  function setWording(lines: string[]) {
    if (lines.length === 0 || lines.join("\n") === builtIn.join("\n")) editor.resetFor(`prayers.wording.${wordingKey}`, null);
    else editor.setFor(`prayers.wording.${wordingKey}`, null, lines);
  }

  // Extra prayers at the start or the end.
  function extras(where: "extraStart" | "extraEnd"): { title: string; lines: string[] }[] {
    return prayers[where] ?? [];
  }
  function setExtras(where: "extraStart" | "extraEnd", list: { title: string; lines: string[] }[]) {
    if (list.length === 0) editor.resetFor(`prayers.${where}`, null);
    else editor.setFor(`prayers.${where}`, null, list);
  }

  const EXTRAS = [
    ["extraStart", "Extra prayers at the start", "Said after the opening Sign of the Cross."],
    ["extraEnd", "Extra prayers at the end", "Said on the medal, before the final Sign of the Cross."],
  ] as const;

  let setItems = $derived(options.value?.sets.find((s) => s.key === editor.plan?.setKey)?.items ?? []);
</script>

<div data-testid="prayers-panel">
  <Group title="The mysteries" testid="prayers-mysteries-group">
    <Setting
      path="prayers.mysteries"
      label="Which mysteries"
      kind="select"
      options={[
        ["today", "Today's (by the day of the week)"],
        ["joyful", "Joyful (Monday, Saturday)"],
        ["sorrowful", "Sorrowful (Tuesday, Friday)"],
        ["glorious", "Glorious (Wednesday, Sunday)"],
        ["luminous", "Luminous (Thursday)"],
        ["custom", "My own five meditations"],
      ]}
    />
    {#if custom}
      <div class="space-y-2 py-2" data-testid="prayers-custom">
        <p class="text-sm font-medium text-ink">Your five meditations</p>
        <TextInput value={prayers.custom?.name ?? ""} placeholder="A name, shown as “… Mysteries”" testid="prayers-custom-name-input" onchange={(v) => editor.setFor("prayers.custom.name", null, v)} />
        {#each customItems as item, i (i)}
          <TextInput value={item} placeholder="Decade {i + 1}" testid="prayers-custom-item-input" onchange={(v) => setCustomItem(i, v)} />
        {/each}
        <p class="text-xs text-ink-muted">All five need a name before the video can be made.</p>
      </div>
    {/if}
    <Setting path="prayers.title" label="Title on the opening card" kind="text" />
  </Group>

  <Group title="How much" testid="prayers-range-group">
    <Setting path="prayers.opening" label="Start at the crucifix" kind="toggle" help="Off goes straight to the first decade." />
    <Setting path="prayers.firstDecade" label="First decade" kind="number" min={1} max={5} slider={false} />
    <Setting path="prayers.lastDecade" label="Last decade" kind="number" min={1} max={5} slider={false} />
    <Setting path="prayers.closing" label="End with the closing prayers" kind="toggle" help="The Hail, Holy Queen and the final Sign of the Cross." />
  </Group>

  <Group title="Optional prayers" testid="prayers-optional-group">
    <Setting path="prayers.fatima" label="O My Jesus" kind="toggle" help="Optional. Many people add it after each Glory Be." />
    <Setting path="prayers.closingPrayer" label="Closing Prayer" kind="toggle" help="Optional. Said after the Hail, Holy Queen." />
    <Setting path="prayers.gloryBe" label="Where the Glory Be goes" kind="select" options={[["chain", "On the chain (no bead)"], ["next-bead", "On the next big bead"]]} />
  </Group>

  <Group title="Your own wording" folded testid="prayers-wording-group" note="The traditional wording is used unless you change it here. One line of the prayer on each line.">
    <div class="space-y-2 py-2">
      <Select value={wordingKey} options={(options.value?.prayers ?? []).map((p) => [p.key, p.title + (prayers.wording?.[p.key] ? " (changed)" : "")])} testid="prayers-wording-select" onchange={(v) => (wordingKey = v)} />
      <LinesInput lines={ownWording ?? builtIn} rows={9} testid="prayers-wording-input" onchange={setWording} />
      {#if ownWording}
        <Button size="sm" testid="prayers-wording-reset-btn" onclick={() => editor.resetFor(`prayers.wording.${wordingKey}`, null)}>Use the traditional wording</Button>
      {/if}
    </div>
  </Group>

  <Group title="A reading before each mystery" folded testid="prayers-readings-group" note="You supply the text, word for word. A mystery with no text gets no reading.">
    {#each [1, 2, 3, 4, 5] as n (n)}
      <div class="py-2">
        <p class="mb-1.5 text-sm font-medium text-ink">Mystery {n}{setItems[n - 1] ? `: ${setItems[n - 1].name}` : ""}</p>
        {#if setItems[n - 1]?.reference}<p class="mb-1.5 text-xs text-ink-muted">{setItems[n - 1].reference}</p>{/if}
        <LinesInput
          lines={(prayers.readings?.[String(n)] ?? "").split("\n")}
          rows={3}
          placeholder="Paste the reading here"
          testid="prayers-reading-input"
          onchange={(lines) => (lines.length ? editor.setFor(`prayers.readings.${n}`, null, lines.join("\n")) : editor.resetFor(`prayers.readings.${n}`, null))}
        />
      </div>
    {/each}
  </Group>

  {#each EXTRAS as [where, title, note] (where)}
    <Group {title} folded {note} testid="prayers-{where}-group">
      {#each extras(where) as extra, i (i)}
        <div class="space-y-2 py-2" data-testid="prayers-{where}-item">
          <div class="flex gap-2">
            <TextInput value={extra.title} placeholder="Title" testid="prayers-extra-title-input" onchange={(v) => setExtras(where, extras(where).map((e, j) => (j === i ? { ...e, title: v } : e)))} />
            <Button variant="ghost" title="Remove this prayer" testid="prayers-extra-remove-btn" onclick={() => setExtras(where, extras(where).filter((_, j) => j !== i))}><Trash2 size={16} /></Button>
          </div>
          <LinesInput lines={extra.lines} rows={4} placeholder="The words, one line each" testid="prayers-extra-lines-input" onchange={(lines) => setExtras(where, extras(where).map((e, j) => (j === i ? { ...e, lines } : e)))} />
        </div>
      {/each}
      <div class="py-2">
        <Button size="sm" testid="prayers-{where}-add-btn" onclick={() => setExtras(where, [...extras(where), { title: "My prayer", lines: [] }])}><Plus size={14} /> Add a prayer</Button>
      </div>
    </Group>
  {/each}
</div>
