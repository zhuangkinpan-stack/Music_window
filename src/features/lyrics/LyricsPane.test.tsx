import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import type { LyricsState } from "../../contracts";
import { LyricsPane } from "./LyricsPane";

const ready: LyricsState = {
  status: "ready",
  source: "automatic",
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

  it("explains that no synchronized lyrics were found", () => {
    render(
      <LyricsPane
        lyrics={{ status: "notFound", source: "none", lines: [], error: null }}
        positionMs={0}
      />,
    );

    expect(screen.getByText("暂无同步歌词")).toBeVisible();
  });
});

