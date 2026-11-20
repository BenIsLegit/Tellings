// The lists the menus are built from: screen sizes, prayers, sets of mysteries.
// They come from the engine once, so the window never keeps its own copy of the prayers.
import { api } from "$lib/api";
import type { Options } from "$lib/types";

export const options = $state<{ value: Options | null }>({ value: null });

export async function loadOptions() {
  if (!options.value) options.value = await api.options();
}
