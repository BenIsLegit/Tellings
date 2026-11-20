<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    variant = "secondary",
    size = "md",
    disabled = false,
    title,
    testid = "button",
    onclick,
    children,
  }: {
    variant?: "primary" | "secondary" | "danger" | "ghost";
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
    title?: string;
    testid?: string;
    onclick?: (e: MouseEvent) => void;
    children: Snippet;
  } = $props();

  const variants = {
    primary: "bg-brand font-medium text-on-brand hover:bg-brand-hover",
    secondary: "border border-frame text-ink-secondary hover:bg-well",
    danger: "bg-red-600 font-medium text-white hover:bg-red-700",
    ghost: "text-ink-muted hover:bg-well hover:text-ink-secondary",
  };
  const sizes = { sm: "px-2 py-1 text-xs", md: "px-3 py-1.5 text-sm", lg: "px-5 py-2.5 text-base" };
</script>

<button
  type="button"
  data-testid={testid}
  {title}
  {disabled}
  {onclick}
  class="inline-flex shrink-0 items-center justify-center gap-2 rounded-md transition-colors disabled:cursor-not-allowed disabled:opacity-50 {variants[variant]} {sizes[size]}"
>
  {@render children()}
</button>
