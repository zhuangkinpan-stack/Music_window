import { useEffect, useRef, useState } from "react";

import {
  defaultAppSettings,
  type AppSettings,
  type LyricsState,
} from "./contracts";
import { MainWindow } from "./features/main/MainWindow";
import { SettingsPanel } from "./features/settings/SettingsPanel";
import { usePlaybackPosition } from "./hooks/usePlaybackPosition";
import {
  hideMainWindow,
  getSettings,
  isTauriRuntime,
  listSessions,
  listenMediaSnapshot,
  loadLyrics,
  minimizeMainWindow,
  nextTrack,
  previousTrack,
  selectSession,
  setQuickBarVisible,
  togglePlayPause,
  updateSettings,
} from "./lib/mediaApi";
import { useAppStore } from "./store/appStore";

const loadingLyrics: LyricsState = {
  status: "loading",
  source: "none",
  lines: [],
  durationMs: 0,
  error: null,
};

export default function App() {
  const {
    sessions,
    selectedSessionId,
    playback,
    diagnostic,
    lyrics,
    replaceSnapshot,
    selectSession: selectSessionLocally,
    setLyrics,
    setDiagnostic,
  } = useAppStore();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [settings, setSettings] = useState<AppSettings>(defaultAppSettings);
  const [settingsHydrated, setSettingsHydrated] = useState(false);
  const [settingsError, setSettingsError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const selectedSession = sessions.find(
    (session) => session.id === selectedSessionId,
  );
  const trackKey = selectedSession
    ? `${selectedSession.id}|${selectedSession.title}|${selectedSession.artist}|${selectedSession.album}`
    : "";
  const positionMs = usePlaybackPosition(
    playback,
    trackKey,
    lyrics.durationMs,
  );

  useEffect(() => {
    document.documentElement.dataset.theme = settings.theme;
    document.documentElement.dataset.reduceTransparency = String(
      settings.reduceTransparency,
    );
  }, [settings.reduceTransparency, settings.theme]);

  useEffect(() => {
    if (!isTauriRuntime()) {
      setSettingsHydrated(true);
      return;
    }
    let active = true;
    void getSettings()
      .then((loaded) => {
        if (active) {
          setSettings(loaded);
          setSettingsError(null);
          setSettingsHydrated(true);
        }
      })
      .catch(() => {
        if (active) {
          setSettingsHydrated(true);
        }
      });
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    if (!settingsHydrated || !isTauriRuntime()) {
      return;
    }
    const timeout = window.setTimeout(() => {
      void updateSettings(settings).catch((error) => {
        const message = String(error);
        setSettingsError(message);
        setDiagnostic({
          code: "settings_failed",
          message: `无法保存设置：${message}`,
          recoverable: true,
        });
        void getSettings().then(setSettings).catch(() => undefined);
      });
    }, 250);
    return () => window.clearTimeout(timeout);
  }, [settings, settingsHydrated, setDiagnostic]);

  const hasSessions = sessions.length > 0;
  useEffect(() => {
    if (!settingsHydrated || !isTauriRuntime()) {
      return;
    }
    void setQuickBarVisible(
      hasSessions && settings.showQuickBarOnLaunch,
    ).catch(() => undefined);
  }, [
    hasSessions,
    settings.showQuickBarOnLaunch,
    settingsHydrated,
  ]);

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
      } catch (error) {
        if (active) {
          setDiagnostic({
            code: "sync_failed",
            message: `无法同步 Windows 媒体会话：${String(error)}`,
            recoverable: true,
          });
        }
      }
    };
    void refresh();
    const interval = window.setInterval(refresh, 5_000);
    return () => {
      active = false;
      window.clearInterval(interval);
      void cleanupPromise.then((cleanup) => cleanup());
    };
  }, [replaceSnapshot, setDiagnostic]);

  const requestLyrics = async (manualLrc?: string) => {
    if (!isTauriRuntime() || !selectedSession || !playback) {
      return;
    }
    setLyrics(loadingLyrics);
    try {
      const result = await loadLyrics(selectedSession, playback, manualLrc);
      setLyrics(result);
    } catch (error) {
      setLyrics({
        status: "error",
        source: "none",
        lines: [],
        durationMs: 0,
        error: String(error),
      });
    }
  };

  useEffect(() => {
    if (!selectedSession || !playback || !isTauriRuntime()) {
      return;
    }
    let active = true;
    setLyrics(loadingLyrics);
    void loadLyrics(selectedSession, playback)
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
            durationMs: 0,
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
    selectedSession?.album,
    selectedSession?.artist,
    selectedSession?.id,
    selectedSession?.title,
    setLyrics,
  ]);

  const runCommand = async (
    command: (
      sessionId: string,
    ) => Promise<Parameters<typeof replaceSnapshot>[0]>,
  ) => {
    if (!selectedSessionId) {
      return;
    }
    try {
      replaceSnapshot(await command(selectedSessionId));
    } catch (error) {
      setDiagnostic({
        code: "command_failed",
        message: `播放控制失败：${String(error)}`,
        recoverable: true,
      });
    }
  };

  const handleSelectSession = (sessionId: string) => {
    selectSessionLocally(sessionId);
    if (isTauriRuntime()) {
      void selectSession(sessionId)
        .then(replaceSnapshot)
        .catch((error) =>
          setDiagnostic({
            code: "select_failed",
            message: `无法切换音乐来源：${String(error)}`,
            recoverable: true,
          }),
        );
    }
  };

  const handleImportLyrics = () => fileInputRef.current?.click();

  return (
    <>
      <MainWindow
        state={{
          sessions,
          selectedSessionId,
          playback,
          diagnostic,
          lyrics,
        }}
        positionMs={positionMs}
        lyricOffsetMs={settings.lyricOffsetMs}
        onSelectSession={handleSelectSession}
        onPlayPause={() => void runCommand(togglePlayPause)}
        onNext={() => void runCommand(nextTrack)}
        onPrevious={() => void runCommand(previousTrack)}
        onOpenSettings={() => setSettingsOpen(true)}
        onRetryLyrics={() => void requestLyrics()}
        onImportLyrics={handleImportLyrics}
        onMinimize={() => void minimizeMainWindow().catch(() => undefined)}
        onClose={() => void hideMainWindow().catch(() => undefined)}
      />
      <SettingsPanel
        open={settingsOpen}
        settings={settings}
        shortcutError={settingsError}
        onOpenChange={setSettingsOpen}
        onChange={(next) => {
          setSettingsError(null);
          setSettings(next);
        }}
      />
      <input
        ref={fileInputRef}
        type="file"
        accept=".lrc,text/plain"
        hidden
        onChange={(event) => {
          const file = event.target.files?.[0];
          if (file) {
            void file.text().then((text) => requestLyrics(text));
          }
          event.currentTarget.value = "";
        }}
      />
    </>
  );
}

