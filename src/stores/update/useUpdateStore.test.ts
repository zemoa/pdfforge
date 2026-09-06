import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({ check: vi.fn() }));

vi.mock("../../application/updateClient", () => ({
  updateClient: {
    cancel: vi.fn(),
    check: mocks.check,
    onUpdateEvent: vi.fn(),
    restore: vi.fn(),
    start: vi.fn(),
    status: vi.fn(),
  },
}));

import { useUpdateStore } from "./useUpdateStore";

describe("update store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mocks.check.mockReset();
  });

  it("shows an available update after an explicit search", async () => {
    mocks.check.mockResolvedValue({ kind: "available", version: "1.2.0", notes: "Notes" });
    const update = useUpdateStore();

    await update.check("en");

    expect(update.result).toEqual({ kind: "available", version: "1.2.0", notes: "Notes" });
    expect(update.phase).toBe("idle");
  });
});
