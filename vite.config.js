import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [sveltekit(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    hmr: { port: 1421 },
    watch: { ignored: ["**/src-tauri/**", "**/target/**", "**/crates/**"] },
  },
});
