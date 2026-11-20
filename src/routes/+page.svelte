<script lang="ts">
  import { currentView } from "$lib/stores/navigation";
  import EditorView from "$lib/views/EditorView.svelte";
  import LibraryView from "$lib/views/LibraryView.svelte";
  import SettingsView from "$lib/views/SettingsView.svelte";
</script>

{#if $currentView === "library"}
  <LibraryView />
{:else if $currentView === "editor"}
  <EditorView />
{:else if $currentView === "settings"}
  <SettingsView />
{/if}
