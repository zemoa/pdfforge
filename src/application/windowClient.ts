import { getCurrentWindow } from "@tauri-apps/api/window";

export const windowClient = {
  close() {
    return getCurrentWindow().close();
  },
  minimize() {
    return getCurrentWindow().minimize();
  },
  startDragging() {
    return getCurrentWindow().startDragging();
  },
  toggleMaximize() {
    return getCurrentWindow().toggleMaximize();
  },
};
