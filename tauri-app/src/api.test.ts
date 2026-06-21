import { describe, expect, it } from "vitest";
import { canInstall, ClientStatus } from "./api";

describe("canInstall", () => {
  it("enables install only for supported status", () => {
    const supported: ClientStatus = {
      kind: "supported",
      version: "synthetic-001",
      installed: false,
      changedFiles: ["String/Item.synthetic.json"],
    };

    expect(canInstall(supported)).toBe(true);
    expect(canInstall({ kind: "notSelected" })).toBe(false);
    expect(canInstall({ kind: "unsupported", reason: "unknown version" })).toBe(false);
  });
});
