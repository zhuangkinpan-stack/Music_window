import { FileMusic, LoaderCircle, RotateCcw } from "lucide-react";
import { useEffect, useRef } from "react";

import type { LyricsState } from "../../contracts";

interface LyricsPaneProps {
  lyrics: LyricsState;
  positionMs: number;
  onRetry?: () => void;
  onImport?: () => void;
}

export function LyricsPane({
  lyrics,
  positionMs,
  onRetry,
  onImport,
}: LyricsPaneProps) {
  const activeLineRef = useRef<HTMLParagraphElement | null>(null);
  const currentIndex =
    lyrics.status === "ready"
      ? findCurrentLyricIndex(lyrics.lines, positionMs)
      : -1;

  useEffect(() => {
    const reduceMotion =
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    activeLineRef.current?.scrollIntoView({
      block: "center",
      behavior: reduceMotion ? "auto" : "smooth",
    });
  }, [currentIndex]);

  if (lyrics.status === "ready" && lyrics.lines.length > 0) {
    return (
      <section className="lyrics-pane" aria-label="同步歌词">
        <p className="section-kicker">
          {lyrics.source === "manual" ? "本地歌词" : "同步歌词"}
        </p>
        <div className="lyrics-list">
          {lyrics.lines.map((line, index) => {
            const current = index === currentIndex;
            const distance = Math.abs(index - currentIndex);
            return (
              <p
                key={`${line.timeMs}-${index}`}
                ref={current ? activeLineRef : undefined}
                className="lyric-line"
                data-active={current}
                data-distance={Math.min(distance, 3)}
                aria-current={current ? "true" : undefined}
              >
                {line.text || "♪"}
              </p>
            );
          })}
        </div>
      </section>
    );
  }

  if (lyrics.status === "loading") {
    return (
      <section className="lyrics-pane lyrics-pane--message" aria-label="同步歌词">
        <LoaderCircle className="spin" size={24} aria-hidden="true" />
        <h2>正在匹配同步歌词</h2>
        <p>只向歌词服务发送歌名、歌手、专辑和时长。</p>
      </section>
    );
  }

  const isError = lyrics.status === "error";
  return (
    <section
      className="lyrics-pane lyrics-pane--message"
      aria-label="同步歌词"
    >
      <FileMusic size={26} aria-hidden="true" />
      <h2>{isError ? "歌词加载失败" : "暂无同步歌词"}</h2>
      <p>
        {isError
          ? lyrics.error
          : "可以重试联网匹配，或为当前歌曲导入本地 LRC 文件。"}
      </p>
      <div className="lyrics-actions">
        {onRetry ? (
          <button type="button" className="text-button" onClick={onRetry}>
            <RotateCcw size={15} aria-hidden="true" />
            重新匹配
          </button>
        ) : null}
        {onImport ? (
          <button type="button" className="text-button" onClick={onImport}>
            导入 LRC
          </button>
        ) : null}
      </div>
    </section>
  );
}

function findCurrentLyricIndex(
  lines: LyricsState["lines"],
  positionMs: number,
): number {
  for (let index = lines.length - 1; index >= 0; index -= 1) {
    if (lines[index].timeMs <= positionMs) {
      return index;
    }
  }
  return -1;
}

