import { writable } from "svelte/store";

export type Theme = "dark" | "light" | "system";

const stored = (typeof localStorage !== "undefined" ? localStorage.getItem("theme") : null) as Theme | null;

export const theme = writable<Theme>(stored || "system");

theme.subscribe((value) => {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem("theme", value);
  const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  const isDark = value === "dark" || (value === "system" && prefersDark);
  document.documentElement.classList.toggle("dark", isDark);
});
