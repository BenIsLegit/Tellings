<script lang="ts">
  import Group from "$lib/components/Group.svelte";
  import Setting from "$lib/components/Setting.svelte";
  import { editor } from "$lib/stores/editor.svelte";
  import { fileName } from "$lib/utils/format";
</script>

<div data-testid="output-panel">
  <Group title="What gets made" testid="output-kinds-group">
    <Setting path="output.video" label="A video" kind="toggle" help="An .mp4 file." />
    <Setting path="output.audio" label="A sound-only file" kind="toggle" help="An .m4a file, for praying on a walk with the beads in your hand." />
    <Setting path="output.captions" label="Captions" kind="toggle" help="A separate .srt file with every line, timed to the voice." />
    <Setting path="output.guide" label="The tap-through guide" kind="toggle" help="A page you move one bead at a time, at your own pace. Opens in any browser." />
    <Setting path="output.chapters" label="Chapters" kind="toggle" help="Marks in the video so you can jump to a decade." />
  </Group>

  <Group title="Where it goes" testid="output-where-group">
    <Setting path="output.folder" label="Folder" kind="file" picks="folder" placeholder="Videos / Prayer Videos" />
    <Setting path="output.fileName" label="File name" kind="text" placeholder="The setup's name" />
    {#if editor.look?.files.length}
      <div class="rounded-md border border-edge bg-canvas p-3 text-xs leading-relaxed text-ink-muted" data-testid="output-files">
        <p class="mb-1 font-medium text-ink-secondary">Will write</p>
        {#each editor.look.files as f (f)}<p class="truncate" title={f}>{fileName(f)}</p>{/each}
      </div>
    {/if}
  </Group>

  <Group title="Background sound" folded testid="output-music-group" note="Music or chant that you supply. It loops under the voice and fades at the end.">
    <Setting path="music.file" label="Sound file" kind="file" picks="sound" placeholder="Silence" />
    <Setting path="music.volume" label="How loud" kind="number" min={0} max={1} step={0.01} />
  </Group>

  <Group
    title="A room for the voice"
    folded
    testid="output-room-group"
    note="Puts the voice in a room, with the echo ringing into the pauses. It is added to the finished sound, so it costs no credits and nothing is recorded again."
  >
    <Setting
      path="room.kind"
      label="Room"
      kind="select"
      options={[
        ["none", "No room"],
        ["chapel", "A chapel"],
        ["church", "A church"],
        ["cathedral", "A cathedral"],
      ]}
      help="No room is the first video. A chapel's echo is gone in about a second. A cathedral's lasts four or five."
    />
    {#if editor.setup?.room?.kind && editor.setup.room.kind !== "none"}
      <Setting path="room.amount" label="How much" kind="number" min={0} max={1} step={0.05} help="0.3 is a clear echo behind the voice. 1 is an echo as loud as the voice. A long echo makes fast speech harder to follow, so keep it low in a cathedral." />
    {/if}
  </Group>

  <Group title="Picture quality" folded testid="output-quality-group">
    <Setting path="output.quality" label="Quality" kind="number" min={12} max={30} slider={false} help="Lower is sharper and a bigger file. 18 is the first video." />
  </Group>
</div>
