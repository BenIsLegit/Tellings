<script lang="ts">
  import Group from "$lib/components/Group.svelte";
  import Setting from "$lib/components/Setting.svelte";
  import { editor } from "$lib/stores/editor.svelte";
  import { commas, fileName } from "$lib/utils/format";

  // Music can be a file of the person's own, or made by ElevenLabs from a description.
  let described = $derived(String(editor.setup?.music?.prompt ?? "").trim() !== "");
  let music = $derived(editor.look?.music);
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
    <Setting
      path="output.fileName"
      label="File name"
      kind="text"
      placeholder="Named for what is in it"
      help="Leave empty and the name says what the video is: the prayer, the mysteries, the part, the voice, the model and the screen size. The title a player shows is the same."
    />
    {#if editor.look?.files.length}
      <div class="rounded-md border border-edge bg-canvas p-3 text-xs leading-relaxed text-ink-muted" data-testid="output-files">
        <p class="mb-1 font-medium text-ink-secondary">Will write</p>
        <!-- The end of a name is what tells two videos apart, so a long one wraps and is never cut short. -->
        {#each editor.look.files as f (f)}<p class="break-words" title={f}>{fileName(f)}</p>{/each}
      </div>
    {/if}
  </Group>

  <Group title="Background sound" folded testid="output-music-group" note="Music or chant under the voice. It loops and fades at the end. Supply a sound file of your own, or describe music for ElevenLabs to make.">
    <Setting path="music.file" label="Sound file" kind="file" picks="sound" placeholder="Silence" help="A file of your own. It wins over music made from words." />
    <Setting
      path="music.prompt"
      label="Or describe music to make"
      kind="text"
      placeholder="slow, quiet organ in an empty chapel"
      help="ElevenLabs makes it once, and it is kept. Describe the sound. Do not name a hymn, a song or an artist: ElevenLabs refuses those. This spends credits, about 900 a minute. You are asked first."
    />
    {#if described}
      <Setting path="music.minutes" label="How long a piece" kind="number" min={0.5} max={5} step={0.5} unit="minutes" help="It loops under the whole video. A longer piece repeats less and costs more." />
      <Setting path="music.instrumental" label="No singing" kind="toggle" />
      <Setting path="music.take" label="Which piece" kind="number" min={1} max={99} slider={false} help="Not happy with the piece? A new number has ElevenLabs make another from the same words. That costs again." />
      <div class="mb-2 rounded-md border border-edge bg-canvas p-3 text-sm leading-relaxed" data-testid="output-music-cost">
        {#if music?.wanted && music.kept}
          <p class="font-medium text-ink">This piece is already made and kept</p>
          <p class="text-ink-muted">Using it costs nothing.</p>
        {:else if music?.wanted}
          <p class="font-medium text-ink">Making this piece: about {commas(music.credits)} ElevenLabs credits</p>
          <p class="text-ink-muted">A rough figure. ElevenLabs does not give the price before it makes music. You will be asked before anything is spent, and told afterwards what it came to.</p>
        {:else if editor.setup?.music?.file}
          <p class="font-medium text-ink">Not made</p>
          <p class="text-ink-muted">A sound file is chosen above, so it is used and no music is made from these words.</p>
        {:else}
          <p class="font-medium text-ink">Not made</p>
          <p class="text-ink-muted">This setup makes no video and no sound file, so there is nothing to put music under.</p>
        {/if}
      </div>
    {/if}
    <Setting path="music.volume" label="How loud" kind="number" min={0} max={1} step={0.01} />
  </Group>

  <Group title="Picture quality" folded testid="output-quality-group">
    <Setting path="output.quality" label="Quality" kind="number" min={12} max={30} slider={false} help="Lower is sharper and a bigger file. 18 is the first video." />
  </Group>
</div>
