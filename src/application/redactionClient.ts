import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";

export interface NormalizedRect {
  left: number;
  top: number;
  width: number;
  height: number;
}

export interface TextWord {
  index: number;
  text: string;
  bounds: NormalizedRect[];
}

export interface RedactionSource {
  path: string;
  name: string;
  pageCount: number;
}

export interface RedactionPage {
  page: number;
  aspectRatio: number;
  pngDataUrl: string;
  words: TextWord[];
}

export interface PageRedaction {
  page: number;
  rectangles: NormalizedRect[];
}

export interface OutputPreview {
  outputPath: string;
  normalizedName: string;
}

export type RedactionEvent =
  | { type: "progress"; current: number; total: number; percent: number }
  | { type: "succeeded"; outputPath: string; opened: boolean }
  | { type: "cancelled" }
  | { type: "failed"; message: string };

export const redactionClient = {
  async pickPdfFile(): Promise<string | null> {
    const result = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }], multiple: false });
    return typeof result === "string" ? result : null;
  },
  inspectSource(paths: string[]) {
    return invoke<RedactionSource>("inspect_redaction_source", { paths });
  },
  renderPage(sourcePath: string, page: number) {
    return invoke<RedactionPage>("render_redaction_page", { sourcePath, page });
  },
  async pickFolder(): Promise<string | null> {
    const result = await open({ directory: true, multiple: false });
    return typeof result === "string" ? result : null;
  },
  previewOutput(
    sourcePath: string,
    selections: PageRedaction[],
    directory: string,
    fileName: string,
  ) {
    return invoke<OutputPreview>("preview_redaction_output", {
      sourcePath,
      selections,
      directory,
      fileName,
    });
  },
  start(sourcePath: string, selections: PageRedaction[], directory: string, fileName: string) {
    return invoke<void>("start_redaction", {
      request: { sourcePath, selections, directory, fileName },
    });
  },
  cancel() {
    return invoke<void>("cancel_redaction");
  },
  onRedactionEvent(callback: (event: RedactionEvent) => void): Promise<UnlistenFn> {
    return listen<RedactionEvent>("redaction-event", (event) => callback(event.payload));
  },
  async onFileDrop(callback: (paths: string[]) => void): Promise<() => void> {
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
