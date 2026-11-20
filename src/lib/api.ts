// Every call the window makes to the engine. The window itself never touches
// files, the network or the password store.
import { invoke } from "@tauri-apps/api/core";
import type { Checked, Credits, Doc, Doctor, Entry, KeySource, Made, Options, Settings, VoiceInfo } from "./types";

export const api = {
  libraryList: () => invoke<{ presets: Entry[]; saved: Entry[] }>("library_list"),
  setupOpen: (id: string) => invoke<{ entry: Entry; doc: Doc }>("setup_open", { id }),
  setupLook: (doc: Doc) => invoke<Checked>("setup_look", { doc }),
  setupSave: (name: string, doc: Doc) => invoke<Entry>("setup_save", { name, doc }),
  setupDelete: (id: string) => invoke<void>("setup_delete", { id }),
  setupImport: (path: string) => invoke<Entry>("setup_import", { path }),
  setupExport: (doc: Doc, path: string) => invoke<void>("setup_export", { doc, path }),
  options: () => invoke<Options>("options"),
  pictureData: (path: string) => invoke<string>("picture_data", { path }),

  keyStatus: () => invoke<KeySource | null>("key_status"),
  keySet: (key: string) => invoke<Credits>("key_set", { key }),
  keyClear: () => invoke<void>("key_clear"),
  voices: (refresh = false) => invoke<VoiceInfo[]>("voices", { refresh }),
  credits: () => invoke<Credits | null>("credits"),

  renderStart: (doc: Doc, approvedLetters: number) => invoke<Made>("render_start", { doc, approvedLetters }),
  renderCancel: () => invoke<void>("render_cancel"),

  doctor: () => invoke<Doctor>("doctor"),
  settingsGet: () => invoke<Settings>("settings_get"),
  settingsSet: (settings: Settings) => invoke<void>("settings_set", { settings }),
  clearKeptSpeech: () => invoke<void>("clear_kept_speech"),
  openPath: (path: string) => invoke<void>("open_path", { path }),
  revealPath: (path: string) => invoke<void>("reveal_path", { path }),
};
