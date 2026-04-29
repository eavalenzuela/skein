// Deterministic spine-visual derivation from a book name. Same name → same look,
// independent of order in the vault.

const HUES = [28, 145, 220, 350, 60, 195, 25, 130, 280, 95, 175];

function hash(s: string): number {
  let h = 2166136261;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return Math.abs(h);
}

export function spineHue(name: string): number {
  return HUES[hash(name) % HUES.length];
}

export function spineHeight(name: string): number {
  return 74 + (hash(name + "h") % 11); // 74–84
}

export function spineShade(name: string): number {
  // 0.28–0.34
  return 0.28 + (hash(name + "s") % 7) / 100;
}

/** Spine width grows with title length so long names stay legible.
 * Vertical text in `writing-mode: vertical-rl` wraps into extra columns
 * once we widen, which is the visual we want — comically wide is fine. */
export function spineWidth(name: string): number {
  const base = 28;
  const len = name.length;
  if (len <= 10) return base;
  // ~10px per extra ~3 chars beyond the comfortable cap.
  const extra = Math.ceil((len - 10) / 3) * 10;
  return base + extra;
}
