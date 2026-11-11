import { create } from "zustand";

import type {
  AppState,
  LyricsState,
  MediaSessionSummary,
  MediaSnapshot,
  PlaybackState,
} from "../contracts";

const idleLyrics: LyricsState = {
  status: "idle",
  source: "none",
  lines: [],
  error: null,
};

export function createInitialAppState(): AppState {
  return {
    sessions: [],
    selectedSessionId: null,
    playback: null,
    diagnostic: null,
    lyrics: idleLyrics,
  };
}

function chooseSession(
  state: AppState,
  sessions: MediaSessionSummary[],
  currentSessionId: string | null,
): string | null {
  if (
    state.selectedSessionId &&
    sessions.some((session) => session.id === state.selectedSessionId)
  ) {
    return state.selectedSessionId;
  }

  if (
    currentSessionId &&
    sessions.some((session) => session.id === currentSessionId)
  ) {
    return currentSessionId;
  }

  return (
    sessions.find((session) => session.playbackStatus === "playing")?.id ??
    sessions[0]?.id ??
    null
  );
}

function validPlayback(
  playback: PlaybackState | null,
  sessionId: string | null,
): PlaybackState | null {
  return playback?.sessionId === sessionId ? playback : null;
}

export function applyMediaSnapshot(
  state: AppState,
  snapshot: MediaSnapshot,
): AppState {
  const selectedSessionId = chooseSession(
    state,
    snapshot.sessions,
    snapshot.currentSessionId,
  );

  return {
    ...state,
    sessions: snapshot.sessions,
    selectedSessionId,
    playback: validPlayback(snapshot.playback, selectedSessionId),
    diagnostic: snapshot.diagnostic,
  };
}

interface AppStore extends AppState {
  replaceSnapshot: (snapshot: MediaSnapshot) => void;
  selectSession: (sessionId: string) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  ...createInitialAppState(),
  replaceSnapshot: (snapshot) =>
    set((state) => applyMediaSnapshot(state, snapshot)),
  selectSession: (sessionId) =>
    set((state) => ({
      selectedSessionId: sessionId,
      playback:
        state.playback?.sessionId === sessionId ? state.playback : null,
    })),
}));

