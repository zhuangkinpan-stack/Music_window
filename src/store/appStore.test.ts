import { describe, expect, it } from "vitest";

import type { AppState, MediaSnapshot } from "../contracts";
import { applyMediaSnapshot, createInitialAppState } from "./appStore";

function playingSnapshot(sessionId: string): MediaSnapshot {
  return {
    currentSessionId: sessionId,
    sessions: [
      {
        id: sessionId,
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
    ],
    playback: {
      sessionId,
      playbackStatus: "playing",
      positionMs: 42_000,
      durationMs: 269_000,
    },
    diagnostic: null,
  };
}

describe("applyMediaSnapshot", () => {
  it("uses the current Windows session on first synchronization", () => {
    const state = createInitialAppState();

    const next = applyMediaSnapshot(state, playingSnapshot("cloudmusic.exe"));

    expect(next.selectedSessionId).toBe("cloudmusic.exe");
    expect(next.playback?.positionMs).toBe(42_000);
  });

  it("clears stale playback when the selected session disappears", () => {
    const playing = applyMediaSnapshot(
      createInitialAppState(),
      playingSnapshot("cloudmusic.exe"),
    );
    const empty = applyMediaSnapshot(playing, {
      currentSessionId: null,
      sessions: [],
      playback: null,
      diagnostic: null,
    });

    expect(empty.selectedSessionId).toBeNull();
    expect(empty.playback).toBeNull();
  });
});

