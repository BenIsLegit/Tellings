import { writable } from "svelte/store";

export type View = "library" | "editor" | "settings";

export const currentView = writable<View>("library");

export function navigate(view: View) {
  currentView.set(view);
}
