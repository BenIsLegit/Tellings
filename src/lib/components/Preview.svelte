<script lang="ts" module>
  import { api } from "$lib/api";

  // Pictures are read through the engine once and kept for the life of the window.
  const pictures = new Map<string, Promise<string>>();
  function picture(file: string): Promise<string> {
    let p = pictures.get(file);
    if (!p) {
      p = api.pictureData(file).catch(() => "");
      pictures.set(file, p);
    }
    return p;
  }
</script>

<script lang="ts">
  // The live preview. It runs the very same drawing code that makes the video's
  // stills (renderer/frame.js), so what shows here is what the video will show.
  import "../../../renderer/frame.js";
  import frameCss from "../../../renderer/frame.css?raw";
  import cardo400 from "../../../renderer/fonts/cardo-latin-400-normal.woff2?url";
  import cardo700 from "../../../renderer/fonts/cardo-latin-700-normal.woff2?url";
  import fira400 from "../../../renderer/fonts/fira-sans-latin-400-normal.woff2?url";
  import fira500 from "../../../renderer/fonts/fira-sans-latin-500-normal.woff2?url";
  import type { Plan } from "$lib/types";

  let { plan, index, onbead }: { plan: Plan; index: number; onbead?: (bead: string) => void } = $props();

  let box = $state<HTMLDivElement>();
  let host = $state<HTMLDivElement>();
  let room = $state({ w: 0, h: 0 });
  let frame = $state.raw<ReturnType<typeof PVC.create> | null>(null);

  PVC.installFonts(PVC.fontCss({ cardo400, cardo700, fira400, fira500 }));
  const PREVIEW_CSS = frameCss + "\n.pvc-beads .bead { cursor: pointer; }";

  // Watch how much room there is.
  $effect(() => {
    if (!box) return;
    const watch = new ResizeObserver(([entry]) => {
      room = { w: entry.contentRect.width, h: entry.contentRect.height };
    });
    watch.observe(box);
    return () => watch.disconnect();
  });

  // A new plan means a new drawing.
  $effect(() => {
    if (!host) return;
    frame = PVC.create(host, $state.snapshot(plan), { css: PREVIEW_CSS, shadow: true, picture });
  });

  // Clicking a bead jumps to it.
  $effect(() => {
    const root = host?.shadowRoot;
    if (!root || !frame) return;
    const click = (e: Event) => {
      const bead = (e.target as Element | null)?.closest?.(".bead")?.getAttribute("data-bead");
      if (bead) onbead?.(bead);
    };
    root.addEventListener("click", click);
    return () => root.removeEventListener("click", click);
  });

  // Fit the drawing to the room, and show the scene asked for.
  let scale = $derived(room.w && room.h ? Math.min(room.w / plan.stage.width, room.h / plan.stage.height) : 0);

  $effect(() => {
    if (frame && scale) frame.fitTo(plan.stage.width * scale, plan.stage.height * scale);
  });

  $effect(() => {
    const f = frame;
    const i = index;
    const follow = plan.scenes[i]?.words.follow && plan.scenes[i]?.kind === "prayer";
    if (f) f.ready.then(() => f.show(i, { line: follow ? 0 : null }));
  });
</script>

<div bind:this={box} class="flex h-full w-full items-center justify-center overflow-hidden" data-testid="preview">
  <div class="overflow-hidden rounded-lg shadow-2xl ring-1 ring-edge" style="width: {plan.stage.width * scale}px; height: {plan.stage.height * scale}px;" data-testid="preview-frame">
    <div bind:this={host}></div>
  </div>
</div>
