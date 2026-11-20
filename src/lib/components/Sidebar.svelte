<script lang="ts">
  import { Library, Settings, SlidersHorizontal } from "lucide-svelte";
  import { editor } from "$lib/stores/editor.svelte";
  import { currentView, navigate, type View } from "$lib/stores/navigation";

  const items: { view: View; label: string; icon: typeof Library }[] = [
    { view: "library", label: "Setups", icon: Library },
    { view: "editor", label: "Editor", icon: SlidersHorizontal },
    { view: "settings", label: "Settings", icon: Settings },
  ];
</script>

<aside class="flex h-full w-56 shrink-0 flex-col border-r border-edge bg-panel" data-testid="sidebar">
  <div class="px-4 pt-5 pb-4">
    <p class="text-base font-semibold text-ink">Prayer Video Creator</p>
    <p class="mt-0.5 text-xs text-ink-muted">Follow the gold bead.</p>
  </div>
  <nav class="flex-1 space-y-1 px-2" data-testid="sidebar-nav">
    {#each items as item (item.view)}
      {@const disabled = item.view === "editor" && !editor.isOpen}
      <button
        type="button"
        {disabled}
        title={disabled ? "Open a setup first" : ""}
        data-testid="sidebar-{item.view}-btn"
        onclick={() => navigate(item.view)}
        class="flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors disabled:cursor-not-allowed disabled:opacity-40
          {$currentView === item.view ? 'bg-brand/10 font-medium text-brand' : 'text-ink-muted hover:bg-well/50 hover:text-ink-secondary'}"
      >
        <item.icon size={18} />
        <span class="truncate">{item.view === "editor" && editor.isOpen ? editor.name || "Editor" : item.label}</span>
      </button>
    {/each}
  </nav>
</aside>
