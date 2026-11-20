<script lang="ts">
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { CircleAlert, CircleCheck, FolderOpen, KeyRound } from "lucide-svelte";
  import { api } from "$lib/api";
  import Badge from "$lib/components/Badge.svelte";
  import Button from "$lib/components/Button.svelte";
  import Card from "$lib/components/Card.svelte";
  import Select from "$lib/components/Select.svelte";
  import TextInput from "$lib/components/TextInput.svelte";
  import { confirmDialog } from "$lib/stores/confirm";
  import { theme, type Theme } from "$lib/stores/theme";
  import type { Credits, Doctor, Settings } from "$lib/types";
  import { invokeWithToast } from "$lib/utils/errors";
  import { commas, megabytes } from "$lib/utils/format";

  let doctor = $state<Doctor | null>(null);
  let settings = $state<Settings>({});
  let credits = $state<Credits | null>(null);
  let key = $state("");
  let savingKey = $state(false);

  async function load() {
    doctor = await invokeWithToast(() => api.doctor());
    settings = (await invokeWithToast(() => api.settingsGet())) ?? {};
    if (doctor?.key) credits = await api.credits().catch(() => null);
  }

  $effect(() => {
    load();
  });

  async function saveKey() {
    savingKey = true;
    // The key is checked against ElevenLabs before it is kept.
    const ok = await invokeWithToast(() => api.keySet(key), "Key saved in this computer's password store");
    savingKey = false;
    if (ok) {
      key = "";
      credits = ok;
      await load();
    }
  }

  async function removeKey() {
    const ok = await confirmDialog({ title: "Remove the key?", message: "The ElevenLabs API key will be removed from this computer's password store. Speech already recorded is kept.", confirmLabel: "Remove", variant: "danger" });
    if (!ok) return;
    await invokeWithToast(() => api.keyClear(), "Key removed");
    credits = null;
    await load();
  }

  async function change(patch: Settings) {
    settings = { ...settings, ...patch };
    await invokeWithToast(() => api.settingsSet(settings));
    await load();
  }

  async function chooseFolder() {
    const picked = await openDialog({ directory: true });
    if (typeof picked === "string") await change({ outputFolder: picked });
  }

  async function clearSpeech() {
    const ok = await confirmDialog({
      title: "Forget all recorded speech?",
      message: "Every prayer recorded so far will be deleted. The next video with a voice will have to record them again, and that spends credits. Music made by ElevenLabs is not deleted.",
      confirmLabel: "Delete the recordings",
      variant: "danger",
    });
    if (!ok) return;
    await invokeWithToast(() => api.clearKeptSpeech(), "Recorded speech cleared");
    await load();
  }
</script>

<div class="h-full overflow-y-auto p-6" data-testid="settings-view">
  <div class="mx-auto max-w-3xl space-y-6">
    <h1 class="text-2xl font-bold text-ink">Settings</h1>

    <Card title="ElevenLabs voice" testid="settings-key-card">
      <div class="space-y-3 text-sm text-ink-secondary">
        <p class="flex items-center gap-2" data-testid="settings-key-status">
          {#if doctor?.key === "password-store"}
            <Badge tone="good"><KeyRound size={12} /> Key saved</Badge> It is kept in this computer's password store.
          {:else if doctor?.key === "environment"}
            <Badge tone="good"><KeyRound size={12} /> Key set</Badge> It comes from the ELEVENLABS_API_KEY environment variable.
          {:else}
            <Badge tone="warn">No key</Badge> Needed only for a generated voice.
          {/if}
        </p>
        {#if credits}
          <p data-testid="settings-credits">
            {commas(Math.max(0, credits.limit - credits.used))} credits left of {commas(credits.limit)} this period.
            {#if credits.resets}The count starts again on {new Date(credits.resets * 1000).toLocaleDateString(undefined, { day: "numeric", month: "long" })}.{/if}
          </p>
        {/if}
        <div class="flex gap-2">
          <TextInput type="password" value={key} placeholder="Paste your ElevenLabs API key" testid="settings-key-input" onchange={(v) => (key = v)} />
          <Button variant="primary" disabled={!key.trim() || savingKey} testid="settings-key-save-btn" onclick={saveKey}>{savingKey ? "Checking…" : "Save key"}</Button>
          {#if doctor?.key === "password-store"}<Button testid="settings-key-remove-btn" onclick={removeKey}>Remove</Button>{/if}
        </div>
        <p class="text-xs leading-relaxed text-ink-muted">The key is never written to a setup, a log or a video. Find it at elevenlabs.io under your profile, API keys.</p>
      </div>
    </Card>

    <Card title="Where videos go" testid="settings-folders-card">
      <div class="space-y-3 text-sm text-ink-secondary">
        <p class="truncate rounded-md border border-edge bg-well px-3 py-2 text-ink" title={doctor?.videosFolder} data-testid="settings-videos-folder">{doctor?.videosFolder ?? ""}</p>
        <div class="flex flex-wrap gap-2">
          <Button testid="settings-folder-choose-btn" onclick={chooseFolder}>Choose a folder…</Button>
          {#if settings.outputFolder}<Button variant="ghost" testid="settings-folder-reset-btn" onclick={() => change({ outputFolder: null })}>Use the usual folder</Button>{/if}
          <Button variant="ghost" testid="settings-folder-open-btn" onclick={() => doctor && invokeWithToast(() => api.openPath(doctor!.videosFolder))}><FolderOpen size={16} /> Open it</Button>
          <Button variant="ghost" testid="settings-pictures-open-btn" onclick={() => doctor && invokeWithToast(() => api.openPath(doctor!.picturesFolder))}><FolderOpen size={16} /> Open my pictures folder</Button>
        </div>
      </div>
    </Card>

    <Card title="Recorded speech" testid="settings-speech-card">
      <div class="space-y-3 text-sm text-ink-secondary">
        <p>Each prayer is recorded once per voice and kept, so it is never paid for twice. <span class="text-ink" data-testid="settings-speech-size">{megabytes(doctor?.keptSpeechBytes ?? 0)} kept.</span></p>
        <p data-testid="settings-music-kept">Music made by ElevenLabs is kept as well. The button below leaves it alone.</p>
        <Button variant="danger" disabled={!doctor?.keptSpeechBytes} testid="settings-speech-clear-btn" onclick={clearSpeech}>Forget all recorded speech…</Button>
      </div>
    </Card>

    <Card title="This computer" testid="settings-tools-card">
      <div class="space-y-4 text-sm text-ink-secondary">
        {#each [{ name: "FFmpeg", what: "makes the sound and the video", path: doctor?.ffmpeg, problem: doctor?.ffmpegProblem, field: "ffmpeg" as const }, { name: "Chrome or Edge", what: "draws each bead", path: doctor?.browser, problem: doctor?.browserProblem, field: "browser" as const }] as tool (tool.name)}
          <div data-testid="settings-tool-{tool.field}">
            <p class="flex items-center gap-2 font-medium text-ink">
              {#if tool.path}<CircleCheck size={16} class="text-emerald-400" />{:else}<CircleAlert size={16} class="text-red-400" />{/if}
              {tool.name} <span class="font-normal text-ink-muted">{tool.what}</span>
            </p>
            <p class="mt-1 truncate text-xs {tool.path ? 'text-ink-muted' : 'text-red-400'}" title={tool.path ?? ""}>{tool.path ?? tool.problem ?? "Checking…"}</p>
            <div class="mt-2">
              <TextInput value={settings[tool.field] ?? ""} placeholder="Found on its own. Type a path here only if it is somewhere unusual." mono testid="settings-{tool.field}-input" onchange={(v) => change({ [tool.field]: v.trim() || null })} />
            </div>
          </div>
        {/each}
      </div>
    </Card>

    <Card title="How the app looks" testid="settings-theme-card">
      <div class="max-w-xs">
        <Select value={$theme} options={[["system", "Match this computer"], ["dark", "Dark"], ["light", "Light"]]} testid="settings-theme-select" onchange={(v) => theme.set(v as Theme)} />
      </div>
      <p class="mt-2 text-xs text-ink-muted">This is the app only. A video's colors are set in its own setup.</p>
    </Card>

    <p class="pb-4 text-xs text-ink-faint" data-testid="settings-about">
      Prayer Video Creator {doctor?.version ?? ""} · Setups and recorded speech live in
      <button type="button" class="underline hover:text-ink-muted" data-testid="settings-data-open-btn" onclick={() => doctor && invokeWithToast(() => api.openPath(doctor!.dataFolder))}>{doctor?.dataFolder ?? ""}</button>
    </p>
  </div>
</div>
