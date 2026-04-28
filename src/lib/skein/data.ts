// Mock data for the Phase 1 UI shell. Replaced in Phase 2 by real
// vault contents loaded from disk.

export type ShelfBook =
  | { kind: "folio" }
  | {
      kind: "book";
      title: string;
      h: number;
      hue: number;
      shade: number;
      active: boolean;
    };

export const BOOKS: ShelfBook[] = [
  { kind: "folio" },
  { kind: "book", title: "Research", h: 78, hue: 28, shade: 0.3, active: false },
  { kind: "book", title: "Daily", h: 84, hue: 145, shade: 0.32, active: true },
  { kind: "book", title: "Project Ideas", h: 76, hue: 220, shade: 0.28, active: false },
  { kind: "book", title: "Reading Notes", h: 82, hue: 350, shade: 0.31, active: false },
  { kind: "book", title: "Recipes", h: 74, hue: 60, shade: 0.34, active: false },
  { kind: "book", title: "Travel", h: 80, hue: 195, shade: 0.3, active: false },
  { kind: "book", title: "Letters", h: 78, hue: 25, shade: 0.33, active: false },
  { kind: "book", title: "Garden", h: 82, hue: 130, shade: 0.32, active: false },
];

export interface LoosePage {
  ttl: string;
  body: string;
  meta: [string, string];
  x: number;
  y: number;
  r: number;
}

export const LOOSE_PAGES: LoosePage[] = [
  {
    ttl: "Quick thought — chat as marginalia",
    body: 'What if the assistant\'s replies behaved like marginalia? They\'d sit in the gutter, you could pull them in, but they\'d default to "in the margin" not "in the page."',
    meta: ["untitled", "4d"],
    x: 90,
    y: 90,
    r: -3,
  },
  {
    ttl: "Lemon tree — repotting plan",
    body: "Roots circling at the bottom of the current pot. Up-pot to ~14\" terracotta, gritty mix, late spring after last frost. Don't over-water for the first two weeks; let it find the new soil.",
    meta: ["garden", "6d"],
    x: 320,
    y: 60,
    r: 2.4,
  },
  {
    ttl: "Walk, Sat morning",
    body: "Out the door by 7. Take the path along the canal as far as the lock keeper's cottage. Coffee from the little place by the bridge. Home by 9 if I want to get any writing done.",
    meta: ["untitled", "1w"],
    x: 540,
    y: 110,
    r: -1.2,
  },
  {
    ttl: "Reading queue, summer",
    body: "In progress: The Peregrine. Next: Pilgrim at Tinker Creek (re-read), The Living Mountain, A Field Guide to Getting Lost. Maybe Berger's Pig Earth if I find a copy.",
    meta: ["reading", "1w"],
    x: 760,
    y: 80,
    r: 1.8,
  },
  {
    ttl: "Q3 invoice — Henley",
    body: "Send by the 14th. Itemize the two extra revision rounds separately so they're visible. Net 21 this time, not 30 — they paid the last one early anyway.",
    meta: ["admin", "2w"],
    x: 980,
    y: 130,
    r: -2,
  },
  {
    ttl: "Birthday list — September",
    body: "— L: a really good notebook. Or the small Leuchtturm in plum.\n— D: bring the mezcal back from the trip.\n— Mom: print the Iceland photos finally.\n— Mike: nothing, he insists.",
    meta: ["family", "2w"],
    x: 180,
    y: 290,
    r: 1.5,
  },
  {
    ttl: "Skein — naming",
    body: "A skein is a length of yarn loosely coiled. Also a flock of wild geese in flight. Both feel right. Threads loosely held. Things that travel together without being tied.",
    meta: ["untitled", "3w"],
    x: 410,
    y: 340,
    r: -1.8,
  },
  {
    ttl: "Office plant",
    body: "The pothos by the window has put out four new leaves since the move. The fiddle-leaf is sulking. Maybe move it closer to the south-facing wall, see if it perks up by June.",
    meta: ["untitled", "3w"],
    x: 640,
    y: 320,
    r: 2.2,
  },
  {
    ttl: "Half-formed: a small tool for reading",
    body: "You highlight a sentence, the tool finds three other things you've highlighted that share its shape — not its keywords, its <em>shape</em>. A sympathetic library. Probably impossible.",
    meta: ["ideas", "1mo"],
    x: 870,
    y: 290,
    r: -2.6,
  },
];
