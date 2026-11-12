export interface ShortcutKeyEvent {
  key: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
}

export function formatShortcut(event: ShortcutKeyEvent): string | null {
  if (["Control", "Alt", "Shift", "Meta", "Escape", "Tab"].includes(event.key)) {
    return null;
  }
  if (!event.ctrlKey && !event.altKey && !event.metaKey) {
    return null;
  }

  const parts: string[] = [];
  if (event.ctrlKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  if (event.metaKey) parts.push("Meta");
  parts.push(normalizeKey(event.key));
  return parts.join("+");
}

function normalizeKey(key: string): string {
  const namedKeys: Record<string, string> = {
    " ": "Space",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
  };
  return namedKeys[key] ?? (key.length === 1 ? key.toUpperCase() : key);
}
