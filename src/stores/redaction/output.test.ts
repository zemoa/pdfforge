import { describe, expect, it } from "vitest";

import { defaultOutputName, sourceDirectory } from "./output";

describe("redaction output defaults", () => {
  it("derives a masked PDF name from the source name", () => {
    expect(defaultOutputName("contract.PDF")).toBe("contract-masked.pdf");
  });

  it("keeps the source directory on Linux and Windows paths", () => {
    expect(sourceDirectory("/documents/contract.pdf")).toBe("/documents");
    expect(sourceDirectory("C:\\documents\\contract.pdf")).toBe("C:\\documents");
  });
});
