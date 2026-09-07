import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { errorCodeFrom, invokeCommand, type ErrorCode } from "./error";

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
  | { type: "failed"; error: ErrorCode };

export const redactionClient = {
  async pickPdfFile(): Promise<string | null> {
    const result = await open({ filters: [{ name: "PDF", extensions: ["pdf"] }], multiple: false });
    return typeof result === "string" ? result : null;
  },
  inspectSource(paths: string[]) {
    return invokeCommand<RedactionSource>("inspect_redaction_source", { paths }, "redactionFailed");
  },
  renderPage(sourcePath: string, page: number) {
    return invokeCommand<RedactionPage>(
      "render_redaction_page",
      { sourcePath, page },
      "redactionFailed",
    );
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
    return invokeCommand<OutputPreview>(
      "preview_redaction_output",
      {
        sourcePath,
        selections,
        directory,
        fileName,
      },
      "redactionFailed",
    );
  },
  start(sourcePath: string, selections: PageRedaction[], directory: string, fileName: string) {
    return invokeCommand<void>(
      "start_redaction",
      {
        request: { sourcePath, selections, directory, fileName },
      },
      "redactionFailed",
    );
  },
  cancel() {
    return invokeCommand<void>("cancel_redaction", undefined, "redactionFailed");
  },
  onRedactionEvent(callback: (event: RedactionEvent) => void): Promise<UnlistenFn> {
    return listen<RedactionEvent>("redaction-event", (event) => {
      const payload = event.payload;
      callback(
        payload.type === "failed"
          ? { type: "failed", error: errorCodeFrom(payload.error, "redactionFailed") }
          : payload,
      );
    });
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
