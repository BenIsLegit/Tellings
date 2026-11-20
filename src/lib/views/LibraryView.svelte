<script lang="ts">
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { Clapperboard, FolderOpen, Loader2, SlidersHorizontal, Trash2, Upload } from "lucide-svelte";
  import { api } from "$lib/api";
  import Button from "$lib/components/Button.svelte";
  import RenderDialog from "$lib/components/RenderDialog.svelte";
  import { confirmDialog } from "$lib/stores/confirm";
  import { editor } from "$lib/stores/editor.svelte";
  import { navigate } from "$lib/stores/navigation";
  import type { Doc, Entry } from "$lib/types";
  import { invokeWithToast } from "$lib/utils/errors";

  let presets = $state<Entry[]>([]);
  let saved = $state<Entry[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let making = $state<Doc | null>(null);

  async function load() {
    try {
      const list = await api.libraryList();
      presets = list.presets;
      saved = list.saved;
      error = null;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    load();
  });

  async function edit(entry: Entry) {
    const ok = await invokeWithToast(() => editor.open(entry.id).then(() => true));
    if (ok) navigate("editor");
  }

  async function make(entry: Entry) {
    const opened = await invokeWithToast(() => api.setupOpen(entry.id));
    if (opened) making = opened.doc;
  }

  async function remove(entry: Entry) {
    const ok = await confirmDialog({ title: "Delete this setup?", message: `“${entry.name}” will be removed from your saved setups. Videos already made are not touched.`, confirmLabel: "Delete", variant: "danger" });
    if (!ok) return;
    await invokeWithToast(() => api.setupDelete(entry.id), "Deleted");
    if (editor.entry?.id === entry.id) editor.close();
    await load();
  }

  async function bringIn() {
    const path = await openDialog({ multiple: false, filters: [{ name: "Setup", extensions: ["json"] }] });
    if (typeof path !== "string") return;
    const entry = await invokeWithToast(() => api.setupImport(path), "Setup added");
    if (entry) await load();
  }
</script>

{#snippet card(entry: Entry)}
  <article class="flex flex-col rounded-lg border border-edge bg-panel p-5" data-testid="library-item">
    <h3 class="text-lg font-semibold text-ink">{entry.name}</h3>
    <p class="mt-1 flex-1 text-sm leading-relaxed text-ink-secondary">
      {entry.description || (entry.extends ? `Starts from “${entry.extends}”.` : "Your own setup.")}
    </p>
    <div class="mt-4 flex items-center gap-2">
      <Button variant="primary" testid="library-make-btn" onclick={() => make(entry)}><Clapperboard size={16} /> Make</Button>
      <Button testid="library-edit-btn" onclick={() => edit(entry)}><SlidersHorizontal size={16} /> Change it</Button>
      {#if entry.kind === "saved"}
        <span class="ml-auto"><Button variant="ghost" title="Delete this setup" testid="library-delete-btn" onclick={() => remove(entry)}><Trash2 size={16} /></Button></span>
      {/if}
    </div>
  </article>
{/snippet}

<div class="h-full overflow-y-auto p-6" data-testid="library-view">
  <div class="mx-auto max-w-6xl space-y-8">
    <header class="flex items-end justify-between gap-4">
      <div>
        <h1 class="text-2xl font-bold text-ink">Setups</h1>
        <p class="mt-1 text-base text-ink-secondary">Pick one and press Make. Or change anything first.</p>
      </div>
      <Button testid="library-import-btn" onclick={bringIn}><Upload size={16} /> Add a setup someone sent you</Button>
    </header>

    {#if loading}
      <div class="flex items-center justify-center py-12" data-testid="library-loading">
        <Loader2 size={24} class="animate-spin text-ink-muted" />
        <span class="ml-3 text-sm text-ink-muted">Loading…</span>
      </div>
    {:else if error}
      <div class="rounded-lg border border-red-900/50 bg-red-900/20 p-4" data-testid="library-error"><p class="text-sm text-red-400">{error}</p></div>
    {:else}
      <section data-testid="library-saved">
        <h2 class="mb-3 text-lg font-semibold text-ink">Your saved setups</h2>
        {#if saved.length === 0}
          <div class="flex flex-col items-center justify-center rounded-lg border border-dashed border-frame py-10 text-ink-muted" data-testid="library-empty-state">
            <FolderOpen size={48} class="mb-3 text-ink-ghost" />
            <p class="text-sm">None yet. Change a preset and save it as your own.</p>
          </div>
        {:else}
          <div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
            {#each saved as entry (entry.id)}{@render card(entry)}{/each}
          </div>
        {/if}
      </section>

      <section data-testid="library-presets">
        <h2 class="mb-3 text-lg font-semibold text-ink">Presets</h2>
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
          {#each presets as entry (entry.id)}{@render card(entry)}{/each}
        </div>
      </section>
    {/if}
  </div>
</div>

<RenderDialog open={making !== null} doc={making} onclose={() => (making = null)} />
