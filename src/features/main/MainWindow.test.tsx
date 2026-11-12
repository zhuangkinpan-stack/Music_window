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

  it("uses lyrics duration for progress when Netease timeline has no duration", () => {
    render(
      <MainWindow
        state={{
          sessions: [
            {
              id: "netease:cloudmusic",
              appName: "网易云音乐",
              title: "晴天",
              artist: "周杰伦",
              album: "",
              artworkUrl: null,
              playbackStatus: "playing",
              capabilities: {
                playPause: true,
                next: true,
                previous: true,
                seek: false,
              },
            },
          ],
          selectedSessionId: "netease:cloudmusic",
          playback: {
            sessionId: "netease:cloudmusic",
            playbackStatus: "playing",
            positionMs: 120_000,
            durationMs: 0,
            timelineEstimated: true,
          },
          diagnostic: null,
          lyrics: {
            status: "ready",
            source: "automatic",
            lines: [{ timeMs: 1_000, text: "故事的小黄花" }],
            durationMs: 269_000,
            error: null,
          },
        }}
        onSelectSession={() => undefined}
        onPlayPause={() => undefined}
        onNext={() => undefined}
        onPrevious={() => undefined}
        onOpenSettings={() => undefined}
      />,
    );

    expect(screen.getByRole("progressbar", { name: "播放进度" })).toHaveAttribute(
      "aria-valuemax",
      "269000",
    );
    expect(screen.getByText("4:29")).toBeVisible();
  });
});

