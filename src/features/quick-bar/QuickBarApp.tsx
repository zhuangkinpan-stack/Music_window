import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type { LyricsState } from "../../contracts";
import { useInteractionLock } from "../../hooks/useInteractionLock";
import { usePlaybackPosition } from "../../hooks/usePlaybackPosition";
import {
  getQuickBarLockState,
  getSettings,
  isTauriRuntime,
  listenQuickBarHover,
  listenQuickBarLockState,
  listenSettingsChanged,
  listenMediaSnapshot,
  listSessions,
  loadLyrics,
  nextTrack,
  previousTrack,
  setQuickBarLocked,
  saveQuickBarPosition,
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
    void getSettings()
      .then((settings) => {
        document.documentElement.dataset.theme = settings.theme;
        document.documentElement.dataset.reduceTransparency = String(
          settings.reduceTransparency,
        );
      })
      .catch(() => undefined);
    let active = true;
    let cleanup: (() => void) | undefined;
    void listenSettingsChanged((settings) => {
      document.documentElement.dataset.theme = settings.theme;
      document.documentElement.dataset.reduceTransparency = String(
        settings.reduceTransparency,
      );
    }).then((unlisten) => {
      if (active) {
        cleanup = unlisten;
      } else {
        unlisten();
      }
    });
    return () => {
      active = false;
      cleanup?.();
    };
  }, []);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }
    let active = true;
    const cleanupPromise = listenMediaSnapshot((snapshot) => {
      if (active) {
        replaceSnapshot(snapshot);
      }
    });
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
    const interval = window.setInterval(refresh, 5_000);
    return () => {
      active = false;
      window.clearInterval(interval);
      void cleanupPromise.then((cleanup) => cleanup());
    };
  }, [replaceSnapshot]);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }
    let active = true;
    let stateEventReceived = false;
    const cleanups: Array<() => void> = [];
    void listenQuickBarLockState((locked) => {
      stateEventReceived = true;
      interaction.setLocked(locked);
    }).then(async (cleanup) => {
      if (active) {
        cleanups.push(cleanup);
        const locked = await getQuickBarLockState();
        if (active && !stateEventReceived) {
          interaction.setLocked(locked);
        }
      } else {
        cleanup();
      }
    });
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
    if (!isTauriRuntime()) {
      return;
    }
    let active = true;
    let saveTimer: number | null = null;
    let cleanup: (() => void) | undefined;
    void getCurrentWindow()
      .onMoved(({ payload }) => {
        if (saveTimer !== null) {
          window.clearTimeout(saveTimer);
        }
        saveTimer = window.setTimeout(() => {
          void saveQuickBarPosition(
            payload.x,
            payload.y,
            null,
          ).catch(() => undefined);
        }, 300);
      })
      .then((unlisten) => {
        if (active) {
          cleanup = unlisten;
        } else {
          unlisten();
        }
      });
    return () => {
      active = false;
      cleanup?.();
      if (saveTimer !== null) {
        window.clearTimeout(saveTimer);
      }
    };
  }, []);

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

