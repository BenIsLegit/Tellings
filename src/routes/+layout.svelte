<script lang="ts">
  import "../app.css";
  import { attachConsole } from "@tauri-apps/plugin-log";
  import { Toaster } from "svelte-sonner";
  import ConfirmDialog from "$lib/components/ConfirmDialog.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import { loadOptions } from "$lib/stores/options.svelte";
  import "$lib/stores/theme";

  let { children } = $props();

  $effect(() => {
    // Show the engine's log lines in the window's own console too.
    attachConsole();
    loadOptions();
  });
</script>

<div class="flex h-screen overflow-hidden bg-canvas text-ink" data-testid="app-shell">
  <Sidebar />
  <main class="min-w-0 flex-1 overflow-hidden" data-testid="app-main">{@render children()}</main>
</div>
<ConfirmDialog />
<Toaster theme="dark" position="bottom-right" richColors />
