import { useEffect, useRef, useState } from "react";

import type { PlaybackState } from "../contracts";

export function usePlaybackPosition(playback: PlaybackState | null): number {
  const [positionMs, setPositionMs] = useState(playback?.positionMs ?? 0);
  const anchorRef = useRef({ positionMs: 0, updatedAt: Date.now() });

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
      const nextPosition = Math.min(
        playback.durationMs || Number.MAX_SAFE_INTEGER,
        basePosition + elapsed,
      );
      setPositionMs(nextPosition);
    }, 250);
    return () => window.clearInterval(interval);
  }, [
    playback?.durationMs,
    playback?.playbackStatus,
    playback?.positionMs,
    playback?.sessionId,
  ]);

  return positionMs;
}

