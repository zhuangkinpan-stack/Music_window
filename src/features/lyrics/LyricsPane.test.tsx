import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import type { LyricsState } from "../../contracts";
import { LyricsPane } from "./LyricsPane";

const ready: LyricsState = {
  status: "ready",
  source: "automatic",
  durationMs: 269_000,
  error: null,
  lines: [
    { timeMs: 1_000, text: "故事的小黄花" },
    { timeMs: 4_000, text: "从出生那年就飘着" },
    { timeMs: 8_000, text: "童年的荡秋千" },
  ],
};

describe("LyricsPane", () => {
  it("highlights the lyric matching the playback position", () => {
    render(<LyricsPane lyrics={ready} positionMs={4_500} />);

    expect(screen.getByText("从出生那年就飘着")).toHaveAttribute(
      "aria-current",
      "true",
    );
    expect(screen.getByText("故事的小黄花")).not.toHaveAttribute("aria-current");
  });

  it("applies the configured lyrics offset", () => {
    render(<LyricsPane lyrics={ready} positionMs={4_500} offsetMs={-2_000} />);

    expect(screen.getByText("故事的小黄花")).toHaveAttribute(
      "aria-current",
      "true",
    );
  });

  it("explains that no synchronized lyrics were found", () => {
    render(
      <LyricsPane
        lyrics={{
          status: "notFound",
          source: "none",
          lines: [],
          durationMs: 0,
          error: null,
        }}
        positionMs={0}
      />,
    );

    expect(screen.getByText("暂无同步歌词")).toBeVisible();
  });

  it("shows a recoverable network failure with retry", () => {
    render(
      <LyricsPane
        lyrics={{
          status: "error",
          source: "none",
          lines: [],
          durationMs: 0,
          error: "歌词网络请求失败",
        }}
        positionMs={0}
        onRetry={() => undefined}
      />,
    );

    expect(screen.getByText("歌词加载失败")).toBeVisible();
    expect(screen.getByText("歌词网络请求失败")).toBeVisible();
    expect(screen.getByRole("button", { name: "重新匹配" })).toBeEnabled();
  });
});

