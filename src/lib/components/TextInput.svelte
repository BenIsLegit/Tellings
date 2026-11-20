<script lang="ts">
  // A text box that reports its value when the person stops typing or leaves it,
  // so a setup is not re-planned on every keystroke.
  let {
    value,
    placeholder = "",
    type = "text",
    mono = false,
    testid = "text-input",
    onchange,
  }: {
    value: string;
    placeholder?: string;
    type?: "text" | "password";
    mono?: boolean;
    testid?: string;
    onchange: (value: string) => void;
  } = $props();

  let draft = $state("");
  let editing = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  $effect(() => {
    if (!editing) draft = value ?? "";
  });

  function commit() {
    clearTimeout(timer);
    if (draft !== (value ?? "")) onchange(draft);
  }
</script>

<input
  {type}
  {placeholder}
  data-testid={testid}
  value={draft}
  spellcheck="false"
  autocomplete="off"
  oninput={(e) => {
    draft = e.currentTarget.value;
    clearTimeout(timer);
    timer = setTimeout(commit, 500);
  }}
  onfocus={() => (editing = true)}
  onblur={() => {
    editing = false;
    commit();
  }}
  onkeydown={(e) => {
    if (e.key === "Enter") commit();
  }}
  class="w-full min-w-0 rounded-md border border-frame bg-well px-3 py-2 text-sm text-ink placeholder-ink-faint focus:border-brand focus:outline-none {mono ? 'font-mono text-xs' : ''}"
/>
