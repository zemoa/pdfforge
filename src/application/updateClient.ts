import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface UpdateStatus {
  installedVersion: string;
  rollbackAvailable: boolean;
  updatedTo: string | null;
  installationError: boolean;
}

export type UpdateCheck =
  | { kind: "upToDate" }
  | { kind: "unsupported"; version: string }
  | { kind: "available"; version: string; notes: string };

export type UpdateEvent =
  | { type: "progress"; downloaded: number; total: number | null }
  | { type: "cancelled" }
  | { type: "manualDownload"; version: string }
  | { type: "failed"; message: string };

export const updateClient = {
  status() {
    return invoke<UpdateStatus>("update_status");
  },
  check(locale: string) {
    return invoke<UpdateCheck>("check_for_update", { locale });
  },
  start() {
    return invoke<void>("start_update");
  },
  cancel() {
    return invoke<void>("cancel_update");
  },
  restore() {
    return invoke<void>("restore_previous_update");
  },
  onUpdateEvent(callback: (event: UpdateEvent) => void): Promise<UnlistenFn> {
    return listen<UpdateEvent>("update-event", (event) => callback(event.payload));
  },
};
