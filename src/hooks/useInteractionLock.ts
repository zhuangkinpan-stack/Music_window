import { useCallback, useEffect, useRef, useState } from "react";

const AUTO_LOCK_MS = 5_000;

export function useInteractionLock() {
  const [locked, setLockedState] = useState(true);
  const timerRef = useRef<number | null>(null);
  const lockedRef = useRef(true);

  const clearTimer = useCallback(() => {
    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const scheduleRelock = useCallback(() => {
    clearTimer();
    timerRef.current = window.setTimeout(() => {
      lockedRef.current = true;
      setLockedState(true);
      timerRef.current = null;
    }, AUTO_LOCK_MS);
  }, [clearTimer]);

  const setLocked = useCallback(
    (next: boolean) => {
      clearTimer();
      lockedRef.current = next;
      setLockedState(next);
      if (!next) {
        scheduleRelock();
      }
    },
    [clearTimer, scheduleRelock],
  );

  const lock = useCallback(() => {
    setLocked(true);
  }, [setLocked]);

  const unlock = useCallback(() => {
    setLocked(false);
  }, [setLocked]);

  const touch = useCallback(() => {
    if (!lockedRef.current) {
      scheduleRelock();
    }
  }, [scheduleRelock]);

  useEffect(() => {
    return clearTimer;
  }, [clearTimer]);

  return { locked, unlock, lock, touch, setLocked };
}

