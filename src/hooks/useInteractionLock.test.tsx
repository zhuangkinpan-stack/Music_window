import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useInteractionLock } from "./useInteractionLock";

describe("useInteractionLock", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("relocks five seconds after the last interaction", () => {
    const { result } = renderHook(() => useInteractionLock());

    act(() => result.current.unlock());
    expect(result.current.locked).toBe(false);

    act(() => {
      vi.advanceTimersByTime(4_000);
      result.current.touch();
      vi.advanceTimersByTime(4_999);
    });
    expect(result.current.locked).toBe(false);

    act(() => vi.advanceTimersByTime(1));
    expect(result.current.locked).toBe(true);
  });

  it("can relock immediately when the pointer leaves", () => {
    const { result } = renderHook(() => useInteractionLock());

    act(() => {
      result.current.unlock();
      result.current.lock();
    });

    expect(result.current.locked).toBe(true);
  });
});

