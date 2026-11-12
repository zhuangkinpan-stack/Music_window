import { useEffect, useRef, useState } from "react";

import type { PlaybackState } from "../contracts";

/** 采样回退的容忍范围：超出这个幅度才怀疑是外部拖动进度，而不是采样滞后。 */
const BACKWARD_TOLERANCE_MS = 2_000;
const TICK_MS = 250;

function clampToDuration(positionMs: number, durationMs: number): number {
  const floored = Math.max(0, positionMs);
  return durationMs > 0 ? Math.min(floored, durationMs) : floored;
}

/**
 * 播放进度展示。
 *
 * 后端每 500ms 推一次快照，采样点自带滞后。若直接把快照里的 positionMs 套到 UI 上，
 * 暂停时进度条就会退回上一次采样点（网易云兼容层早期甚至直接回退到 0）。
 * 这里维护一条「锚点 + 墙钟推进」的展示时间轴：
 * - 播放中按本地时钟平滑推进，采样只用于校正锚点，且不倒退；
 * - 暂停瞬间冻结在暂停那一刻的显示位置，不跟随滞后采样回退；
 * - 真正的位置倒退（外部拖动进度）需要连续两次采样一致才采纳；
 * - 换歌立即重置到新的采样位置。
 */
export function usePlaybackPosition(
  playback: PlaybackState | null,
  trackKey: string,
  durationOverrideMs = 0,
): number {
  const initial = Math.max(0, playback?.positionMs ?? 0);
  const [positionMs, setPositionMs] = useState(initial);
  const displayedRef = useRef(initial);
  const trackRef = useRef(trackKey);
  const lastRewindRef = useRef<number | null>(null);
  const anchorRef = useRef({ positionMs: initial, updatedAt: Date.now() });
  const durationRef = useRef(0);
  durationRef.current =
    playback && playback.durationMs > 0
      ? playback.durationMs
      : durationOverrideMs;

  const playing = playback?.playbackStatus === "playing";
  const base = Math.max(0, playback?.positionMs ?? 0);

  useEffect(() => {
    const now = Date.now();
    const displayed = displayedRef.current;
    const trackChanged = trackRef.current !== trackKey;
    let next: number;

    if (trackChanged) {
      trackRef.current = trackKey;
      lastRewindRef.current = null;
      next = base;
    } else if (!playing && base === 0 && displayed > BACKWARD_TOLERANCE_MS) {
      // 暂停后收到位置 0：这是后端采样缺失的信号，不是真实进度，保持冻结
      lastRewindRef.current = null;
      next = displayed;
    } else if (base + BACKWARD_TOLERANCE_MS < displayed) {
      const confirmed =
        lastRewindRef.current !== null &&
        Math.abs(base - lastRewindRef.current) <= BACKWARD_TOLERANCE_MS;
      lastRewindRef.current = confirmed ? null : base;
      next = confirmed ? base : displayed;
    } else {
      lastRewindRef.current = null;
      next = Math.max(base, displayed);
    }

    next = clampToDuration(next, durationRef.current);
    displayedRef.current = next;
    anchorRef.current = { positionMs: next, updatedAt: now };
    setPositionMs(next);
  }, [base, trackKey]);

  useEffect(() => {
    if (!playing) {
      return;
    }
    // 从暂停恢复时，锚点要刷新到当前显示位置，否则会把暂停时长算进播放进度
    anchorRef.current = { positionMs: displayedRef.current, updatedAt: Date.now() };
    const interval = window.setInterval(() => {
      const elapsed = Date.now() - anchorRef.current.updatedAt;
      const next = clampToDuration(
        anchorRef.current.positionMs + elapsed,
        durationRef.current,
      );
      displayedRef.current = next;
      setPositionMs(next);
    }, TICK_MS);
    return () => window.clearInterval(interval);
  }, [playing, base, trackKey]);

  return positionMs;
}
