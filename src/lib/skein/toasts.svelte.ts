// App-wide notification bus.
//
// Background work — autosave, index rebuilds, git sync, exports — used to
// fail into console.error, so a save that never landed looked identical to
// one that did. Anything the user would want to know about goes through
// here instead.
//
// Errors stick until dismissed; successes fade. An action button lets a
// toast carry the recovery for what just failed (retry, undo).

export type ToastKind = "info" | "success" | "error";

export interface ToastAction {
  label: string;
  run: () => void | Promise<void>;
}

export interface Toast {
  id: number;
  kind: ToastKind;
  message: string;
  detail?: string;
  action?: ToastAction;
}

const AUTO_DISMISS_MS: Record<ToastKind, number | null> = {
  info: 4000,
  success: 3000,
  // Errors are the whole point of the bus — never time them out.
  error: null,
};

export const toastState: { items: Toast[] } = $state({ items: [] });

let nextId = 1;
const timers = new Map<number, ReturnType<typeof setTimeout>>();

export function dismissToast(id: number) {
  const t = timers.get(id);
  if (t) {
    clearTimeout(t);
    timers.delete(id);
  }
  toastState.items = toastState.items.filter((x) => x.id !== id);
}

export function pushToast(
  kind: ToastKind,
  message: string,
  opts: { detail?: string; action?: ToastAction } = {},
): number {
  const id = nextId++;
  toastState.items = [...toastState.items, { id, kind, message, ...opts }];
  // Cap the stack so a failing loop can't bury the UI.
  if (toastState.items.length > 4) {
    dismissToast(toastState.items[0].id);
  }
  const ms = AUTO_DISMISS_MS[kind];
  if (ms !== null) {
    timers.set(
      id,
      setTimeout(() => dismissToast(id), ms),
    );
  }
  return id;
}

export const toastInfo = (message: string, detail?: string) =>
  pushToast("info", message, { detail });
export const toastSuccess = (message: string, detail?: string, action?: ToastAction) =>
  pushToast("success", message, { detail, action });
export const toastError = (message: string, detail?: string, action?: ToastAction) =>
  pushToast("error", message, { detail, action });

export function clearToasts() {
  for (const t of timers.values()) clearTimeout(t);
  timers.clear();
  toastState.items = [];
}
