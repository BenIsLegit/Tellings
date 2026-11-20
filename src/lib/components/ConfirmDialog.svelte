<script lang="ts">
  import { confirmState, resolveConfirm } from "$lib/stores/confirm";
  import Button from "./Button.svelte";
  import Modal from "./Modal.svelte";
</script>

<Modal open={$confirmState.open} title={$confirmState.title} testid="confirm-dialog" onclose={() => resolveConfirm(false)}>
  <p class="text-base leading-relaxed">{$confirmState.message}</p>
  {#snippet actions()}
    <Button testid="confirm-cancel-btn" onclick={() => resolveConfirm(false)}>{$confirmState.cancelLabel}</Button>
    <Button testid="confirm-ok-btn" variant={$confirmState.variant === "danger" ? "danger" : "primary"} onclick={() => resolveConfirm(true)}>
      {$confirmState.confirmLabel}
    </Button>
  {/snippet}
</Modal>
