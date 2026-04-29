// Focus-trap action for modals. Captures the previously-focused element
// on mount, focuses the first tabbable element inside the trap (or the
// trap container itself), and confines Tab / Shift+Tab to the trap. On
// destroy, restores focus to whatever held it before.

const SELECTOR = [
  "a[href]",
  "button:not([disabled])",
  "textarea:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  '[tabindex]:not([tabindex="-1"])',
].join(",");

function tabbables(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>(SELECTOR)).filter(
    (el) => el.offsetParent !== null,
  );
}

export function focusTrap(node: HTMLElement) {
  const previouslyFocused = document.activeElement as HTMLElement | null;

  // Defer focus so children render before we look up tabbables.
  queueMicrotask(() => {
    const items = tabbables(node);
    (items[0] ?? node).focus();
  });

  function onKey(e: KeyboardEvent) {
    if (e.key !== "Tab") return;
    const items = tabbables(node);
    if (items.length === 0) {
      e.preventDefault();
      node.focus();
      return;
    }
    const first = items[0];
    const last = items[items.length - 1];
    const active = document.activeElement as HTMLElement | null;
    if (e.shiftKey && (active === first || !node.contains(active))) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && (active === last || !node.contains(active))) {
      e.preventDefault();
      first.focus();
    }
  }

  node.addEventListener("keydown", onKey);

  return {
    destroy() {
      node.removeEventListener("keydown", onKey);
      previouslyFocused?.focus?.();
    },
  };
}
