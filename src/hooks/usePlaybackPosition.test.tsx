import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { PlaybackState } from "../contracts";
import { usePlaybackPosition } from "./usePlaybackPosition";

const playback: PlaybackState = {
  sessionId: "netease:cloudmusic",
  playbackStatus: "playing",
  positionMs: 0,
  durationMs: 0,
  timelineEstimated: true,
};

describe("usePlaybackPosition", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-11-12T12:00:00Z"));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("resets when the track key changes", () => {
    const { result, rerender } = renderHook(
      ({ trackKey }) =>
        usePlaybackPosition(playback, trackKey, 200_000),
      { initialProps: { trackKey: "song-a" } },
    );

    act(() => vi.advanceTimersByTime(3_000));
    expect(result.current).toBe(3_000);

    rerender({ trackKey: "song-b" });
    expect(result.current).toBe(0);
  });

  it("caps estimated playback at the effective duration", () => {
    const { result } = renderHook(() =>
      usePlaybackPosition(playback, "song-a", 5_000),
    );

    act(() => vi.advanceTimersByTime(7_000));
    expect(result.current).toBe(5_000);
  });

  it("keeps elapsed time when the duration arrives later", () => {
    const { result, rerender } = renderHook(
      ({ duration }) =>
        usePlaybackPosition(playback, "song-a", duration),
      { initialProps: { duration: 0 } },
    );

    act(() => vi.advanceTimersByTime(3_000));
    rerender({ duration: 200_000 });

    expect(result.current).toBe(3_000);
  });
});
