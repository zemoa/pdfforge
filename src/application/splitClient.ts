import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type SplitMode = "eachPage" | "extract" | "groups";

export interface SplitSource {
  path: string;
  name: string;
  pageCount: number;
}

export interface Thumbnail {
  page: number;
  pngDataUrl: string;
}

export interface OutputPreview {
  outputPaths: string[];
  normalizedName: string;
}

export type SplitEvent =
  | { type: "progress"; current: number; total: number; percent: number }
  | { type: "succeeded"; outputPaths: string[]; opened: boolean }
  | { type: "cancelled" }
  | { type: "failed"; message: string };

export const splitClient = {
  async pickPdfFile(): Promise<string | null> {
    const result = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }], multiple: false });
    return typeof result === "string" ? result : null;
  },
  async pickFolder(): Promise<string | null> {
    const result = await open({ directory: true, multiple: false });
    return typeof result === "string" ? result : null;
  },
  inspectSource(paths: string[]) {
    return invoke<SplitSource>("inspect_split_source", { paths });
  },
  renderThumbnails(sourcePath: string, pages: number[]) {
    return invoke<Thumbnail[]>("render_split_thumbnails", { sourcePath, pages });
  },
  previewOutput(
    sourcePath: string,
    mode: SplitMode,
    pages: number[],
    groups: number[][],
    directory: string,
    fileName: string,
  ) {
    return invoke<OutputPreview>("preview_split_output", {
      sourcePath,
      mode,
      pages,
      groups,
      directory,
      fileName,
    });
  },
  start(
    sourcePath: string,
    mode: SplitMode,
    pages: number[],
    groups: number[][],
    directory: string,
    fileName: string,
  ) {
    return invoke<void>("start_split", {
      request: { sourcePath, mode, pages, groups, directory, fileName },
    });
  },
  cancel() {
    return invoke<void>("cancel_split");
  },
  onSplitEvent(callback: (event: SplitEvent) => void): Promise<UnlistenFn> {
    return listen<SplitEvent>("split-event", (event) => callback(event.payload));
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
  destroyWindow() {
    return getCurrentWindow().destroy();
  },
};
