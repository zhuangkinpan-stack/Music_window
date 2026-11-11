import {
  LockKeyhole,
  LockKeyholeOpen,
  Music2,
  Pause,
  Play,
  SkipBack,
  SkipForward,
} from "lucide-react";

import type {
  LyricsState,
  MediaSessionSummary,
  PlaybackState,
} from "../../contracts";

interface QuickBarProps {
  session: MediaSessionSummary;
  playback: PlaybackState;
  lyrics: LyricsState;
  positionMs: number;
  locked: boolean;
  hovered: boolean;
  onPlayPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
  onInteract?: () => void;
}

export function QuickBar({
  session,
  playback,
  lyrics,
  positionMs,
  locked,
  hovered,
  onPlayPause,
  onNext,
  onPrevious,
  onInteract,
}: QuickBarProps) {
  const lyric = currentLyric(lyrics, positionMs);
  const playing = playback.playbackStatus === "playing";

  return (
    <section
      className="quick-bar"
      data-locked={locked}
      data-hovered={hovered}
      aria-label="流光便捷条"
      onPointerDown={onInteract}
      onPointerMove={onInteract}
    >
      <div className="quick-art" data-tauri-drag-region>
        {session.artworkUrl ? (
          <img src={session.artworkUrl} alt="" />
        ) : (
          <Music2 size={17} aria-hidden="true" />
        )}
      </div>

      <div className="quick-copy" data-tauri-drag-region>
        <p data-tauri-drag-region>{session.title || "未知歌曲"}</p>
        <span data-tauri-drag-region>
          {lyric || session.artist || "等待同步歌词"}
        </span>
      </div>

      <div className="quick-controls">
        <button
          type="button"
          aria-label="上一首"
          disabled={!session.capabilities.previous}
          onClick={(event) => {
            event.stopPropagation();
            onPrevious();
          }}
        >
          <SkipBack size={14} fill="currentColor" aria-hidden="true" />
        </button>
        <button
          type="button"
          aria-label={playing ? "暂停" : "播放"}
          disabled={!session.capabilities.playPause}
          onClick={(event) => {
            event.stopPropagation();
            onPlayPause();
          }}
        >
          {playing ? (
            <Pause size={15} fill="currentColor" aria-hidden="true" />
          ) : (
            <Play size={15} fill="currentColor" aria-hidden="true" />
          )}
        </button>
        <button
          type="button"
          aria-label="下一首"
          disabled={!session.capabilities.next}
          onClick={(event) => {
            event.stopPropagation();
            onNext();
          }}
        >
          <SkipForward size={14} fill="currentColor" aria-hidden="true" />
        </button>
      </div>

      <span
        className="quick-lock"
        aria-label={locked ? "交互已锁定" : "交互已解锁"}
        title={locked ? "鼠标穿透中" : "点击控制不会抢走游戏焦点"}
      >
        {locked ? (
          <LockKeyhole size={11} aria-hidden="true" />
        ) : (
          <LockKeyholeOpen size={11} aria-hidden="true" />
        )}
      </span>
    </section>
  );
}

function currentLyric(lyrics: LyricsState, positionMs: number): string | null {
  for (let index = lyrics.lines.length - 1; index >= 0; index -= 1) {
    if (lyrics.lines[index].timeMs <= positionMs) {
      return lyrics.lines[index].text || null;
    }
  }
  return null;
}

