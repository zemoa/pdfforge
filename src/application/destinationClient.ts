import { invoke } from "@tauri-apps/api/core";

export const destinationClient = {
  resolvePastedFolder() {
    return invoke<string | null>("resolve_pasted_destination_folder");
  },
};
