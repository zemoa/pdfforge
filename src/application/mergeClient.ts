import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type InteractiveWarning = "bookmarks" | "forms" | "taggedStructure" | "namedDestinations";
export type IncidentKind = "passwordProtected" | "unreadable" | "inaccessible";

export interface MergeSource {
  path: string;
  name: string;
  pageCount: number;
  warnings: InteractiveWarning[];
}

export interface MergeInspection {
  accepted: MergeSource[];
  ignoredNonPdfs: string[];
  incidents: { kind: IncidentKind; path: string; name: string }[];
  warnings: InteractiveWarning[];
}

export interface OutputPreview {
  outputPath: string;
  normalizedName: string;
}

export type MergeEvent =
  | { type: "progress"; current: number; total: number; percent: number }
  | { type: "succeeded"; outputPath: string; opened: boolean }
  | { type: "cancelled" }
  | { type: "failed"; message: string };

function asPaths(result: string | string[] | null): string[] {
  if (result === null) return [];
  return Array.isArray(result) ? result : [result];
}

export const mergeClient = {
  async pickPdfFiles(): Promise<string[]> {
    return asPaths(await open({ filters: [{ name: "PDF", extensions: ["pdf"] }], multiple: true }));
  },
  async pickFolder(): Promise<string | null> {
    const result = await open({ directory: true, multiple: false });
    return typeof result === "string" ? result : null;
  },
  inspectSources(paths: string[]) {
    return invoke<MergeInspection>("inspect_merge_sources", { paths });
  },
  previewOutput(directory: string, fileName: string) {
    return invoke<OutputPreview>("preview_merge_output", { directory, fileName });
  },
  start(sourcePaths: string[], directory: string, fileName: string) {
    return invoke<void>("start_merge", { sourcePaths, directory, fileName });
  },
  cancel() {
    return invoke<void>("cancel_merge");
  },
  onMergeEvent(callback: (event: MergeEvent) => void): Promise<UnlistenFn> {
    return listen<MergeEvent>("merge-event", (event) => callback(event.payload));
  },
  async onFileDrop(callback: (paths: string[]) => void): Promise<UnlistenFn> {
    return getCurrentWindow().onDragDropEvent((event) => {
      if (event.payload.type === "drop") callback(event.payload.paths);
    });
  },
  async onCloseRequest(callback: () => Promise<boolean>): Promise<UnlistenFn> {
    return getCurrentWindow().onCloseRequested(async (event) => {
      if (!(await callback())) event.preventDefault();
    });
  },
};
