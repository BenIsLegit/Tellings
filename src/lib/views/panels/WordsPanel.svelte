<script lang="ts">
  import Group from "$lib/components/Group.svelte";
  import ScopeBar from "$lib/components/ScopeBar.svelte";
  import Setting from "$lib/components/Setting.svelte";
</script>

<div data-testid="words-panel">
  <ScopeBar />

  <Group title="The prayer's words" testid="words-text-group">
    <Setting path="words.size" label="Size" kind="number" min={0.6} max={1.6} step={0.05} unit="x" help="1 is the first video. Long prayers shrink on their own to fit." />
    <Setting path="words.text" label="Show the words" kind="select" options={[["always", "Always"], ["first", "Only the first time each prayer comes up"], ["never", "Never (title only)"]]} />
    <Setting path="words.follow" label="Follow along" kind="toggle" help="The line being spoken lights up." />
  </Group>

  <Group title="Around the words" testid="words-extras-group">
    <Setting path="words.place" label="Where your fingers are" kind="toggle" help="Lines like “Small bead 4 of 10”." />
    <Setting path="words.counter" label="The bead counter" kind="toggle" help="Ten dots that fill as you go through a decade." />
    <Setting path="words.mysteryName" label="The mystery's name" kind="toggle" />
    <Setting path="words.hints" label="Hints" kind="toggle" help="Lines like “Many people offer this one for faith.”" />
    <Setting path="words.optionalLabel" label="The “Optional” label" kind="toggle" />
    <Setting path="words.progress" label="The progress line" kind="toggle" help="“Prayer 13 of 75” and the bar under it." />
  </Group>
</div>
