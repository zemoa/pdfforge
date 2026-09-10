import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { errorCodeFrom, invokeCommand, type ErrorCode } from "./error";

export interface UpdateStatus {
  installedVersion: string;
  rollbackAvailable: boolean;
  updatedTo: string | null;
  installationError: boolean;
}

export interface ReleaseNotes {
  version: string;
  notes: string;
}

export type UpdateCheck =
  | { kind: "upToDate" }
  | { kind: "unsupported"; version: string; releases: ReleaseNotes[] }
  | { kind: "available"; version: string; releases: ReleaseNotes[] };

export type UpdateEvent =
  | { type: "progress"; downloaded: number; total: number | null }
  | { type: "cancelled" }
  | { type: "manualDownload"; version: string }
  | { type: "failed"; error: ErrorCode };

export const updateClient = {
  status() {
    return invokeCommand<UpdateStatus>("update_status", undefined, "updateInstallFailed");
  },
  check(locale: string) {
    return invokeCommand<UpdateCheck>("check_for_update", { locale }, "updateCheckFailed");
  },
  start() {
    return invokeCommand<void>("start_update", undefined, "updateInstallFailed");
  },
  cancel() {
    return invokeCommand<void>("cancel_update", undefined, "updateInstallFailed");
  },
  restore() {
    return invokeCommand<void>("restore_previous_update", undefined, "updateRestoreFailed");
  },
  onUpdateEvent(callback: (event: UpdateEvent) => void): Promise<UnlistenFn> {
    return listen<UpdateEvent>("update-event", (event) => {
      const payload = event.payload;
      callback(
        payload.type === "failed"
          ? { type: "failed", error: errorCodeFrom(payload.error, "updateInstallFailed") }
          : payload,
      );
    });
  },
};
