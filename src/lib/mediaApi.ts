import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type {
  AppSettings,
  LyricsState,
  MediaSessionSummary,
  MediaSnapshot,
  PlaybackState,
} from "../contracts";

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function artworkSource(url: string | null): string | null {
  if (!url) {
    return null;
  }
  if (/^(data:|https?:|asset:)/i.test(url)) {
    return url;
  }
  return convertFileSrc(url);
}

export async function listSessions(): Promise<MediaSnapshot> {
  return invoke<MediaSnapshot>("list_sessions");
}

export function listenMediaSnapshot(
  handler: (snapshot: MediaSnapshot) => void,
): Promise<() => void> {
  return listen<MediaSnapshot>("media://snapshot", (event) =>
    handler(event.payload),
  );
}

export async function selectSession(
  sessionId: string,
): Promise<MediaSnapshot> {
  return invoke<MediaSnapshot>("select_session", { sessionId });
}

export async function togglePlayPause(
  sessionId: string,
): Promise<MediaSnapshot> {
  return invoke<MediaSnapshot>("toggle_play_pause", { sessionId });
}

export async function nextTrack(sessionId: string): Promise<MediaSnapshot> {
  return invoke<MediaSnapshot>("next_track", { sessionId });
}

export async function previousTrack(
  sessionId: string,
): Promise<MediaSnapshot> {
  return invoke<MediaSnapshot>("previous_track", { sessionId });
}

export async function seek(
  sessionId: string,
  positionMs: number,
): Promise<MediaSnapshot> {
  return invoke<MediaSnapshot>("seek", { sessionId, positionMs });
}

export async function loadLyrics(
  session: MediaSessionSummary,
  playback: PlaybackState,
  manualLrc?: string,
): Promise<LyricsState> {
  return invoke<LyricsState>("load_lyrics", {
    title: session.title,
    artist: session.artist,
    album: session.album,
    durationMs: playback.durationMs,
    manualLrc: manualLrc ?? null,
  });
}

export async function minimizeMainWindow(): Promise<void> {
  await getCurrentWindow().minimize();
}

export async function hideMainWindow(): Promise<void> {
  await getCurrentWindow().hide();
}

export async function getQuickBarLockState(): Promise<boolean> {
  return invoke<boolean>("quick_bar_lock_state");
}

export async function setQuickBarLocked(locked: boolean): Promise<boolean> {
  return invoke<boolean>("set_quick_bar_locked", { locked });
}

export async function touchQuickBar(): Promise<void> {
  await invoke("touch_quick_bar");
}

export async function toggleQuickBar(): Promise<boolean> {
  return invoke<boolean>("toggle_quick_bar");
}

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function updateSettings(
  settings: AppSettings,
): Promise<AppSettings> {
  return invoke<AppSettings>("update_settings", { settings });
}

export async function saveQuickBarPosition(
  x: number,
  y: number,
  monitor: string | null,
): Promise<void> {
  await invoke("save_quick_bar_position", { x, y, monitor });
}

export async function setQuickBarVisible(visible: boolean): Promise<void> {
  await invoke("set_quick_bar_visible", { visible });
}

export function listenQuickBarLockState(
  handler: (locked: boolean) => void,
): Promise<() => void> {
  return listen<boolean>("quickbar://lock-state", (event) =>
    handler(event.payload),
  );
}

export function listenQuickBarHover(
  handler: (hovered: boolean) => void,
): Promise<() => void> {
  return listen<boolean>("quickbar://hover-state", (event) =>
    handler(event.payload),
  );
}

export function listenSettingsChanged(
  handler: (settings: AppSettings) => void,
): Promise<() => void> {
  return listen<AppSettings>("settings://changed", (event) =>
    handler(event.payload),
  );
}


