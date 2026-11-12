import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import type { MediaSessionSummary } from "../../contracts";
import { SessionSwitcher } from "./SessionSwitcher";

const sessions: MediaSessionSummary[] = [
  {
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
  },
  {
    id: "spotify.exe",
    appName: "Spotify",
    title: "Nightcall",
    artist: "Kavinsky",
    album: "OutRun",
    artworkUrl: null,
    playbackStatus: "paused",
    capabilities: {
      playPause: true,
      next: true,
      previous: true,
      seek: true,
    },
  },
];

describe("SessionSwitcher", () => {
  it("selects another available media session", async () => {
    const onSelect = vi.fn();
    const user = userEvent.setup();
    render(
      <SessionSwitcher
        sessions={sessions}
        selectedSessionId="cloudmusic.exe"
        onSelect={onSelect}
      />,
    );

    await user.click(screen.getByRole("button", { name: /Spotify/ }));

    expect(onSelect).toHaveBeenCalledWith("spotify.exe");
  });

  it("marks the selected session as pressed", () => {
    render(
      <SessionSwitcher
        sessions={sessions}
        selectedSessionId="spotify.exe"
        onSelect={() => undefined}
      />,
    );

    expect(screen.getByRole("button", { name: /Spotify/ })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
  });
});

