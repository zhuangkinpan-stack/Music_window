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
  isTauriRuntime,
  listSessions,
  loadLyrics,
  minimizeMainWindow,
  nextTrack,
  previousTrack,
  selectSession,
  togglePlayPause,
} from "./lib/mediaApi";
import { useAppStore } from "./store/appStore";

const loadingLyrics: LyricsState = {
  status: "loading",
  source: "none",
  lines: [],
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
  const fileInputRef = useRef<HTMLInputElement>(null);
  const positionMs = usePlaybackPosition(playback);
  const selectedSession = sessions.find(
    (session) => session.id === selectedSessionId,
  );

  useEffect(() => {
    document.documentElement.dataset.theme = settings.theme;
  }, [settings.theme]);

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
    const interval = window.setInterval(refresh, 1_500);
    return () => {
      active = false;
      window.clearInterval(interval);
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
        onOpenChange={setSettingsOpen}
        onChange={setSettings}
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

