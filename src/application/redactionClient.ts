import { invoke } from "@tauri-apps/api/core";
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
  async onFileDrop(callback: (paths: string[]) => void): Promise<() => void> {
    return getCurrentWindow().onDragDropEvent((event) => {
      if (event.payload.type === "drop") callback(event.payload.paths);
    });
  },
};
