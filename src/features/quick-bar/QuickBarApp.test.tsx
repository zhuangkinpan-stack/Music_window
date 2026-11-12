import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  defaultAppSettings,
  type MediaSessionSummary,
  type MediaSnapshot,
  type PlaybackState,
} from "../../contracts";
import { useAppStore } from "../../store/appStore";

const mocks = vi.hoisted(() => ({
  getQuickBarLockState: vi.fn(),
  listenQuickBarLockState: vi.fn(),
  setQuickBarLocked: vi.fn(),
}));

vi.mock("../../lib/mediaApi", () => ({
  artworkSource: vi.fn(() => null),
  getCurrentWindow: vi.fn(),
  getQuickBarLockState: mocks.getQuickBarLockState,
  getSettings: vi.fn().mockResolvedValue(defaultAppSettings),
  isTauriRuntime: () => true,
  listenMediaSnapshot: vi.fn().mockResolvedValue(() => undefined),
  listenQuickBarHover: vi.fn().mockResolvedValue(() => undefined),
  listenQuickBarLockState: mocks.listenQuickBarLockState,
  listenSettingsChanged: vi.fn().mockResolvedValue(() => undefined),
  listSessions: vi.fn(),
  loadLyrics: vi.fn().mockResolvedValue({
    status: "notFound",
    source: "none",
    lines: [],
    error: null,
  }),
  nextTrack: vi.fn(),
  previousTrack: vi.fn(),
  saveQuickBarPosition: vi.fn(),
  setQuickBarLocked: mocks.setQuickBarLocked,
  togglePlayPause: vi.fn(),
  touchQuickBar: vi.fn(),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    onMoved: vi.fn().mockResolvedValue(() => undefined),
  }),
}));

import { QuickBarApp } from "./QuickBarApp";
import { listSessions } from "../../lib/mediaApi";

const session: MediaSessionSummary = {
  id: "netease:cloudmusic",
  appName: "网易云音乐",
  title: "恋人",
  artist: "李荣浩",
  album: "",
  artworkUrl: null,
  playbackStatus: "playing",
  capabilities: {
    playPause: true,
    next: true,
    previous: true,
    seek: false,
  },
};

const playback: PlaybackState = {
  sessionId: session.id,
  playbackStatus: "playing",
  positionMs: 0,
  durationMs: 0,
  timelineEstimated: false,
};

const snapshot: MediaSnapshot = {
  currentSessionId: session.id,
  selectedSessionId: session.id,
  sessions: [session],
  playback,
  diagnostic: null,
};

describe("QuickBarApp lock initialization", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(listSessions).mockResolvedValue(snapshot);
    mocks.setQuickBarLocked.mockResolvedValue(false);
    useAppStore.setState({
      sessions: [session],
      selectedSessionId: session.id,
      playback,
      diagnostic: null,
    });
  });

  it("keeps a lock-state event that arrives before the initial state read", async () => {
    mocks.listenQuickBarLockState.mockImplementation(
      (handler: (locked: boolean) => void) => {
        handler(false);
        return Promise.resolve(() => undefined);
      },
    );
    mocks.getQuickBarLockState.mockResolvedValue(true);

    render(<QuickBarApp />);

    await waitFor(() => {
      expect(screen.getByLabelText("交互已解锁")).toBeInTheDocument();
    });
    expect(mocks.setQuickBarLocked).not.toHaveBeenCalledWith(true);
  });
});
