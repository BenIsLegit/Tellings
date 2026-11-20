<script lang="ts">
  import Group from "$lib/components/Group.svelte";
  import ScopeBar from "$lib/components/ScopeBar.svelte";
  import Setting from "$lib/components/Setting.svelte";
  import { editor } from "$lib/stores/editor.svelte";

  // The colors in force, to show beside each empty box.
  let dark = $derived((editor.valueFor("colors.mode", editor.activeTarget) ?? "dark") === "dark");
  let base = $derived(
    dark
      ? { background: "#0E1626", panel: "#172138", text: "#E7ECF5", quietText: "#9BA8C0", lines: "#34425F", done: "#6F80A6", marker: "#E6B556" }
      : { background: "#F7F3EA", panel: "#ECE5D6", text: "#1C2434", quietText: "#5A657B", lines: "#CFC6B4", done: "#98A3B9", marker: "#C8860A" },
  );
</script>

<div data-testid="colors-panel">
  <ScopeBar />

  <Group title="Dark or light" testid="colors-mode-group">
    <Setting path="colors.mode" label="Colors" kind="select" options={[["dark", "Dark (first video)"], ["light", "Light"]]} />
  </Group>

  <Group title="The bead you are on" testid="colors-marker-group">
    <Setting path="colors.marker" label="Marker" kind="color" fallback={base.marker} help="Gold in the first video. Keep it easy to tell from the other beads." />
    <Setting path="colors.done" label="Beads already prayed" kind="color" fallback={base.done} />
  </Group>

  <Group title="Everything else" folded testid="colors-rest-group" note="Leave a box empty to use the usual color.">
    <Setting path="colors.background" label="Background" kind="color" fallback={base.background} />
    <Setting path="colors.text" label="Words" kind="color" fallback={base.text} />
    <Setting path="colors.quietText" label="Quieter words" kind="color" fallback={base.quietText} />
    <Setting path="colors.lines" label="Chain and thin lines" kind="color" fallback={base.lines} />
    <Setting path="colors.panel" label="Behind pictures" kind="color" fallback={base.panel} />
  </Group>

  <Group title="Fonts" folded testid="colors-fonts-group" note="Type the name of any font installed on this computer. Empty uses the first video's fonts.">
    <Setting path="colors.titleFont" label="Titles" kind="text" placeholder="Cardo" />
    <Setting path="colors.textFont" label="Everything else" kind="text" placeholder="Fira Sans" />
  </Group>
</div>
