// skein-app.jsx — the Skein desktop window, parameterized by props
// Renders the full four-region layout: GNOME title+menu, bookshelf, desk, chat sidebar.
// Props drive variation: theme, shelf style, sidebar mode, view mode, dragging, content set.

const SKEIN_CSS = `
  /* ─── shared variables, dark default ─── */
  .skein {
    --wood-1: oklch(0.32 0.035 55);
    --wood-2: oklch(0.27 0.030 55);
    --wood-3: oklch(0.22 0.025 55);
    --wood-edge: oklch(0.16 0.020 50);
    --wood-shadow: oklch(0.10 0.015 50 / 0.6);

    --paper: oklch(0.20 0.008 60);
    --paper-2: oklch(0.24 0.009 65);
    --page: oklch(0.27 0.011 70);
    --page-edge: oklch(0.34 0.013 70);

    --ink: oklch(0.92 0.012 80);
    --ink-2: oklch(0.78 0.015 75);
    --ink-3: oklch(0.60 0.012 70);
    --ink-4: oklch(0.42 0.010 65);

    --chrome: oklch(0.18 0.006 60);
    --chrome-2: oklch(0.22 0.008 60);
    --chrome-edge: oklch(0.13 0.005 55);
    --chrome-ink: oklch(0.78 0.010 75);
    --chrome-ink-2: oklch(0.55 0.010 70);

    --accent: oklch(0.78 0.13 75);     /* warm amber */
    --accent-soft: oklch(0.78 0.13 75 / 0.18);
    --accent-edge: oklch(0.78 0.13 75 / 0.55);

    --user-msg: oklch(0.30 0.012 65);
    --asst-msg: oklch(0.24 0.009 60);

    --shadow-page: 0 1px 0 oklch(1 0 0 / 0.04) inset, 0 18px 40px -16px oklch(0 0 0 / 0.6), 0 6px 16px -8px oklch(0 0 0 / 0.5);

    color: var(--ink);
    font-family: 'Inter', system-ui, sans-serif;
    font-size: 13px;
    -webkit-font-smoothing: antialiased;
  }
  .skein.theme-light {
    --wood-1: oklch(0.62 0.055 55);
    --wood-2: oklch(0.55 0.050 55);
    --wood-3: oklch(0.48 0.045 55);
    --wood-edge: oklch(0.36 0.035 50);
    --wood-shadow: oklch(0.30 0.025 50 / 0.4);

    --paper: oklch(0.95 0.012 85);
    --paper-2: oklch(0.92 0.013 85);
    --page: oklch(0.985 0.008 90);
    --page-edge: oklch(0.88 0.013 85);

    --ink: oklch(0.22 0.015 60);
    --ink-2: oklch(0.36 0.018 60);
    --ink-3: oklch(0.50 0.015 60);
    --ink-4: oklch(0.65 0.012 60);

    --chrome: oklch(0.96 0.008 80);
    --chrome-2: oklch(0.93 0.010 80);
    --chrome-edge: oklch(0.85 0.010 75);
    --chrome-ink: oklch(0.30 0.010 60);
    --chrome-ink-2: oklch(0.50 0.010 60);

    --accent: oklch(0.62 0.15 55);
    --accent-soft: oklch(0.62 0.15 55 / 0.14);
    --accent-edge: oklch(0.62 0.15 55 / 0.55);

    --user-msg: oklch(0.90 0.013 80);
    --asst-msg: oklch(0.95 0.010 80);

    --shadow-page: 0 1px 0 oklch(1 0 0 / 0.7) inset, 0 14px 30px -14px oklch(0.30 0.020 50 / 0.45), 0 4px 12px -6px oklch(0.30 0.020 50 / 0.30);
  }

  .skein .win {
    display: flex; flex-direction: column;
    width: 100%; height: 100%;
    border-radius: 10px;
    overflow: hidden;
    background: var(--paper);
    box-shadow: 0 0 0 1px oklch(0 0 0 / 0.5), 0 30px 80px -20px oklch(0 0 0 / 0.55);
    isolation: isolate;
  }
  .skein.theme-light .win { box-shadow: 0 0 0 1px oklch(0.30 0.020 50 / 0.18), 0 30px 80px -20px oklch(0.30 0.020 50 / 0.35); }

  /* ─── GNOME-style headerbar (title + menu merged in modern GNOME) ─── */
  .sk-titlebar {
    display: flex; align-items: center;
    height: 38px;
    padding: 0 8px 0 12px;
    background: var(--chrome);
    border-bottom: 1px solid var(--chrome-edge);
    color: var(--chrome-ink);
    gap: 8px;
    flex-shrink: 0;
  }
  .sk-tb-traffic { display: none; } /* GNOME puts close on the right */
  .sk-tb-menus {
    display: flex; gap: 2px; align-items: center;
  }
  .sk-tb-menu {
    padding: 4px 9px;
    border-radius: 6px;
    font-size: 12.5px;
    color: var(--chrome-ink);
    cursor: default;
  }
  .sk-tb-menu:hover { background: oklch(1 0 0 / 0.06); }
  .skein.theme-light .sk-tb-menu:hover { background: oklch(0 0 0 / 0.05); }
  .sk-tb-title {
    flex: 1;
    text-align: center;
    font-size: 12.5px;
    font-weight: 600;
    color: var(--chrome-ink);
    letter-spacing: 0.01em;
  }
  .sk-tb-title .vault {
    color: var(--chrome-ink-2);
    font-weight: 400;
  }
  .sk-tb-right { display: flex; align-items: center; gap: 4px; }
  .sk-tb-btn {
    width: 26px; height: 26px;
    border-radius: 6px;
    display: flex; align-items: center; justify-content: center;
    color: var(--chrome-ink-2);
    cursor: default;
  }
  .sk-tb-btn:hover { background: oklch(1 0 0 / 0.06); color: var(--chrome-ink); }
  .skein.theme-light .sk-tb-btn:hover { background: oklch(0 0 0 / 0.05); color: var(--chrome-ink); }
  .sk-tb-window {
    display: flex; gap: 4px; margin-left: 4px;
  }
  .sk-tb-window button {
    width: 22px; height: 22px;
    border: 0;
    border-radius: 50%;
    background: oklch(1 0 0 / 0.07);
    color: var(--chrome-ink-2);
    display: flex; align-items: center; justify-content: center;
    cursor: default;
    padding: 0;
  }
  .skein.theme-light .sk-tb-window button { background: oklch(0 0 0 / 0.06); }
  .sk-tb-window button:hover { background: oklch(1 0 0 / 0.12); }
  .skein.theme-light .sk-tb-window button:hover { background: oklch(0 0 0 / 0.10); }
  .sk-tb-window button.close:hover { background: oklch(0.65 0.18 25); color: white; }

  /* ─── body grid: bookshelf top, then desk + sidebar ─── */
  .sk-body {
    flex: 1;
    display: grid;
    grid-template-rows: auto 1fr;
    min-height: 0;
  }
  .sk-mid {
    display: grid;
    grid-template-columns: 1fr 340px;
    min-height: 0;
  }
  .sk-mid.sidebar-collapsed { grid-template-columns: 1fr 36px; }
  .sk-mid.sidebar-hidden    { grid-template-columns: 1fr; }

  /* ─── BOOKSHELF ─── */
  .sk-shelf {
    background: linear-gradient(180deg, var(--wood-2), var(--wood-3) 60%, var(--wood-edge));
    padding: 14px 16px 0;
    position: relative;
    border-bottom: 1px solid var(--wood-edge);
  }
  .sk-shelf.style-tactile {
    background:
      repeating-linear-gradient(90deg,
        oklch(from var(--wood-2) calc(l - 0.02) c h) 0 13px,
        var(--wood-2) 13px 26px,
        oklch(from var(--wood-2) calc(l + 0.01) c h) 26px 41px),
      linear-gradient(180deg, var(--wood-2), var(--wood-3) 60%, var(--wood-edge));
    background-blend-mode: overlay, normal;
  }
  .sk-shelf.style-abstract {
    background: var(--chrome-2);
    border-bottom: 1px solid var(--chrome-edge);
  }
  .sk-shelf-row {
    position: relative;
    height: 92px;
    display: flex;
    align-items: flex-end;
    gap: 4px;
    padding: 0 4px;
  }
  .sk-shelf-row + .sk-shelf-row { margin-top: 18px; }
  /* the wooden plank under each row */
  .sk-shelf-row::after {
    content: '';
    position: absolute;
    left: -16px; right: -16px;
    bottom: -10px;
    height: 10px;
    background: linear-gradient(180deg,
      oklch(from var(--wood-1) calc(l - 0.04) c h),
      oklch(from var(--wood-edge) calc(l - 0.04) c h));
    box-shadow: 0 2px 4px oklch(0 0 0 / 0.4);
    z-index: 0;
  }
  .sk-shelf.style-abstract .sk-shelf-row::after {
    background: var(--chrome-edge);
    box-shadow: none;
    height: 1px;
    bottom: 0;
  }
  .sk-shelf.style-abstract .sk-shelf-row { padding-bottom: 0; }

  /* spine */
  .sk-spine {
    --spine-w: 28px;
    --spine-h: 80px;
    --spine-bg: var(--wood-2);
    --spine-band: var(--accent);
    width: var(--spine-w);
    height: var(--spine-h);
    background:
      linear-gradient(90deg,
        oklch(from var(--spine-bg) calc(l - 0.04) c h) 0%,
        var(--spine-bg) 12%,
        oklch(from var(--spine-bg) calc(l + 0.03) c h) 50%,
        var(--spine-bg) 88%,
        oklch(from var(--spine-bg) calc(l - 0.06) c h) 100%);
    border-radius: 2px 2px 0 0;
    border-top: 1px solid oklch(from var(--spine-bg) calc(l + 0.06) c h);
    position: relative;
    flex-shrink: 0;
    z-index: 1;
    cursor: default;
    transition: transform 0.15s ease;
    box-shadow: 0 1px 2px oklch(0 0 0 / 0.3);
  }
  .sk-spine:hover { transform: translateY(-2px); }
  .sk-spine.active {
    transform: translateY(-6px);
    box-shadow: 0 6px 12px -2px oklch(0 0 0 / 0.5), 0 0 0 1px var(--accent-edge);
  }
  .sk-spine.active::before {
    content: '';
    position: absolute;
    inset: -3px;
    border-radius: 3px 3px 0 0;
    background: var(--accent-soft);
    z-index: -1;
    filter: blur(4px);
  }
  /* cloth band */
  .sk-spine .band {
    position: absolute;
    left: 0; right: 0;
    top: 14px;
    height: 14px;
    background: var(--spine-band);
    box-shadow: 0 1px 0 oklch(0 0 0 / 0.25), 0 -1px 0 oklch(0 0 0 / 0.25);
  }
  .sk-spine .band-2 {
    position: absolute;
    left: 0; right: 0;
    bottom: 8px;
    height: 4px;
    background: oklch(from var(--spine-band) calc(l - 0.08) c h);
  }
  /* vertical title */
  .sk-spine .title {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    writing-mode: vertical-rl;
    transform: rotate(180deg);
    font-family: 'Source Serif 4', serif;
    font-size: 10.5px;
    font-weight: 600;
    letter-spacing: 0.05em;
    color: oklch(from var(--spine-bg) calc(l + 0.55) calc(c * 0.3) h);
    text-transform: uppercase;
    padding: 18px 0 14px;
    overflow: hidden;
    white-space: nowrap;
    text-shadow: 0 1px 0 oklch(0 0 0 / 0.4);
  }
  .sk-spine .title.below-band { padding-top: 36px; }

  /* abstract spine style overrides */
  .sk-shelf.style-abstract .sk-spine {
    border-radius: 4px;
    border-top: 0;
    box-shadow: none;
    background: var(--spine-bg);
  }
  .sk-shelf.style-abstract .sk-spine .band { display: none; }
  .sk-shelf.style-abstract .sk-spine .band-2 {
    top: auto; bottom: 6px; height: 2px;
    background: var(--spine-band);
  }
  .sk-shelf.style-abstract .sk-spine .title {
    color: oklch(from var(--spine-bg) calc(l + 0.50) calc(c * 0.4) h);
    text-shadow: none;
  }

  /* folio: stack of papers tied with twine */
  .sk-folio {
    width: 38px; height: 80px;
    flex-shrink: 0;
    position: relative;
    z-index: 1;
    cursor: default;
  }
  .sk-folio .sheet {
    position: absolute;
    left: 0; right: 0;
    background: oklch(0.85 0.025 80);
    border: 1px solid oklch(0.55 0.020 75);
    border-radius: 1px;
  }
  .sk-folio .sheet:nth-child(1) { top: 4px; bottom: 0; transform: rotate(-3deg); background: oklch(0.78 0.030 80); }
  .sk-folio .sheet:nth-child(2) { top: 2px; bottom: 2px; transform: rotate(2deg); background: oklch(0.83 0.025 82); }
  .sk-folio .sheet:nth-child(3) { top: 0; bottom: 4px; background: oklch(0.88 0.020 85); }
  .sk-folio .twine {
    position: absolute;
    left: -3px; right: -3px;
    top: 38px;
    height: 3px;
    background: oklch(0.55 0.080 40);
    border-radius: 1px;
    z-index: 2;
    box-shadow: 0 1px 0 oklch(0 0 0 / 0.25);
    transform: rotate(-1deg);
  }
  .sk-folio .twine::before {
    content: '';
    position: absolute;
    left: 50%; top: -2px;
    width: 4px; height: 7px;
    background: oklch(0.55 0.080 40);
    transform: translateX(-50%) rotate(-15deg);
  }
  .sk-folio .label {
    position: absolute;
    inset: 0;
    display: flex; align-items: center; justify-content: center;
    writing-mode: vertical-rl;
    transform: rotate(180deg);
    font-family: 'Source Serif 4', serif;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.18em;
    color: oklch(0.30 0.020 60);
    text-transform: uppercase;
    z-index: 3;
    padding-top: 10px;
  }

  /* ─── DESK ─── */
  .sk-desk {
    background: var(--paper);
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    position: relative;
  }
  .sk-tabs {
    display: flex;
    align-items: stretch;
    background: var(--paper-2);
    border-bottom: 1px solid var(--chrome-edge);
    padding: 6px 8px 0;
    gap: 1px;
    height: 34px;
    flex-shrink: 0;
  }
  .sk-tab {
    display: flex; align-items: center; gap: 8px;
    padding: 0 12px;
    background: transparent;
    color: var(--ink-3);
    font-size: 12px;
    border-radius: 7px 7px 0 0;
    border: 1px solid transparent;
    border-bottom: 0;
    height: 100%;
    cursor: default;
    max-width: 200px;
    position: relative;
  }
  .sk-tab .pin-ind {
    width: 9px; height: 9px;
    color: var(--ink-4);
    flex-shrink: 0;
  }
  .sk-tab .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .sk-tab .x {
    width: 14px; height: 14px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 3px;
    color: var(--ink-4);
    font-size: 13px; line-height: 1;
    margin-left: 2px;
  }
  .sk-tab.active {
    background: var(--page);
    color: var(--ink);
    border-color: var(--chrome-edge);
  }
  .sk-tab.active.pinned-l, .sk-tab.active.pinned-r { color: var(--ink); }
  .sk-tab.pinned-l .pin-ind, .sk-tab.pinned-r .pin-ind { color: var(--accent); }
  .sk-tab .dirty { width: 6px; height: 6px; border-radius: 50%; background: var(--accent); }

  /* desk surface (where pages live) */
  .sk-surface {
    flex: 1;
    display: flex;
    gap: 18px;
    padding: 22px;
    min-height: 0;
    background: var(--paper);
  }
  .sk-page {
    flex: 1;
    background: var(--page);
    border: 1px solid var(--page-edge);
    border-radius: 4px;
    box-shadow: var(--shadow-page);
    overflow: hidden;
    display: flex; flex-direction: column;
    min-width: 0;
    position: relative;
  }
  .sk-page-inner {
    padding: 38px 56px 42px;
    overflow: hidden;
    flex: 1;
    font-family: var(--page-font, 'Source Serif 4'), Georgia, serif;
    font-size: 14.5px;
    line-height: 1.65;
    color: var(--ink);
  }
  .sk-page-inner h1 {
    font-family: 'Source Serif 4', Georgia, serif;
    font-weight: 600;
    font-size: 22px;
    margin: 0 0 6px;
    letter-spacing: -0.005em;
  }
  .sk-page-inner h2 {
    font-family: 'Source Serif 4', Georgia, serif;
    font-weight: 600;
    font-size: 16px;
    margin: 22px 0 6px;
    color: var(--ink);
  }
  .sk-page-inner h3 {
    font-family: 'Source Serif 4', Georgia, serif;
    font-weight: 600;
    font-size: 14px;
    margin: 16px 0 4px;
    color: var(--ink-2);
  }
  .sk-page-inner p { margin: 0 0 10px; }
  .sk-page-inner .muted { color: var(--ink-3); font-size: 12.5px; font-family: 'Inter', sans-serif; letter-spacing: 0.02em;}
  .sk-page-inner ul, .sk-page-inner ol { margin: 0 0 10px; padding-left: 22px; }
  .sk-page-inner li { margin-bottom: 3px; }
  .sk-page-inner .md-syntax { color: var(--ink-4); font-family: 'JetBrains Mono', monospace; font-size: 0.88em; }
  .sk-page-inner blockquote {
    margin: 12px 0;
    padding: 4px 14px;
    border-left: 2px solid var(--accent-edge);
    color: var(--ink-2);
    font-style: italic;
  }
  .sk-page-inner code {
    font-family: 'JetBrains Mono', monospace;
    font-size: 12px;
    background: oklch(from var(--page) calc(l - 0.04) c h);
    padding: 1px 5px;
    border-radius: 3px;
  }
  .sk-page-inner a.wiki {
    color: var(--accent);
    text-decoration: none;
    border-bottom: 1px dashed var(--accent-edge);
  }
  .sk-page-inner .tag {
    color: var(--accent);
    font-family: 'JetBrains Mono', monospace;
    font-size: 12px;
  }
  .sk-page-inner hr { border: 0; border-top: 1px solid var(--page-edge); margin: 16px 0; }

  /* the live cursor (caret) */
  .sk-caret {
    display: inline-block;
    width: 1.5px;
    height: 1.1em;
    background: var(--ink);
    vertical-align: text-bottom;
    margin: 0 -0.5px;
    animation: sk-blink 1.05s steps(2, jump-none) infinite;
  }
  .sk-caret.insert {
    background: var(--accent);
    width: 2px;
    box-shadow: 0 0 0 3px var(--accent-soft);
    animation: none;
  }
  @keyframes sk-blink { 50% { opacity: 0; } }

  /* selection on chat looking like a real text selection */
  .chat-selection {
    background: oklch(0.55 0.18 250 / 0.40);
    color: inherit;
    box-decoration-break: clone;
    -webkit-box-decoration-break: clone;
    padding: 1px 0;
  }
  .skein.theme-light .chat-selection {
    background: oklch(0.65 0.20 250 / 0.30);
  }

  /* drag preview floating near cursor */
  .sk-drag-preview {
    position: absolute;
    pointer-events: none;
    z-index: 50;
    max-width: 280px;
    background: oklch(from var(--page) calc(l + 0.02) c h);
    border: 1px solid var(--accent-edge);
    border-radius: 4px;
    padding: 8px 11px;
    font-family: 'Source Serif 4', serif;
    font-size: 12.5px;
    line-height: 1.45;
    color: var(--ink-2);
    box-shadow: 0 18px 40px -10px oklch(0 0 0 / 0.55), 0 4px 12px -4px oklch(0 0 0 / 0.4);
    transform: rotate(-1.5deg);
  }
  .sk-drag-preview::before {
    content: '↳ insert at caret';
    position: absolute;
    top: -22px; left: 0;
    background: var(--accent);
    color: oklch(0.20 0.020 60);
    font-family: 'Inter', sans-serif;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.03em;
    padding: 3px 8px;
    border-radius: 3px;
    text-transform: uppercase;
    white-space: nowrap;
  }
  .sk-cursor-icon {
    position: absolute;
    z-index: 51;
    pointer-events: none;
    color: var(--accent);
    filter: drop-shadow(0 1px 2px oklch(0 0 0 / 0.5));
  }

  /* ─── EMPTY STATE: scattered cards ─── */
  .sk-empty-stage {
    flex: 1;
    position: relative;
    background:
      radial-gradient(ellipse 80% 70% at 50% 60%, oklch(from var(--paper) calc(l + 0.02) c h), var(--paper) 70%);
    overflow: hidden;
  }
  .sk-empty-hint {
    position: absolute;
    top: 24px; left: 50%;
    transform: translateX(-50%);
    color: var(--ink-4);
    font-size: 12px;
    letter-spacing: 0.04em;
    text-align: center;
    font-family: 'Inter', sans-serif;
  }
  .sk-empty-hint b {
    display: block;
    font-family: 'Source Serif 4', serif;
    font-style: italic;
    font-weight: 400;
    font-size: 18px;
    color: var(--ink-3);
    margin-bottom: 4px;
    letter-spacing: -0.005em;
  }
  .sk-card {
    position: absolute;
    width: 200px; height: 132px;
    background: var(--page);
    border: 1px solid var(--page-edge);
    border-radius: 3px;
    box-shadow: var(--shadow-page);
    padding: 14px 16px;
    font-family: 'Source Serif 4', serif;
    overflow: hidden;
    cursor: default;
    transition: transform 0.18s ease, box-shadow 0.18s ease;
  }
  .sk-card:hover {
    transform: rotate(0deg) translateY(-3px) scale(1.02) !important;
    z-index: 10;
    box-shadow: 0 24px 50px -16px oklch(0 0 0 / 0.7), 0 8px 20px -8px oklch(0 0 0 / 0.55);
  }
  .sk-card .ttl {
    font-weight: 600;
    font-size: 13.5px;
    color: var(--ink);
    margin-bottom: 6px;
    line-height: 1.3;
    letter-spacing: -0.005em;
  }
  .sk-card .body {
    font-size: 11.5px;
    color: var(--ink-3);
    line-height: 1.5;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 5;
    -webkit-box-orient: vertical;
  }
  .sk-card .meta {
    position: absolute;
    bottom: 10px; left: 16px; right: 16px;
    font-family: 'Inter', sans-serif;
    font-size: 10px;
    color: var(--ink-4);
    letter-spacing: 0.04em;
    display: flex; justify-content: space-between;
  }

  /* ─── RIGHT SIDEBAR — chat ─── */
  .sk-side {
    background: var(--chrome-2);
    border-left: 1px solid var(--chrome-edge);
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
  }
  .sk-side .sk-msg { max-width: 100%; word-wrap: break-word; overflow-wrap: anywhere; }
  .sk-side-hd {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 10px 10px;
    border-bottom: 1px solid var(--chrome-edge);
    flex-shrink: 0;
  }
  .sk-pill {
    display: inline-flex; align-items: center; gap: 5px;
    height: 24px; padding: 0 8px;
    border: 1px solid var(--chrome-edge);
    border-radius: 6px;
    background: oklch(from var(--chrome-2) calc(l + 0.02) c h);
    color: var(--chrome-ink);
    font-size: 11.5px;
    cursor: default;
  }
  .sk-pill .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--accent); }
  .sk-pill .chev { color: var(--ink-4); font-size: 9px; }
  .sk-pill.context { color: var(--ink-2); }
  .sk-side-spacer { flex: 1; }
  .sk-side-icon {
    width: 26px; height: 26px;
    display: flex; align-items: center; justify-content: center;
    border-radius: 6px; color: var(--ink-3);
  }
  .sk-side-icon:hover { background: oklch(1 0 0 / 0.06); color: var(--ink-2); }
  .skein.theme-light .sk-side-icon:hover { background: oklch(0 0 0 / 0.05); color: var(--ink-2); }

  .sk-chat {
    flex: 1;
    overflow: hidden;
    padding: 10px 12px 14px;
    display: flex; flex-direction: column;
    gap: 10px;
    font-size: 12.5px;
    line-height: 1.5;
  }
  .sk-msg {
    border-radius: 8px;
    padding: 8px 11px;
    max-width: 100%;
  }
  .sk-msg.user {
    background: var(--user-msg);
    color: var(--ink);
    align-self: flex-end;
    margin-left: 18px;
  }
  .sk-msg.asst {
    background: transparent;
    color: var(--ink-2);
    padding: 4px 0;
  }
  .sk-msg.asst .who {
    font-size: 10.5px;
    font-weight: 600;
    color: var(--ink-3);
    letter-spacing: 0.05em;
    text-transform: uppercase;
    margin-bottom: 4px;
    display: flex; align-items: center; gap: 6px;
  }
  .sk-msg.asst .who .ico {
    width: 14px; height: 14px;
    border-radius: 4px;
    background: var(--accent-soft);
    color: var(--accent);
    display: flex; align-items: center; justify-content: center;
    font-family: 'Source Serif 4', serif; font-style: italic; font-size: 10px;
  }
  .sk-msg.asst p { margin: 0 0 7px; }
  .sk-msg.asst code {
    font-family: 'JetBrains Mono', monospace;
    font-size: 11.5px;
    background: oklch(from var(--chrome-2) calc(l + 0.04) c h);
    padding: 0 5px;
    border-radius: 3px;
  }
  .sk-msg .streaming-dot {
    display: inline-block;
    width: 6px; height: 13px;
    background: var(--accent);
    vertical-align: text-bottom;
    margin-left: 2px;
    animation: sk-blink 0.9s steps(2, jump-none) infinite;
  }

  .sk-input {
    margin: 0 12px 12px;
    background: oklch(from var(--chrome-2) calc(l + 0.03) c h);
    border: 1px solid var(--chrome-edge);
    border-radius: 9px;
    padding: 9px 10px 8px;
    color: var(--ink-3);
    flex-shrink: 0;
  }
  .sk-input .ph { font-size: 12.5px; line-height: 1.5; min-height: 38px; }
  .sk-input .ph .cursor {
    display: inline-block; width: 1.5px; height: 14px;
    background: var(--ink-3);
    vertical-align: middle;
    animation: sk-blink 1s steps(2, jump-none) infinite;
  }
  .sk-input-btns {
    display: flex; align-items: center; justify-content: space-between;
    margin-top: 6px;
  }
  .sk-input-btns .left { display: flex; gap: 4px; }
  .sk-input-btns button {
    height: 22px; padding: 0 8px;
    border-radius: 5px;
    background: transparent;
    color: var(--ink-4);
    border: 0;
    font: inherit; font-size: 11px;
    display: flex; align-items: center; gap: 4px;
    cursor: default;
  }
  .sk-input-btns button:hover { background: oklch(1 0 0 / 0.06); color: var(--ink-3); }
  .skein.theme-light .sk-input-btns button:hover { background: oklch(0 0 0 / 0.05); }
  .sk-input-btns .send {
    background: var(--accent);
    color: oklch(0.20 0.020 60);
    padding: 0 10px;
  }
  .sk-input-btns .send:hover { background: var(--accent); }

  /* sidebar collapsed strip */
  .sk-side.collapsed {
    width: 36px;
    padding: 10px 0;
    align-items: center;
    gap: 8px;
  }
  .sk-side.collapsed .col-icon {
    width: 24px; height: 24px;
    display: flex; align-items: center; justify-content: center;
    color: var(--ink-3);
    border-radius: 5px;
  }
  .sk-side.collapsed .col-icon:hover { background: oklch(1 0 0 / 0.06); color: var(--ink-2); }
  .skein.theme-light .sk-side.collapsed .col-icon:hover { background: oklch(0 0 0 / 0.05); }
  .sk-side.collapsed .col-pill {
    writing-mode: vertical-rl;
    transform: rotate(180deg);
    color: var(--ink-3);
    font-size: 10.5px;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    margin-top: 6px;
  }
`;

if (typeof document !== 'undefined' && !document.getElementById('skein-styles')) {
  const s = document.createElement('style');
  s.id = 'skein-styles';
  s.textContent = SKEIN_CSS;
  document.head.appendChild(s);
}

// ─── SVG icons (small, monoline) ─────────────────────────────────────────
const Icon = {
  gear: (p) => <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.4" {...p}><circle cx="8" cy="8" r="2.2"/><path d="M8 1.5v2M8 12.5v2M14.5 8h-2M3.5 8h-2M12.6 3.4l-1.4 1.4M4.8 11.2l-1.4 1.4M12.6 12.6l-1.4-1.4M4.8 4.8L3.4 3.4"/></svg>,
  chev: (p) => <svg width="9" height="9" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.6" {...p}><path d="M3 5l3 3 3-3"/></svg>,
  pin: (p) => <svg width="9" height="9" viewBox="0 0 12 12" fill="currentColor" {...p}><path d="M6 1L7.5 4l3 .5L8 7l.5 3L6 8.7 3.5 10 4 7 1.5 4.5l3-.5z"/></svg>,
  pinL: (p) => <svg width="10" height="10" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.4" {...p}><path d="M2 2v8M5 4l4-2v8L5 8z" fill="currentColor" fillOpacity="0.4"/></svg>,
  pinR: (p) => <svg width="10" height="10" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.4" {...p}><path d="M10 2v8M7 4l-4-2v8l4-2z" fill="currentColor" fillOpacity="0.4"/></svg>,
  doc: (p) => <svg width="10" height="10" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3" {...p}><path d="M2.5 1.5h5l2 2v7h-7zM7 1.5v2h2"/></svg>,
  paperclip: (p) => <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.4" {...p}><path d="M11 5l-5.5 5.5a2.5 2.5 0 003.5 3.5L14 8.5a4 4 0 10-5.7-5.7L3 8"/></svg>,
  send: (p) => <svg width="11" height="11" viewBox="0 0 14 14" fill="currentColor" {...p}><path d="M2 2l10 5-10 5 2-5z"/></svg>,
  panel: (p) => <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.3" {...p}><rect x="2" y="3" width="12" height="10" rx="1.2"/><path d="M10 3v10" /></svg>,
  search: (p) => <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.4" {...p}><circle cx="7" cy="7" r="4"/><path d="M10 10l3 3"/></svg>,
  cmd: (p) => <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.3" {...p}><path d="M3 2.5h10M3 5.5h10M3 8.5h7M3 11.5h5"/></svg>,
  cursor: (p) => <svg width="20" height="22" viewBox="0 0 20 22" fill="currentColor" {...p}><path d="M2 1.5v17l5-4 3 6.2 2.6-1.2L9.5 13H16z" stroke="black" strokeWidth="0.8" strokeLinejoin="round"/></svg>,
};

// ─── BOOK DATA + spines ──────────────────────────────────────────────────
const BOOKS = [
  { kind: 'folio' },
  { title: 'Research',      h: 78, hue: 28,  shade: 0.30, active: false },
  { title: 'Daily',         h: 84, hue: 145, shade: 0.32, active: true  },
  { title: 'Project Ideas', h: 76, hue: 220, shade: 0.28, active: false },
  { title: 'Reading Notes', h: 82, hue: 350, shade: 0.31, active: false },
  { title: 'Recipes',       h: 74, hue: 60,  shade: 0.34, active: false },
  { title: 'Travel',        h: 80, hue: 195, shade: 0.30, active: false },
  { title: 'Letters',       h: 78, hue: 25,  shade: 0.33, active: false },
  { title: 'Garden',        h: 82, hue: 130, shade: 0.32, active: false },
];

function Spine({ title, h, hue, shade, active, narrow, theme }) {
  // build spine bg from a slightly varied wood tone
  const bg = `oklch(${shade.toFixed(2)} 0.025 ${50 + (hue % 30) - 15})`;
  // band uses the cloth color for the book
  const band = `oklch(${theme === 'light' ? 0.55 : 0.50} 0.085 ${hue})`;
  return (
    <div
      className={`sk-spine ${active ? 'active' : ''}`}
      style={{
        '--spine-h': h + 'px',
        '--spine-w': (narrow ? 22 : 28) + 'px',
        '--spine-bg': bg,
        '--spine-band': band,
      }}
    >
      <div className="band" />
      <div className="band-2" />
      <div className="title below-band">{title}</div>
    </div>
  );
}

function Folio() {
  return (
    <div className="sk-folio" title="Folio — loose pages">
      <div className="sheet" />
      <div className="sheet" />
      <div className="sheet" />
      <div className="twine" />
      <div className="label">Folio</div>
    </div>
  );
}

function Bookshelf({ style, theme }) {
  const row1 = BOOKS.slice(0, 6); // folio + 5 books
  const row2 = BOOKS.slice(6);    // 3 books, then empty slots
  const renderItem = (b, i) => b.kind === 'folio'
    ? <Folio key={'f'+i} />
    : <Spine key={i} {...b} theme={theme} />;
  return (
    <div className={`sk-shelf style-${style}`}>
      <div className="sk-shelf-row">
        {row1.map(renderItem)}
        {/* empty slots are just visible plank — no need for placeholders */}
      </div>
      <div className="sk-shelf-row">
        {row2.map((b, i) => renderItem(b, i + 100))}
      </div>
      <div style={{ height: 12 }} />
    </div>
  );
}

// ─── PAGE CONTENT (mixed) ────────────────────────────────────────────────
function PageDaily({ withCaret, caretAt }) {
  return (
    <div className="sk-page-inner">
      <h1>Tuesday, 21 April</h1>
      <div className="muted">Daily · 7 min read · 4 backlinks</div>

      <h2>Morning</h2>
      <p>Slept poorly, but the Sycamore at the end of the lane was throwing
      enough shade by 7 to read outside. Finished the last chapter of
      <a className="wiki"> [[The Peregrine]]</a> — the section on the cliff
      hunts. Baker writes about watching like it's a craft you have to
      practice into existence.</p>

      <blockquote>"To be recognised and accepted by a peregrine you must
      learn to fit into a landscape and use it to your advantage."</blockquote>

      <h2>Skein, mid-build</h2>
      <p>Spent the afternoon on the bookshelf metaphor — the shelf needs to
      feel like a {withCaret && caretAt === 'shelf' ? <span className="sk-caret insert" /> : null}
      place, not a sidebar. Right now the spines look too uniform. Maybe
      vary the heights more, or let one or two lean.</p>

      <p>Decisions today:</p>
      <ul>
        <li>Keep the chat panel collapsible, not removable.</li>
        <li><span className="md-syntax">**</span>Drag-to-insert<span className="md-syntax">**</span> beats auto-saving the transcript. Fewer surprises.</li>
        <li>Folio at the start of the shelf, not the end.</li>
      </ul>

      <h2>Tomorrow</h2>
      <p>Pick up <a className="wiki">[[Project Ideas]]</a> — the note about
      the marginalia mode. Also: water the rosemary{!withCaret && <span className="sk-caret" />}
      {withCaret && caretAt === 'editor' && <span className="sk-caret insert" />}.</p>

      <p className="muted"><span className="tag">#daily</span> &nbsp; <span className="tag">#skein</span> &nbsp; <span className="tag">#reading</span></p>
    </div>
  );
}

function PageRecipe() {
  return (
    <div className="sk-page-inner">
      <h1>Brown butter, sage & lemon pasta</h1>
      <div className="muted">Recipes · 4 servings · 18 min</div>

      <h3>Have</h3>
      <ul>
        <li>200g pappardelle (or any wide ribbon)</li>
        <li>120g good butter, cubed</li>
        <li>20 sage leaves, dry</li>
        <li>1 lemon — zest + half the juice</li>
        <li>Parmesan, salt, black pepper</li>
      </ul>

      <h3>Do</h3>
      <ol>
        <li>Salted water, rolling boil. Pasta in.</li>
        <li>Cold pan, butter in. Medium heat. When the foam rises and goes
        quiet, drop the sage. It should hiss, then crisp in about 30s.</li>
        <li>Off the heat the second the milk solids turn the color of a
        chestnut. Lemon zest in immediately — it stops the cooking and the
        whole pan smells like a bakery.</li>
        <li>Drain pasta into the pan with a splash of starchy water. Toss
        until glossy. Juice, parm, pepper.</li>
      </ol>

      <blockquote>If the butter goes black, start over. There is no saving it.</blockquote>

      <p className="muted"><span className="tag">#recipes</span> &nbsp; <span className="tag">#weeknight</span></p>
    </div>
  );
}

function PageReview({ withCaret, caretAt }) {
  return (
    <div className="sk-page-inner">
      <h1>On <em>The Peregrine</em></h1>
      <div className="muted">Reading Notes · J.A. Baker, 1967</div>

      <p>A book disguised as a field journal. Ten winters compressed into
      six months on the page. Baker watches falcons in Essex and slowly
      stops being himself.</p>

      <h3>What stays with me</h3>
      <p>The <em>looking</em>. Baker doesn't describe what a peregrine is;
      he describes what the act of seeing one does to a person. The bird is
      almost always at the edge of vision — a notch in the sky, a lift over
      the marsh. {withCaret && caretAt === 'review' && <span className="sk-caret insert" />}
      You are pulled along behind it.</p>

      <h3>Sentences I want to keep</h3>
      <blockquote>"The hardest thing of all to see is what is really there."</blockquote>

      <p>Compare to <a className="wiki">[[H is for Hawk]]</a> — Macdonald
      grieves through the goshawk; Baker disappears into the falcon. Two
      directions out of the same forest.</p>

      <hr />
      <p className="muted">Linked from: <a className="wiki">[[Daily/2026-04-21]]</a></p>
    </div>
  );
}

// ─── EMPTY STATE CARDS ──────────────────────────────────────────────────
const LOOSE_PAGES = [
  { ttl: 'Quick thought — chat as marginalia', body: 'What if the assistant\'s replies behaved like marginalia? They\'d sit in the gutter, you could pull them in, but they\'d default to "in the margin" not "in the page."', meta: ['untitled', '4d'], x: 90, y: 90, r: -3 },
  { ttl: 'Lemon tree — repotting plan', body: 'Roots circling at the bottom of the current pot. Up-pot to ~14" terracotta, gritty mix, late spring after last frost. Don\'t over-water for the first two weeks; let it find the new soil.', meta: ['garden', '6d'], x: 320, y: 60, r: 2.4 },
  { ttl: 'Walk, Sat morning', body: 'Out the door by 7. Take the path along the canal as far as the lock keeper\'s cottage. Coffee from the little place by the bridge. Home by 9 if I want to get any writing done.', meta: ['untitled', '1w'], x: 540, y: 110, r: -1.2 },
  { ttl: 'Reading queue, summer', body: 'In progress: The Peregrine. Next: Pilgrim at Tinker Creek (re-read), The Living Mountain, A Field Guide to Getting Lost. Maybe Berger\'s Pig Earth if I find a copy.', meta: ['reading', '1w'], x: 760, y: 80, r: 1.8 },
  { ttl: 'Q3 invoice — Henley', body: 'Send by the 14th. Itemize the two extra revision rounds separately so they\'re visible. Net 21 this time, not 30 — they paid the last one early anyway.', meta: ['admin', '2w'], x: 980, y: 130, r: -2 },
  { ttl: 'Birthday list — September', body: '— L: a really good notebook. Or the small Leuchtturm in plum.\n— D: bring the mezcal back from the trip.\n— Mom: print the Iceland photos finally.\n— Mike: nothing, he insists.', meta: ['family', '2w'], x: 180, y: 290, r: 1.5 },
  { ttl: 'Skein — naming', body: 'A skein is a length of yarn loosely coiled. Also a flock of wild geese in flight. Both feel right. Threads loosely held. Things that travel together without being tied.', meta: ['untitled', '3w'], x: 410, y: 340, r: -1.8 },
  { ttl: 'Office plant', body: 'The pothos by the window has put out four new leaves since the move. The fiddle-leaf is sulking. Maybe move it closer to the south-facing wall, see if it perks up by June.', meta: ['untitled', '3w'], x: 640, y: 320, r: 2.2 },
  { ttl: 'Half-formed: a small tool for reading', body: 'You highlight a sentence, the tool finds three other things you\'ve highlighted that share its shape — not its keywords, its <em>shape</em>. A sympathetic library. Probably impossible.', meta: ['ideas', '1mo'], x: 870, y: 290, r: -2.6 },
];

function EmptyDesk() {
  return (
    <div className="sk-empty-stage">
      <div className="sk-empty-hint">
        <b>The desk is clear.</b>
        Open a book from the shelf, or pick a loose page below.
      </div>
      {LOOSE_PAGES.map((p, i) => (
        <div
          key={i}
          className="sk-card"
          style={{ left: p.x, top: p.y, transform: `rotate(${p.r}deg)` }}
        >
          <div className="ttl">{p.ttl}</div>
          <div className="body" dangerouslySetInnerHTML={{ __html: p.body }} />
          <div className="meta"><span>{p.meta[0]}</span><span>{p.meta[1]}</span></div>
        </div>
      ))}
    </div>
  );
}

// ─── CHAT MESSAGES ─────────────────────────────────────────────────────
function ChatMessages({ withSelection, streaming }) {
  return (
    <div className="sk-chat">
      <div className="sk-msg user">
        Re-reading <i>The Peregrine</i>. Baker has a sentence I can't shake —
        something about how the hardest thing to see is what's really there.
        Where else does that idea show up in what I've been reading lately?
      </div>

      <div className="sk-msg asst">
        <div className="who"><span className="ico">A</span> Haiku 4.5 · current + related</div>
        <p>Three threads in your vault pull on the same idea, just from
        different angles:</p>
        <p><b>Annie Dillard</b>, in <code>Reading Notes/Tinker Creek.md</code>,
        has the line you bracketed last March: looking is an act of
        attention before it is an act of perception. Same shape as Baker —
        the seeing is the practice, not the prize.</p>
        <p><b>Iain McGilchrist</b>, in your March 6 daily, on the difference
        between <i>knowing</i> a tree and <i>seeing</i> it. You wrote
        "this is the falcon argument" in the margin.</p>
        <p>And maybe a stretch: your note on <code>Ideas/marginalia-mode.md</code>
        — "what is unread is not yet there" — is the same claim turned
        toward writing instead of watching.</p>
      </div>

      <div className="sk-msg user">
        Pull the Dillard sentence so I can drop it next to the Baker one.
      </div>

      <div className="sk-msg asst">
        <div className="who"><span className="ico">A</span> Haiku 4.5 · current + related</div>
        <p>Here it is, exactly as you transcribed it:</p>
        <p>{withSelection
          ? <span className="chat-selection" data-chat-selection>“Seeing is of course very much a matter of verbalization. Unless I call my attention to what passes before my eyes, I simply won't see it.”</span>
          : <span>“Seeing is of course very much a matter of verbalization. Unless I call my attention to what passes before my eyes, I simply won't see it.”</span>
        }</p>
        <p>Source: <code>Reading Notes/Tinker Creek.md</code>, line 47.
        It sits a paragraph above the Baker quote in your{streaming && <span className="streaming-dot" />}
        {!streaming && ' current note.'}</p>
      </div>
    </div>
  );
}

// ─── SIDEBAR ──────────────────────────────────────────────────────────────
function Sidebar({ mode, withSelection, streaming }) {
  if (mode === 'collapsed') {
    return (
      <div className="sk-side collapsed">
        <div className="col-icon"><Icon.panel /></div>
        <div className="col-icon"><svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1l1.4 4.3 4.6.1-3.7 2.7L11.7 13 8 10.3 4.3 13 5.7 8.1 2 5.4l4.6-.1z"/></svg></div>
        <div className="col-pill">Haiku 4.5 · Current + related</div>
      </div>
    );
  }
  if (mode === 'hidden') return null;
  return (
    <div className="sk-side">
      <div className="sk-side-hd">
        <div className="sk-pill"><span className="dot" /> Haiku 4.5 <span className="chev"><Icon.chev /></span></div>
        <div className="sk-pill context">Current + related <span className="chev"><Icon.chev /></span></div>
        <div className="sk-side-spacer" />
        <div className="sk-side-icon" title="History"><Icon.cmd /></div>
        <div className="sk-side-icon" title="Collapse"><Icon.panel /></div>
      </div>
      <ChatMessages withSelection={withSelection} streaming={streaming} />
      <div className="sk-input">
        <div className="ph">Ask Claude about this note<span className="cursor" /></div>
        <div className="sk-input-btns">
          <div className="left">
            <button><Icon.paperclip /> Attach</button>
            <button>@ Mention</button>
          </div>
          <button className="send"><Icon.send /> Send</button>
        </div>
      </div>
    </div>
  );
}

// ─── TABS ─────────────────────────────────────────────────────────────────
function Tabs({ tabs, activeId }) {
  return (
    <div className="sk-tabs">
      {tabs.map(t => (
        <div
          key={t.id}
          className={`sk-tab ${t.id === activeId ? 'active' : ''} ${t.pin || ''}`}
        >
          {t.pin === 'pinned-l' && <span className="pin-ind"><Icon.pinL /></span>}
          {t.pin === 'pinned-r' && <span className="pin-ind"><Icon.pinR /></span>}
          {!t.pin && <span className="pin-ind"><Icon.doc /></span>}
          <span className="name">{t.name}</span>
          {t.dirty && <span className="dirty" />}
          <span className="x">×</span>
        </div>
      ))}
    </div>
  );
}

// ─── DESK ───────────────────────────────────────────────────────────────
function Desk({ scenario, withCaret, caretAt }) {
  if (scenario === 'empty') {
    return (
      <div className="sk-desk">
        <div className="sk-tabs" style={{height: 34, color: 'var(--ink-4)', fontFamily: 'Source Serif 4', fontStyle: 'italic', alignItems: 'center', paddingLeft: 18, fontSize: 12.5}}>
          No tabs open
        </div>
        <EmptyDesk />
      </div>
    );
  }
  // populated split-view
  const tabs = [
    { id: 't1', name: 'Tuesday, 21 April',     pin: 'pinned-l', dirty: true  },
    { id: 't2', name: 'On The Peregrine',      pin: 'pinned-r', dirty: false },
    { id: 't3', name: 'brown-butter-pasta.md', pin: null,       dirty: false },
  ];
  return (
    <div className="sk-desk">
      <Tabs tabs={tabs} activeId="t1" />
      <div className="sk-surface">
        <div className="sk-page">
          <PageDaily withCaret={withCaret} caretAt={caretAt} />
        </div>
        <div className="sk-page">
          <PageReview withCaret={withCaret} caretAt={caretAt} />
        </div>
      </div>
    </div>
  );
}

// ─── TITLEBAR ─────────────────────────────────────────────────────────────
function Titlebar({ vault }) {
  return (
    <div className="sk-titlebar">
      <div className="sk-tb-menus">
        <div className="sk-tb-menu">File</div>
        <div className="sk-tb-menu">Edit</div>
        <div className="sk-tb-menu">View</div>
        <div className="sk-tb-menu">Help</div>
      </div>
      <div className="sk-tb-title">
        Skein <span className="vault">— {vault}</span>
      </div>
      <div className="sk-tb-right">
        <div className="sk-tb-btn" title="Search"><Icon.search /></div>
        <div className="sk-tb-btn" title="Vaults"><Icon.cmd /></div>
        <div className="sk-tb-btn" title="Settings"><Icon.gear /></div>
        <div className="sk-tb-window">
          <button title="Minimize"><svg width="8" height="8" viewBox="0 0 8 8" fill="none" stroke="currentColor" strokeWidth="1.2"><path d="M1.5 4h5"/></svg></button>
          <button title="Maximize"><svg width="8" height="8" viewBox="0 0 8 8" fill="none" stroke="currentColor" strokeWidth="1.2"><rect x="1.5" y="1.5" width="5" height="5"/></svg></button>
          <button title="Close" className="close"><svg width="8" height="8" viewBox="0 0 8 8" fill="none" stroke="currentColor" strokeWidth="1.2"><path d="M2 2l4 4M6 2l-4 4"/></svg></button>
        </div>
      </div>
    </div>
  );
}

// ─── DRAG OVERLAY (for the drag-to-insert mockup) ──────────────────────
function DragOverlay({ x, y, text }) {
  return (
    <>
      <div className="sk-drag-preview" style={{ left: x + 12, top: y + 4 }}>
        “{text}”
      </div>
      <div className="sk-cursor-icon" style={{ left: x - 4, top: y - 4 }}>
        <Icon.cursor />
      </div>
    </>
  );
}

// ─── SkeinWindow — top-level ─────────────────────────────────────────────
function SkeinWindow({
  theme = 'dark',
  shelfStyle = 'suggestive',  // abstract | suggestive | tactile
  sidebar = 'open',           // open | collapsed | hidden
  scenario = 'populated',     // populated | empty | dragging
  vault = 'Field Notes',
  pageFont,
}) {
  const isDragging = scenario === 'dragging';
  return (
    <div className={`skein theme-${theme}`} style={{ width: '100%', height: '100%', '--page-font': pageFont }}>
      <div className="win">
        <Titlebar vault={vault} />
        <div className="sk-body">
          <Bookshelf style={shelfStyle} theme={theme} />
          <div className={`sk-mid sidebar-${sidebar === 'open' ? 'open' : sidebar}`} style={{ position: 'relative' }}>
            <Desk
              scenario={scenario === 'empty' ? 'empty' : 'populated'}
              withCaret={isDragging}
              caretAt={isDragging ? 'editor' : null}
            />
            <Sidebar mode={sidebar} withSelection={isDragging} streaming={false} />
            {isDragging && (
              <DragOverlay
                x={680}
                y={350}
                text="Seeing is of course very much a matter of verbalization. Unless I call my attention to what passes before my eyes, I simply won't see it."
              />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

Object.assign(window, { SkeinWindow });
