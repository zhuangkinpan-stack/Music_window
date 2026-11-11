import { useEffect, useRef, useState } from "react";

import type { LyricsState } from "../../contracts";
import { useInteractionLock } from "../../hooks/useInteractionLock";
import { usePlaybackPosition } from "../../hooks/usePlaybackPosition";
import {
  getQuickBarLockState,
  isTauriRuntime,
  listenQuickBarHover,
  listenQuickBarLockState,
  listSessions,
  loadLyrics,
  nextTrack,
  previousTrack,
  setQuickBarLocked,
  togglePlayPause,
  touchQuickBar,
} from "../../lib/mediaApi";
import { useAppStore } from "../../store/appStore";
import { QuickBar } from "./QuickBar";

const loadingLyrics: LyricsState = {
  status: "loading",
  source: "none",
  lines: [],
  error: null,
};

export function QuickBarApp() {
  const {
    sessions,
    selectedSessionId,
    playback,
    lyrics,
    replaceSnapshot,
    setLyrics,
  } = useAppStore();
  const interaction = useInteractionLock();
  const [hovered, setHovered] = useState(false);
  const lastTouchRef = useRef(0);
  const positionMs = usePlaybackPosition(playback);
  const session = sessions.find((item) => item.id === selectedSessionId);

  useEffect(() => {
    document.body.classList.add("quick-window");
    return () => document.body.classList.remove("quick-window");
  }, []);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }
    let active = true;
    const refresh = async () => {
      try {
        const snapshot = await listSessions();
        if (active) {
          replaceSnapshot(snapshot);
        }
      } catch {
        // The main window owns user-visible sync diagnostics.
      }
    };
    void refresh();
    const interval = window.setInterval(refresh, 1_000);
    return () => {
      active = false;
      window.clearInterval(interval);
    };
  }, [replaceSnapshot]);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }
    let active = true;
    const cleanups: Array<() => void> = [];
    void getQuickBarLockState().then((locked) => {
      if (active) {
        interaction.setLocked(locked);
      }
    });
    void listenQuickBarLockState((locked) => interaction.setLocked(locked)).then(
      (cleanup) => {
        if (active) {
          cleanups.push(cleanup);
        } else {
          cleanup();
        }
      },
    );
    void listenQuickBarHover(setHovered).then((cleanup) => {
      if (active) {
        cleanups.push(cleanup);
      } else {
        cleanup();
      }
    });
    return () => {
      active = false;
      cleanups.forEach((cleanup) => cleanup());
    };
  }, [interaction.setLocked]);

  useEffect(() => {
    if (!session || !playback || !isTauriRuntime()) {
      return;
    }
    let active = true;
    setLyrics(loadingLyrics);
    void loadLyrics(session, playback)
      .then((result) => {
        if (active) {
          setLyrics(result);
        }
      })
      .catch((error) => {
        if (active) {
          setLyrics({
            status: "error",
            source: "none",
            lines: [],
            error: String(error),
          });
        }
      });
    return () => {
      active = false;
    };
  }, [
    playback?.durationMs,
    playback?.sessionId,
    session?.album,
    session?.artist,
    session?.id,
    session?.title,
    setLyrics,
  ]);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }
    void setQuickBarLocked(interaction.locked);
  }, [interaction.locked]);

  const interact = () => {
    if (!interaction.locked) {
      const now = Date.now();
      if (now - lastTouchRef.current < 120) {
        return;
      }
      lastTouchRef.current = now;
      interaction.touch();
      if (isTauriRuntime()) {
        void touchQuickBar();
      }
      return;
    }
    interaction.unlock();
    if (isTauriRuntime()) {
      void setQuickBarLocked(false);
    }
  };

  const run = async (
    command: (sessionId: string) => Promise<Parameters<typeof replaceSnapshot>[0]>,
  ) => {
    if (!session) {
      return;
    }
    try {
      replaceSnapshot(await command(session.id));
    } catch {
      // Keep the previous safe state; the main window exposes diagnostics.
    }
  };

  if (!session || !playback) {
    return <div className="quick-placeholder">等待音乐播放</div>;
  }

  return (
    <QuickBar
      session={session}
      playback={playback}
      lyrics={lyrics}
      positionMs={positionMs}
      locked={interaction.locked}
      hovered={hovered}
      onInteract={interact}
      onPlayPause={() => void run(togglePlayPause)}
      onNext={() => void run(nextTrack)}
      onPrevious={() => void run(previousTrack)}
    />
  );
}

