<script lang="ts">
  // One picture for each mystery or section. Each row is a rule for that part
  // of the video, so a picture set here never disturbs the others.
  import { ChevronDown, ChevronRight, FolderOpen, ImageOff } from "lucide-svelte";
  import { api } from "$lib/api";
  import Button from "$lib/components/Button.svelte";
  import Group from "$lib/components/Group.svelte";
  import NumberInput from "$lib/components/NumberInput.svelte";
  import Setting from "$lib/components/Setting.svelte";
  import { editor } from "$lib/stores/editor.svelte";
  import type { Target } from "$lib/types";
  import { fileName } from "$lib/utils/format";
  import { invokeWithToast } from "$lib/utils/errors";

  const ROMAN = ["I", "II", "III", "IV", "V"];

  let rows = $derived.by(() => {
    const scenes = editor.plan?.scenes ?? [];
    const out: { key: string; label: string; target: Target }[] = [{ key: "intro", label: "Opening title", target: { section: "intro" } }];
    if (scenes.some((s) => s.section === "opening")) out.push({ key: "opening", label: "Opening prayers", target: { section: "opening" } });
    for (let n = 1; n <= 5; n++) {
      const scene = scenes.find((s) => s.mystery === n);
      if (scene) out.push({ key: `m${n}`, label: `${ROMAN[n - 1]}. ${editor.plan?.scenes.find((s) => s.mystery === n && s.chapter)?.chapter?.replace(/^[IV]+\.\s*/, "") ?? `Mystery ${n}`}`, target: { mystery: n } });
    }
    if (scenes.some((s) => s.section === "closing")) out.push({ key: "closing", label: "Closing prayers", target: { section: "closing" } });
    out.push({ key: "outro", label: "Ending", target: { section: "outro" } });
    return out;
  });

  let openRow = $state<string | null>(null);

  function show(row: { key: string; target: Target }) {
    openRow = openRow === row.key ? null : row.key;
    const i = editor.plan?.scenes.findIndex((s) => (row.target.mystery ? s.mystery === row.target.mystery : s.section === row.target.section)) ?? -1;
    if (i >= 0) editor.go(i);
  }

  // "50% 90%" is two numbers: how far across, how far down.
  function focusOf(target: Target): [number, number] {
    const text = String(editor.valueFor("picture.focus", target, editor.sceneOf(target)) ?? "50% 50%");
    const [x, y] = text.split(/\s+/).map((p) => Number(p.replace("%", "")));
    return [Number.isFinite(x) ? x : 50, Number.isFinite(y) ? y : 50];
  }
  function setFocus(target: Target, x: number, y: number) {
    editor.setFor("picture.focus", target, `${Math.round(x)}% ${Math.round(y)}%`);
  }

  async function openFolder() {
    const d = await invokeWithToast(() => api.doctor());
    if (d) await invokeWithToast(() => api.openPath(d.picturesFolder));
  }
</script>

<div data-testid="pictures-panel">
  <Group title="A picture for each part" testid="pictures-rows-group" note="Use pictures you supply, or ones in the public domain. Give each one a credit: artist, title and date.">
    <ul class="mt-2 divide-y divide-edge overflow-hidden rounded-lg border border-edge" data-testid="pictures-list">
      {#each rows as row (row.key)}
        {@const scene = editor.sceneOf(row.target)}
        {@const file = scene?.pictureChoice.file as string | null}
        {@const found = !!scene?.picture}
        <li data-testid="pictures-list-item">
          <button type="button" class="flex w-full items-center gap-2 px-3 py-2.5 text-left hover:bg-well" data-testid="pictures-row-btn" onclick={() => show(row)}>
            {#if openRow === row.key}<ChevronDown size={16} class="shrink-0 text-ink-muted" />{:else}<ChevronRight size={16} class="shrink-0 text-ink-muted" />{/if}
            <span class="min-w-0 flex-1">
              <span class="block truncate text-sm font-medium text-ink">{row.label}</span>
              <span class="block truncate text-xs {file && !found ? 'text-amber-400' : 'text-ink-muted'}">
                {#if !file}No picture. The loop holds text.{:else if !found}Not found: {fileName(file)}{:else}{fileName(file)}{/if}
              </span>
            </span>
            {#if file && !found}<ImageOff size={16} class="shrink-0 text-amber-400" />{/if}
          </button>
          {#if openRow === row.key}
            {@const [fx, fy] = focusOf(row.target)}
            <div class="border-t border-edge bg-canvas px-3 pb-2" data-testid="pictures-row-detail">
              <Setting path="picture.file" label="Picture file" kind="file" picks="picture" target={row.target} />
              <Setting path="picture.credit" label="Credit" kind="text" placeholder="Artist, title, date" target={row.target} />
              <div class="py-2.5">
                <p class="mb-1.5 text-sm font-medium text-ink">Which part shows</p>
                <div class="space-y-2">
                  <div class="flex items-center gap-2"><span class="w-14 text-xs text-ink-muted">Across</span><div class="flex-1"><NumberInput value={fx} min={0} max={100} unit="%" testid="pictures-focus-x" onchange={(v) => setFocus(row.target, v, fy)} /></div></div>
                  <div class="flex items-center gap-2"><span class="w-14 text-xs text-ink-muted">Down</span><div class="flex-1"><NumberInput value={fy} min={0} max={100} unit="%" testid="pictures-focus-y" onchange={(v) => setFocus(row.target, fx, v)} /></div></div>
                </div>
              </div>
              <Setting path="picture.zoom" label="Zoom" kind="number" min={1} max={3} step={0.05} unit="x" target={row.target} />
              <Setting path="picture.show" label="Show the picture here" kind="toggle" target={row.target} />
            </div>
          {/if}
        </li>
      {/each}
    </ul>
    <div class="py-3">
      <Button size="sm" testid="pictures-open-folder-btn" onclick={openFolder}><FolderOpen size={14} /> Open my pictures folder</Button>
      <p class="mt-1.5 text-xs leading-relaxed text-ink-muted">The Luminous setup looks there for luminous-1 to luminous-5.</p>
    </div>
  </Group>

  <Group title="For every picture" testid="pictures-all-group">
    <Setting path="picture.place" label="Where the picture sits" kind="select" options={[["loop", "Inside the loop of beads"], ["beside", "Above the words"], ["background", "Filling the background, dimmed"]]} target={null} />
    <Setting path="picture.showCredit" label="Show the credit line" kind="toggle" target={null} help="Artist, title and date, at the foot of the screen." />
  </Group>
</div>
