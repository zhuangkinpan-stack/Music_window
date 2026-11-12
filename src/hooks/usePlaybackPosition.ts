import { useEffect, useRef, useState } from "react";

import type { PlaybackState } from "../contracts";

export function usePlaybackPosition(
  playback: PlaybackState | null,
  trackKey: string,
  durationOverrideMs = 0,
): number {
  const [positionMs, setPositionMs] = useState(playback?.positionMs ?? 0);
  const anchorRef = useRef({ positionMs: 0, updatedAt: Date.now() });
  const durationRef = useRef(0);
  durationRef.current =
    playback && playback.durationMs > 0
      ? playback.durationMs
      : durationOverrideMs;

  useEffect(() => {
    const basePosition = playback?.positionMs ?? 0;
    const updatedAt = Date.now();
    anchorRef.current = { positionMs: basePosition, updatedAt };
    setPositionMs(basePosition);

    if (!playback || playback.playbackStatus !== "playing") {
      return;
    }

    const interval = window.setInterval(() => {
      const elapsed = Date.now() - updatedAt;
      const effectiveDuration = durationRef.current;
      const nextPosition = Math.min(
        effectiveDuration > 0 ? effectiveDuration : Number.MAX_SAFE_INTEGER,
        basePosition + elapsed,
      );
      setPositionMs(nextPosition);
    }, 250);
    return () => window.clearInterval(interval);
  }, [
    playback?.playbackStatus,
    playback?.positionMs,
    playback?.sessionId,
    trackKey,
  ]);

  return positionMs;
}

