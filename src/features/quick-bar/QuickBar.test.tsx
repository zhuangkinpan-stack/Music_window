import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import type {
  LyricsState,
  MediaSessionSummary,
  PlaybackState,
} from "../../contracts";
import { QuickBar } from "./QuickBar";

const session: MediaSessionSummary = {
  id: "cloudmusic.exe",
  appName: "网易云音乐",
  title: "晴天",
  artist: "周杰伦",
  album: "叶惠美",
  artworkUrl: null,
  playbackStatus: "playing",
  capabilities: {
    playPause: true,
    next: true,
    previous: true,
    seek: true,
  },
};

const playback: PlaybackState = {
  sessionId: session.id,
  playbackStatus: "playing",
  positionMs: 4_500,
  durationMs: 269_000,
};

const lyrics: LyricsState = {
  status: "ready",
  source: "automatic",
  error: null,
  lines: [
    { timeMs: 1_000, text: "故事的小黄花" },
    { timeMs: 4_000, text: "从出生那年就飘着" },
  ],
};

describe("QuickBar", () => {
  it("shows the current title and synchronized lyric line", () => {
    render(
      <QuickBar
        session={session}
        playback={playback}
        lyrics={lyrics}
        positionMs={4_500}
        locked
        hovered={false}
        onPlayPause={() => undefined}
        onNext={() => undefined}
        onPrevious={() => undefined}
      />,
    );

    expect(screen.getByText("晴天")).toBeVisible();
    expect(screen.getByText("从出生那年就飘着")).toBeVisible();
    expect(screen.getByLabelText("交互已锁定")).toBeInTheDocument();
  });

  it("disables unsupported previous and next commands", () => {
    render(
      <QuickBar
        session={{
          ...session,
          capabilities: {
            playPause: true,
            next: false,
            previous: false,
            seek: false,
          },
        }}
        playback={playback}
        lyrics={lyrics}
        positionMs={0}
        locked={false}
        hovered
        onPlayPause={() => undefined}
        onNext={() => undefined}
        onPrevious={() => undefined}
      />,
    );

    expect(screen.getByRole("button", { name: "上一首" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "下一首" })).toBeDisabled();
  });
});

