import { writable } from "svelte/store";

interface ConfirmState {
  open: boolean;
  title: string;
  message: string;
  confirmLabel: string;
  cancelLabel: string;
  variant: "danger" | "default";
  resolve: ((value: boolean) => void) | null;
}

const initial: ConfirmState = {
  open: false,
  title: "",
  message: "",
  confirmLabel: "Confirm",
  cancelLabel: "Cancel",
  variant: "default",
  resolve: null,
};

export const confirmState = writable<ConfirmState>(initial);

/** Ask a yes-or-no question in the app's own dialog. Never the browser's. */
export function confirmDialog(opts: { title: string; message: string; confirmLabel?: string; cancelLabel?: string; variant?: "danger" | "default" }): Promise<boolean> {
  return new Promise((resolve) => {
    confirmState.set({
      open: true,
      title: opts.title,
      message: opts.message,
      confirmLabel: opts.confirmLabel ?? "Confirm",
      cancelLabel: opts.cancelLabel ?? "Cancel",
      variant: opts.variant ?? "default",
      resolve,
    });
  });
}

export function resolveConfirm(value: boolean) {
  confirmState.update((s) => {
    s.resolve?.(value);
    return { ...initial };
  });
}
