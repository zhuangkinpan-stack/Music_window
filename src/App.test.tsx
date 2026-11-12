import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import App from "./App";

describe("App", () => {
  it("renders the Lumi main window shell", () => {
    render(<App />);

    expect(screen.getByRole("application", { name: "流光" })).toBeVisible();
    expect(screen.getByText("尚未连接到音乐会话")).toBeVisible();
  });
});

