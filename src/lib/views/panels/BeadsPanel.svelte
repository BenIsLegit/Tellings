<script lang="ts">
  import Group from "$lib/components/Group.svelte";
  import Setting from "$lib/components/Setting.svelte";
</script>

<div data-testid="beads-panel">
  <Group title="Which beads" testid="beads-kind-group">
    <div class="rounded-md border border-edge bg-canvas p-3 text-sm leading-relaxed text-ink-secondary" data-testid="beads-kind-note">
      <p class="font-medium text-ink">Five-decade rosary. 59 beads.</p>
      <p class="mt-1 text-ink-muted">A crucifix. Five beads above it: one big, three small, one big. The medal. Five sets of ten, with a big bead between sets.</p>
      <p class="mt-1 text-ink-muted">Other kinds of beads are not built yet. Each one has to be checked against a source its own tradition trusts first.</p>
    </div>
    <Setting path="beads.direction" label="Which way you go" kind="select" options={[["right-first", "Up the right side first"], ["left-first", "Up the left side first"]]} />
  </Group>

  <Group title="The medal and the cross" testid="beads-ends-group">
    <Setting path="beads.medal" label="The medal" kind="select" options={[["plain", "Plain medal"], ["st-benedict", "St. Benedict medal"], ["miraculous", "Miraculous Medal"]]} />
    <Setting
      path="beads.cross"
      label="What hangs at the end"
      kind="select"
      options={[
        ["latin", "Latin crucifix"],
        ["st-benedict", "St. Benedict crucifix"],
        ["orthodox", "Orthodox three-bar cross"],
        ["greek", "Greek cross, equal arms"],
        ["celtic", "Celtic cross"],
        ["san-damiano", "San Damiano cross"],
        ["knotted", "Knotted cross"],
        ["tassel", "Tassel"],
        ["medal", "A medal"],
      ]}
    />
  </Group>

  <Group title="The look" testid="beads-look-group">
    <Setting
      path="beads.style"
      label="Material"
      kind="select"
      options={[["outline", "Plain outline (first video)"], ["wood", "Wood"], ["glass", "Glass"], ["stone", "Stone"], ["pearl", "Pearl"], ["metal", "Metal"], ["knots", "Knots of wool"]]}
      help="Whatever the material, the bead you are on is always the marker color."
    />
    <Setting path="beads.color" label="Bead color" kind="color" fallback="#9A6B43" help="Leave empty for the usual color of the material. Plain outline uses the text color." />
    <Setting path="beads.size" label="Bead size" kind="number" min={0.6} max={1.4} step={0.05} unit="x" />
    <Setting path="beads.cord" label="Chain or cord" kind="select" options={[["chain", "Fine chain"], ["cord", "Thicker cord"]]} />
    <Setting path="beads.cordColor" label="Chain color" kind="color" fallback="#34425F" />
  </Group>
</div>
