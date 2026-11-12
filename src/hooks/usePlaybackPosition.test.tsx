import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { PlaybackState } from "../contracts";
import { usePlaybackPosition } from "./usePlaybackPosition";

const playing: PlaybackState = {
  sessionId: "netease:cloudmusic",
  playbackStatus: "playing",
  positionMs: 0,
  durationMs: 0,
  timelineEstimated: true,
};

const paused: PlaybackState = {
  ...playing,
  playbackStatus: "paused",
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
      ({ trackKey }) => usePlaybackPosition(playing, trackKey, 200_000),
      { initialProps: { trackKey: "song-a" } },
    );

    act(() => vi.advanceTimersByTime(3_000));
    expect(result.current).toBe(3_000);

    rerender({ trackKey: "song-b" });
    expect(result.current).toBe(0);
  });

  it("caps estimated playback at the effective duration", () => {
    const { result } = renderHook(() =>
      usePlaybackPosition(playing, "song-a", 5_000),
    );

    act(() => vi.advanceTimersByTime(7_000));
    expect(result.current).toBe(5_000);
  });

  it("keeps elapsed time when the duration arrives later", () => {
    const { result, rerender } = renderHook(
      ({ duration }) => usePlaybackPosition(playing, "song-a", duration),
      { initialProps: { duration: 0 } },
    );

    act(() => vi.advanceTimersByTime(3_000));
    rerender({ duration: 200_000 });

    expect(result.current).toBe(3_000);
  });

  it("keeps the current position when playback pauses with a zero sample", () => {
    const { result, rerender } = renderHook(
      ({ playback }) => usePlaybackPosition(playback, "song-a", 200_000),
      { initialProps: { playback: playing } },
    );

    act(() => vi.advanceTimersByTime(42_000));
    expect(result.current).toBe(42_000);

    rerender({ playback: paused });
    expect(result.current).toBe(42_000);

    rerender({ playback: paused });
    expect(result.current).toBe(42_000);
  });

  it("freezes at the live position instead of a lagging sample", () => {
    const { result, rerender } = renderHook(
      ({ playback }) => usePlaybackPosition(playback, "song-a", 200_000),
      { initialProps: { playback: playing } },
    );

    act(() => vi.advanceTimersByTime(30_000));
    // 采样点比本地推进滞后 1.2s，暂停时不能回退
    rerender({ playback: { ...paused, positionMs: 28_800 } });

    expect(result.current).toBe(30_000);
  });

  it("resumes from the frozen position without counting paused time", () => {
    const { result, rerender } = renderHook(
      ({ playback }) => usePlaybackPosition(playback, "song-a", 200_000),
      { initialProps: { playback: playing } },
    );

    act(() => vi.advanceTimersByTime(10_000));
    rerender({ playback: { ...paused, positionMs: 10_000 } });
    act(() => vi.advanceTimersByTime(60_000));
    expect(result.current).toBe(10_000);

    rerender({ playback: { ...playing, positionMs: 10_000 } });
    act(() => vi.advanceTimersByTime(2_000));
    expect(result.current).toBe(12_000);
  });

  it("accepts an externally confirmed rewind", () => {
    const { result, rerender } = renderHook(
      ({ playback }) => usePlaybackPosition(playback, "song-a", 200_000),
      { initialProps: { playback: playing } },
    );

    act(() => vi.advanceTimersByTime(60_000));
    // 用户在音乐软件里把进度拖回 30s：第一次采样先保持，第二次一致才采纳
    rerender({ playback: { ...playing, positionMs: 30_000 } });
    expect(result.current).toBe(60_000);

    act(() => vi.advanceTimersByTime(500));
    rerender({ playback: { ...playing, positionMs: 30_500 } });
    expect(result.current).toBe(30_500);
  });
});
