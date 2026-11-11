export type PlaybackStatus =
  | "closed"
  | "opened"
  | "changing"
  | "stopped"
  | "playing"
  | "paused";

export interface SessionCapabilities {
  playPause: boolean;
  next: boolean;
  previous: boolean;
  seek: boolean;
}

export interface MediaSessionSummary {
  id: string;
  appName: string;
  title: string;
  artist: string;
  album: string;
  artworkUrl: string | null;
  playbackStatus: PlaybackStatus;
  capabilities: SessionCapabilities;
}

export interface PlaybackState {
  sessionId: string;
  playbackStatus: PlaybackStatus;
  positionMs: number;
  durationMs: number;
}

export interface MediaDiagnostic {
  code: string;
  message: string;
  recoverable: boolean;
}

export interface MediaSnapshot {
  currentSessionId: string | null;
  sessions: MediaSessionSummary[];
  playback: PlaybackState | null;
  diagnostic: MediaDiagnostic | null;
}

export interface LyricLine {
  timeMs: number;
  text: string;
}

export type LyricsSource = "automatic" | "manual" | "none";

export interface LyricsState {
  status: "idle" | "loading" | "ready" | "notFound" | "error";
  source: LyricsSource;
  lines: LyricLine[];
  error: string | null;
}

export interface WindowPosition {
  x: number;
  y: number;
  monitor: string | null;
}

export interface AppSettings {
  theme: "dark" | "light" | "system";
  quickBarWidthCm: number;
  quickBarPosition: WindowPosition | null;
  showQuickBarOnLaunch: boolean;
  startWithWindows: boolean;
  reduceTransparency: boolean;
  unlockShortcut: string;
  toggleQuickBarShortcut: string;
}

export const defaultAppSettings: AppSettings = {
  theme: "dark",
  quickBarWidthCm: 6.5,
  quickBarPosition: null,
  showQuickBarOnLaunch: true,
  startWithWindows: false,
  reduceTransparency: false,
  unlockShortcut: "Ctrl+Alt+L",
  toggleQuickBarShortcut: "Ctrl+Alt+M",
};

export interface AppState {
  sessions: MediaSessionSummary[];
  selectedSessionId: string | null;
  playback: PlaybackState | null;
  diagnostic: MediaDiagnostic | null;
  lyrics: LyricsState;
}

