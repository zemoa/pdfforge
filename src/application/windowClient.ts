import { getCurrentWindow } from "@tauri-apps/api/window";

export type WindowResizeDirection =
  "East" | "North" | "NorthEast" | "NorthWest" | "South" | "SouthEast" | "SouthWest" | "West";

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
  startResizeDragging(direction: WindowResizeDirection) {
    return getCurrentWindow().startResizeDragging(direction);
  },
  toggleMaximize() {
    return getCurrentWindow().toggleMaximize();
  },
};
