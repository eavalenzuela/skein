// Typed wrappers around the Rust vault commands. Phase 2.

import { invoke } from "@tauri-apps/api/core";

export interface Vault {
  root: string;
  name: string;
}

export interface Book {
  name: string;
  rel_path: string;
  page_count: number;
}

export interface Page {
  rel_path: string;
  title: string;
  book: string | null;
  tags: string[];
  modified: number;
}

export const currentVault = () => invoke<Vault | null>("current_vault");
export const openVault = (path: string) => invoke<Vault>("open_vault", { path });
export const closeVault = () => invoke<void>("close_vault");
export const listBooks = () => invoke<Book[]>("list_books");
export const listLoosePages = () => invoke<Page[]>("list_loose_pages");
export const listPagesInBook = (book: string) => invoke<Page[]>("list_pages_in_book", { book });
export const readPage = (relPath: string) => invoke<string>("read_page", { relPath });
