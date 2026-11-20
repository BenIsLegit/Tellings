<script lang="ts">
  import Group from "$lib/components/Group.svelte";
  import ScopeBar from "$lib/components/ScopeBar.svelte";
  import Setting from "$lib/components/Setting.svelte";
</script>

<div data-testid="pace-panel">
  <ScopeBar />

  <Group title="Pauses" testid="pace-pauses-group">
    <Setting path="pace.before" label="Quiet before each prayer" kind="number" min={0} max={5} step={0.05} unit="sec" />
    <Setting path="pace.after" label="Quiet after each prayer" kind="number" min={0} max={10} step={0.05} unit="sec" />
    <Setting path="pace.think" label="Time to think after a mystery is named" kind="number" min={0} max={60} step={0.5} unit="sec" />
    <Setting path="pace.announceGap" label="Between the mystery's name and the Our Father" kind="number" min={0} max={5} step={0.05} unit="sec" />
  </Group>

  <Group title="With no voice" testid="pace-silent-group" note="How long a bead waits when nothing is spoken.">
    <Setting path="pace.secondsPerLine" label="For each line of the prayer" kind="number" min={0.5} max={8} step={0.1} unit="sec" />
    <Setting path="pace.minSeconds" label="Shortest wait on any bead" kind="number" min={1} max={30} step={0.5} unit="sec" />
  </Group>

  <Group title="From one bead to the next" testid="pace-change-group">
    <Setting path="pace.transition" label="The change" kind="select" options={[["fade", "A soft fade"], ["cut", "A plain cut"]]} />
    <Setting path="pace.fade" label="Length of the fade" kind="number" min={0.1} max={2} step={0.05} unit="sec" />
  </Group>

  <Group title="Opening and ending" testid="pace-ends-group" note="These two are for the whole video.">
    <Setting path="pace.intro" label="Opening title stays up" kind="number" min={0} max={30} step={0.5} unit="sec" target={null} />
    <Setting path="pace.outro" label="Ending stays up" kind="number" min={0} max={30} step={0.5} unit="sec" target={null} />
  </Group>
</div>
