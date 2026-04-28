// Lightweight global toggle for the command-palette modal.

export const searchUi: { open: boolean } = $state({ open: false });

export function openSearch() {
  searchUi.open = true;
}
export function closeSearch() {
  searchUi.open = false;
}
export function toggleSearch() {
  searchUi.open = !searchUi.open;
}
