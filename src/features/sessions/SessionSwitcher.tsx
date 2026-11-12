import { AudioLines } from "lucide-react";

import type { MediaSessionSummary } from "../../contracts";

interface SessionSwitcherProps {
  sessions: MediaSessionSummary[];
  selectedSessionId: string | null;
  onSelect: (sessionId: string) => void;
}

export function SessionSwitcher({
  sessions,
  selectedSessionId,
  onSelect,
}: SessionSwitcherProps) {
  if (sessions.length === 0) {
    return null;
  }

  return (
    <nav className="session-switcher" aria-label="音乐来源">
      {sessions.map((session) => {
        const selected = session.id === selectedSessionId;
        return (
          <button
            key={session.id}
            type="button"
            className="session-pill"
            aria-pressed={selected}
            onClick={() => onSelect(session.id)}
          >
            <span
              className={`session-status session-status--${session.playbackStatus}`}
              aria-hidden="true"
            />
            <AudioLines size={15} aria-hidden="true" />
            <span>{session.appName}</span>
          </button>
        );
      })}
    </nav>
  );
}

