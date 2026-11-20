import { toast } from "svelte-sonner";

/** Run something that talks to the engine. Failures become a toast, not a crash. */
export async function invokeWithToast<T>(fn: () => Promise<T>, successMsg?: string): Promise<T | null> {
  try {
    const r = await fn();
    if (successMsg) toast.success(successMsg);
    return r;
  } catch (e) {
    toast.error(String(e));
    return null;
  }
}
