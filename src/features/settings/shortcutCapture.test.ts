import { describe, expect, it } from "vitest";

import { formatShortcut } from "./shortcutCapture";

describe("formatShortcut", () => {
  it("formats modifier and key combinations", () => {
    expect(
      formatShortcut({
        key: "q",
        ctrlKey: true,
        altKey: true,
        shiftKey: false,
        metaKey: false,
      }),
    ).toBe("Ctrl+Alt+Q");
  });

  it("ignores modifier-only key presses", () => {
    expect(
      formatShortcut({
        key: "Control",
        ctrlKey: true,
        altKey: false,
        shiftKey: false,
        metaKey: false,
      }),
    ).toBeNull();
  });
});
