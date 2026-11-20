<script lang="ts">
  import Button from "$lib/components/Button.svelte";
  import Group from "$lib/components/Group.svelte";
  import Setting from "$lib/components/Setting.svelte";
  import { editor } from "$lib/stores/editor.svelte";
  import { options } from "$lib/stores/options.svelte";

  let screens = $derived<[string, string][]>((options.value?.screens ?? []).map((s) => [s.key, `${s.name}  (${s.width} x ${s.height})`]));
  let exact = $derived(editor.setup?.screen.width != null && editor.setup?.screen.height != null);

  function useExact() {
    const stage = editor.plan?.stage;
    editor.setFor("screen.width", null, stage?.width ?? 1920);
    editor.setFor("screen.height", null, stage?.height ?? 1080);
  }
  function usePreset() {
    editor.resetFor("screen.width", null);
    editor.resetFor("screen.height", null);
  }
</script>

<div data-testid="screen-panel">
  <Group title="Screen size" testid="screen-size-group">
    {#if !exact}
      <Setting path="screen.preset" label="Made for" kind="select" options={screens} help="Wide screens put the beads beside the words. Tall screens put the beads on top." />
      <div class="pb-2"><Button size="sm" testid="screen-exact-btn" onclick={useExact}>Use an exact size instead…</Button></div>
    {:else}
      <Setting path="screen.width" label="Width" kind="number" min={240} max={7680} step={2} slider={false} unit="px" />
      <Setting path="screen.height" label="Height" kind="number" min={240} max={7680} step={2} slider={false} unit="px" />
      <div class="pb-2"><Button size="sm" testid="screen-preset-btn" onclick={usePreset}>Go back to the named sizes</Button></div>
    {/if}
  </Group>

  <Group title="Safe edges" folded testid="screen-safe-group" note="Space kept clear at each edge, so nothing sits under a camera cutout or a rounded corner.">
    <Setting path="screen.safe.top" label="Top" kind="number" min={0} max={400} step={2} unit="px" />
    <Setting path="screen.safe.bottom" label="Bottom" kind="number" min={0} max={400} step={2} unit="px" />
    <Setting path="screen.safe.left" label="Left" kind="number" min={0} max={400} step={2} unit="px" />
    <Setting path="screen.safe.right" label="Right" kind="number" min={0} max={400} step={2} unit="px" />
  </Group>

  <Group title="Smoothness" folded testid="screen-fps-group">
    <Setting path="screen.fps" label="Frames a second" kind="number" min={10} max={60} slider={false} help="30 is the first video. There is little reason to change it." />
  </Group>
</div>
