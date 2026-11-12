import {
  Minus,
  Music2,
  Settings,
  Sparkles,
  X,
} from "lucide-react";

import type { AppState, MediaSessionSummary } from "../../contracts";
import { formatTime } from "../../lib/formatTime";
import { artworkSource } from "../../lib/mediaApi";
import { LyricsPane } from "../lyrics/LyricsPane";
import { PlaybackControls } from "../player/PlaybackControls";
import { SessionSwitcher } from "../sessions/SessionSwitcher";

interface MainWindowProps {
  state: AppState;
  positionMs?: number;
  lyricOffsetMs?: number;
  onSelectSession: (sessionId: string) => void;
  onPlayPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onOpenSettings: () => void;
  onRetryLyrics?: () => void;
  onImportLyrics?: () => void;
  onMinimize?: () => void;
  onClose?: () => void;
}

export function MainWindow({
  state,
  positionMs = state.playback?.positionMs ?? 0,
  lyricOffsetMs = 0,
  onSelectSession,
  onPlayPause,
  onNext,
  onPrevious,
  onOpenSettings,
  onRetryLyrics,
  onImportLyrics,
  onMinimize,
  onClose,
}: MainWindowProps) {
  const session = state.sessions.find(
    (candidate) => candidate.id === state.selectedSessionId,
  );

  return (
    <main className="main-window" role="application" aria-label="流光">
      <header className="titlebar" data-tauri-drag-region>
        <div className="brand">
          <span className="brand-mark">
            <Sparkles size={15} aria-hidden="true" />
          </span>
          <span>流光</span>
          <span className="brand-subtitle">LUMI</span>
        </div>
        <div className="window-actions">
          <button
            type="button"
            className="window-button"
            aria-label="设置"
            onClick={onOpenSettings}
          >
            <Settings size={16} aria-hidden="true" />
          </button>
          <button
            type="button"
            className="window-button"
            aria-label="最小化"
            onClick={onMinimize}
          >
            <Minus size={16} aria-hidden="true" />
          </button>
          <button
            type="button"
            className="window-button window-button--close"
            aria-label="关闭到托盘"
            onClick={onClose}
          >
            <X size={16} aria-hidden="true" />
          </button>
        </div>
      </header>

      <div className="main-toolbar">
        <SessionSwitcher
          sessions={state.sessions}
          selectedSessionId={state.selectedSessionId}
          onSelect={onSelectSession}
        />
      </div>

      {session && state.playback ? (
        <NowPlaying
          session={session}
          state={state}
          positionMs={positionMs}
          lyricOffsetMs={lyricOffsetMs}
          onPlayPause={onPlayPause}
          onNext={onNext}
          onPrevious={onPrevious}
          onRetryLyrics={onRetryLyrics}
          onImportLyrics={onImportLyrics}
        />
      ) : (
        <EmptyState state={state} />
      )}
    </main>
  );
}

interface NowPlayingProps {
  session: MediaSessionSummary;
  state: AppState;
  positionMs: number;
  lyricOffsetMs: number;
  onPlayPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onRetryLyrics?: () => void;
  onImportLyrics?: () => void;
}

function NowPlaying({
  session,
  state,
  positionMs,
  lyricOffsetMs,
  onPlayPause,
  onNext,
  onPrevious,
  onRetryLyrics,
  onImportLyrics,
}: NowPlayingProps) {
  const playback = state.playback!;
  const effectiveDuration =
    playback.durationMs > 0 ? playback.durationMs : state.lyrics.durationMs;
  const progress =
    effectiveDuration > 0
      ? Math.min(100, (positionMs / effectiveDuration) * 100)
      : 0;

  return (
    <section className="now-playing">
      <div className="artwork-column">
        <Artwork session={session} />
        <div className="track-meta">
          <p className="section-kicker">{session.appName}</p>
          <h1>{session.title || "未知歌曲"}</h1>
          <p>
            {session.artist || "未知歌手"}
            {session.album ? ` · ${session.album}` : ""}
          </p>
        </div>
      </div>

      <LyricsPane
        lyrics={state.lyrics}
        positionMs={positionMs}
        offsetMs={lyricOffsetMs}
        onRetry={onRetryLyrics}
        onImport={onImportLyrics}
      />

      <footer className="transport">
        <div className="progress-row">
          <span>{formatTime(positionMs)}</span>
          <div
            className="progress-track"
            role="progressbar"
            aria-label="播放进度"
            aria-valuemin={0}
            aria-valuemax={effectiveDuration}
            aria-valuenow={Math.min(positionMs, effectiveDuration)}
          >
            <span style={{ width: `${progress}%` }} />
          </div>
          <span>{formatTime(effectiveDuration)}</span>
        </div>
        <PlaybackControls
          playing={playback.playbackStatus === "playing"}
          capabilities={session.capabilities}
          onPlayPause={onPlayPause}
          onNext={onNext}
          onPrevious={onPrevious}
        />
      </footer>
    </section>
  );
}

function Artwork({ session }: { session: MediaSessionSummary }) {
  const source = artworkSource(session.artworkUrl);
  if (source) {
    return (
      <img
        className="album-art"
        src={source}
        alt={`${session.title} 专辑封面`}
      />
    );
  }
  return (
    <div className="album-art album-art--fallback" aria-label="默认专辑封面">
      <Music2 size={58} aria-hidden="true" />
      <span>{session.title.slice(0, 1) || "♪"}</span>
    </div>
  );
}

function EmptyState({ state }: { state: AppState }) {
  return (
    <section className="empty-state">
      <div className="empty-orbit" aria-hidden="true">
        <Music2 size={28} />
      </div>
      <p className="section-kicker">SYSTEM MEDIA CONTROL</p>
      <h1>
        {state.diagnostic?.message ??
          (state.sessions.length
            ? "选择一个音乐来源"
            : "尚未连接到音乐会话")}
      </h1>
      <p>
        {state.diagnostic
          ? "请确认音乐软件正在播放，并支持 Windows 系统媒体控制。"
          : "在网易云音乐、Spotify、QQ 音乐等客户端中开始播放，流光会自动连接。"}
      </p>
    </section>
  );
}

