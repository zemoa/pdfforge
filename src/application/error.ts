import { invoke } from "@tauri-apps/api/core";

export const errorCodes = [
  "sourceNotPdf",
  "sourcePasswordProtected",
  "sourceUnreadable",
  "sourceInaccessible",
  "sourceMultiple",
  "sourceEmpty",
  "mergeSourcesRequired",
  "outputNameRequired",
  "destinationMissing",
  "destinationNotWritable",
  "selectionRequired",
  "pageUnavailable",
  "groupsRequired",
  "groupsOverlap",
  "redactionZoneInvalid",
  "rendererUnavailable",
  "operationInProgress",
  "preparationChanged",
  "mergeFailed",
  "splitFailed",
  "redactionFailed",
  "updateCheckFailed",
  "updateDownloadFailed",
  "updateSpaceUnavailable",
  "updateVerificationFailed",
  "updateInstallFailed",
  "updateRestoreFailed",
  "updateUnavailable",
  "previousVersionUnavailable",
  "pdfOperationInProgress",
  "unexpected",
] as const;

export type ErrorCode = (typeof errorCodes)[number];

export class ApplicationError extends Error {
  constructor(readonly code: ErrorCode) {
    super(code);
  }
}

function isErrorCode(value: unknown): value is ErrorCode {
  return typeof value === "string" && errorCodes.includes(value as ErrorCode);
}

export function errorCodeFrom(error: unknown, fallback: ErrorCode): ErrorCode {
  if (error instanceof ApplicationError) return error.code;
  if (typeof error === "object" && error !== null && "code" in error && isErrorCode(error.code)) {
    return error.code;
  }
  return fallback;
}

export async function invokeCommand<T>(
  command: string,
  args: Record<string, unknown> | undefined,
  fallback: ErrorCode,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw new ApplicationError(errorCodeFrom(error, fallback));
  }
}
