import { Pause, Play, SkipBack, SkipForward } from "lucide-react";

import type { SessionCapabilities } from "../../contracts";

interface PlaybackControlsProps {
  playing: boolean;
  capabilities: SessionCapabilities;
  onPlayPause: () => void;
  onNext: () => void;
  onPrevious: () => void;
}

export function PlaybackControls({
  playing,
  capabilities,
  onPlayPause,
  onNext,
  onPrevious,
}: PlaybackControlsProps) {
  return (
    <div className="playback-controls" aria-label="播放控制">
      <button
        type="button"
        className="icon-button"
        aria-label="上一首"
        disabled={!capabilities.previous}
        onClick={onPrevious}
      >
        <SkipBack size={19} fill="currentColor" aria-hidden="true" />
      </button>
      <button
        type="button"
        className="icon-button icon-button--primary"
        aria-label={playing ? "暂停" : "播放"}
        disabled={!capabilities.playPause}
        onClick={onPlayPause}
      >
        {playing ? (
          <Pause size={21} fill="currentColor" aria-hidden="true" />
        ) : (
          <Play size={21} fill="currentColor" aria-hidden="true" />
        )}
      </button>
      <button
        type="button"
        className="icon-button"
        aria-label="下一首"
        disabled={!capabilities.next}
        onClick={onNext}
      >
        <SkipForward size={19} fill="currentColor" aria-hidden="true" />
      </button>
    </div>
  );
}

