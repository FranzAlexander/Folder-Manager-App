<script lang="ts">
  // Importing the stylesheet here puts it in Vite's module graph, so Tailwind
  // is processed and the CSS is bundled into the production build (a raw <link>
  // in app.html only works in dev).
  import "../app.css";

  let { children } = $props();
</script>

{@render children()}
