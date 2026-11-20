<script lang="ts">
  // Every choice this setup makes for only part of the video, in plain words.
  import { Trash2 } from "lucide-svelte";
  import Button from "$lib/components/Button.svelte";
  import Group from "$lib/components/Group.svelte";
  import { editor } from "$lib/stores/editor.svelte";
  import type { Rule } from "$lib/types";
  import { fileName, targetName } from "$lib/utils/format";
  import { flatten } from "$lib/utils/path";

  let own = $derived<Rule[]>(Array.isArray(editor.doc.rules) ? editor.doc.rules : []);
  let inherited = $derived(Math.max(0, (editor.setup?.rules.length ?? 0) - own.length));

  const NAMES: Record<string, string> = { voice: "Voice", pace: "Pace", words: "Words", picture: "Picture", colors: "Colors" };

  function say(path: string, value: unknown): string {
    const [group, ...rest] = path.split(".");
    const shown = typeof value === "string" && /[\\/]/.test(value) ? fileName(value) : String(value);
    return `${NAMES[group] ?? group}: ${rest.join(" ")} = ${shown}`;
  }

  function jump(rule: Rule) {
    const scene = editor.sceneOf(rule.for);
    const i = scene ? (editor.plan?.scenes.indexOf(scene) ?? -1) : -1;
    if (i >= 0) editor.go(i);
  }
</script>

<div data-testid="parts-panel">
  <Group title="Choices for part of the video" testid="parts-own-group" note="The narrowest choice wins: one bead, then one kind of prayer, then one mystery, then one section, then the whole video.">
    {#if own.length === 0}
      <div class="py-6 text-center text-sm text-ink-muted" data-testid="parts-empty-state">
        <p>Nothing yet.</p>
        <p class="mt-1 text-xs leading-relaxed">Open Voice, Words, Colors or Pace, pick what the choice covers, and change something.</p>
      </div>
    {:else}
      <ul class="mt-2 space-y-2" data-testid="parts-list">
        {#each own as rule, i (i)}
          <li class="rounded-lg border border-edge bg-canvas p-3" data-testid="parts-list-item">
            <div class="flex items-start gap-2">
              <button type="button" class="min-w-0 flex-1 text-left" title="Show this part in the preview" data-testid="parts-jump-btn" onclick={() => jump(rule)}>
                <span class="block text-sm font-medium text-ink">{targetName(rule.for)}</span>
                {#each flatten(rule.set) as [path, value] (path)}
                  <span class="block truncate text-xs text-ink-muted">{say(path, value)}</span>
                {/each}
              </button>
              <Button variant="ghost" size="sm" title="Remove this choice" testid="parts-remove-btn" onclick={() => editor.removeRule(i)}><Trash2 size={14} /></Button>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
    {#if inherited > 0}
      <p class="py-3 text-xs leading-relaxed text-ink-muted" data-testid="parts-inherited">
        {inherited} more {inherited === 1 ? "comes" : "come"} from “{editor.startsFrom}”, which this setup starts from. A choice you make for the same part wins over it.
      </p>
    {/if}
  </Group>
</div>
