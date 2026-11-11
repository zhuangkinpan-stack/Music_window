import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { createInitialAppState } from "../../store/appStore";
import { MainWindow } from "./MainWindow";

describe("MainWindow", () => {
  it("shows the media-service diagnostic instead of fabricated playback", () => {
    render(
      <MainWindow
        state={{
          ...createInitialAppState(),
          diagnostic: {
            code: "smtc_unavailable",
            message: "Windows 系统媒体控制不可用",
            recoverable: true,
          },
        }}
        onSelectSession={() => undefined}
        onPlayPause={() => undefined}
        onNext={() => undefined}
        onPrevious={() => undefined}
        onOpenSettings={() => undefined}
      />,
    );

    expect(screen.getByText("Windows 系统媒体控制不可用")).toBeVisible();
    expect(screen.queryByText("晴天")).not.toBeInTheDocument();
  });
});

