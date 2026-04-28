<script lang="ts">
  // Phase 0 placeholder. The full SkeinWindow shell lands in Phase 1
  // (port of design/mockups/skein-app.jsx to Svelte components).
</script>

<main>
  <h1>Skein</h1>
  <p class="muted">Phase 0 — bootstrap. UI shell lands in Phase 1.</p>
</main>

<style>
  main {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    background: #f0eee9;
    color: #222;
    font-family: "Inter", system-ui, sans-serif;
  }
  h1 {
    font-family: "Source Serif 4", Georgia, serif;
    font-weight: 600;
    margin: 0;
    font-size: 2.4rem;
    letter-spacing: -0.01em;
  }
  .muted {
    color: #888;
    font-size: 0.95rem;
    margin: 0;
  }
  @media (prefers-color-scheme: dark) {
    main {
      background: #1a1a1c;
      color: #e8e6df;
    }
    .muted {
      color: #888;
    }
  }
</style>
