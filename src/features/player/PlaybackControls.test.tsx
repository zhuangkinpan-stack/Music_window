import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { PlaybackControls } from "./PlaybackControls";

describe("PlaybackControls", () => {
  it("disables commands that the active session does not support", () => {
    render(
      <PlaybackControls
        playing
        capabilities={{
          playPause: true,
          next: false,
          previous: true,
          seek: false,
        }}
        onPlayPause={() => undefined}
        onNext={() => undefined}
        onPrevious={() => undefined}
      />,
    );

    expect(screen.getByRole("button", { name: "下一首" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "暂停" })).toBeEnabled();
  });
});

