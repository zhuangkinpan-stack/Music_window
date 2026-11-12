import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { defaultAppSettings } from "../../contracts";
import { SettingsPanel } from "./SettingsPanel";

describe("SettingsPanel", () => {
  it("shows the lyric offset with half-second precision", () => {
    render(
      <SettingsPanel
        open
        settings={{ ...defaultAppSettings, lyricOffsetMs: 750 }}
        onOpenChange={() => undefined}
        onChange={() => undefined}
      />,
    );

    expect(screen.getByLabelText("歌词偏移")).toBeInTheDocument();
    expect(screen.getByText("+0.8 s")).toBeVisible();
  });

  it("captures a new shortcut from the keyboard", () => {
    const onChange = vi.fn();
    render(
      <SettingsPanel
        open
        settings={defaultAppSettings}
        onOpenChange={() => undefined}
        onChange={onChange}
      />,
    );

    fireEvent.keyDown(screen.getByLabelText("便捷条显示快捷键"), {
      key: "q",
      ctrlKey: true,
      altKey: true,
    });

    expect(onChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ toggleQuickBarShortcut: "Ctrl+Alt+Q" }),
    );
  });

  it("shows shortcut registration conflicts inline", () => {
    render(
      <SettingsPanel
        open
        settings={defaultAppSettings}
        shortcutError="Ctrl+Alt+Q 已被其他应用占用"
        onOpenChange={() => undefined}
        onChange={() => undefined}
      />,
    );

    expect(screen.getByText("Ctrl+Alt+Q 已被其他应用占用")).toBeVisible();
  });
});
