import { onBeforeUnmount, ref, watch } from "vue";

interface Insertion {
  entryId: string;
  side: "before" | "after";
}

/** Owns only the pointer gesture; the caller commits the resulting move. */
export function usePointerReorder(options: {
  entryIds: () => readonly string[];
  enabled: () => boolean;
  container: () => HTMLElement | null;
  move: (from: number, to: number) => void;
}) {
  const draggingId = ref<string | null>(null);
  const insertion = ref<Insertion | null>(null);
  let gesture: {
    entryId: string;
    pointerId: number;
    handle: HTMLElement;
    startX: number;
    startY: number;
    x: number;
    y: number;
  } | null = null;
  let animationFrame = 0;
  let previousFrame = 0;

  function cancel() {
    const previous = gesture;
    gesture = null;
    draggingId.value = null;
    insertion.value = null;
    cancelAnimationFrame(animationFrame);
    window.removeEventListener("pointermove", movePointer);
    window.removeEventListener("pointerup", releasePointer);
    window.removeEventListener("pointercancel", cancelPointer);
    window.removeEventListener("keydown", keyDown);
    window.removeEventListener("blur", cancel);
    previous?.handle.removeEventListener("lostpointercapture", cancel);
    if (previous?.handle.hasPointerCapture(previous.pointerId)) {
      previous.handle.releasePointerCapture(previous.pointerId);
    }
  }

  function containerAtPointer() {
    if (!gesture) return undefined;
    const hit = document.elementFromPoint(gesture.x, gesture.y);
    const container = options.container();
    return container && hit && container.contains(hit) ? container : undefined;
  }

  function updateInsertion() {
    insertion.value = null;
    if (!gesture || !draggingId.value) return;
    const container = containerAtPointer();
    if (!container) return;
    const { x, y } = gesture;
    const items = [...container.querySelectorAll<HTMLElement>("[data-reorder-id]")];
    let nearest: HTMLElement | undefined;
    let distance = Infinity;
    for (const item of items) {
      const rect = item.getBoundingClientRect();
      const dx = Math.max(rect.left - x, 0, x - rect.right);
      const dy = Math.max(rect.top - y, 0, y - rect.bottom);
      const candidate = dx * dx + dy * dy;
      if (candidate < distance) {
        distance = candidate;
        nearest = item;
      }
    }
    if (!nearest?.dataset.reorderId) return;
    const rect = nearest.getBoundingClientRect();
    const before = x < rect.left + rect.width / 2;
    insertion.value = {
      entryId: nearest.dataset.reorderId,
      side: before ? "before" : "after",
    };
  }

  function scrollFrame(time: number) {
    if (!gesture) return;
    const elapsed = Math.min(time - previousFrame, 32);
    previousFrame = time;
    const container = containerAtPointer();
    if (draggingId.value && container) {
      const rect = container.getBoundingClientRect();
      const edge = Math.min(40, rect.height / 3);
      const speed =
        gesture.y < rect.top + edge
          ? -Math.min(1, (rect.top + edge - gesture.y) / edge)
          : Math.max(0, Math.min(1, (gesture.y - rect.bottom + edge) / edge));
      container.scrollTop += speed * elapsed * 0.6;
      updateInsertion();
    }
    animationFrame = requestAnimationFrame(scrollFrame);
  }

  function movePointer(event: PointerEvent) {
    if (!gesture || event.pointerId !== gesture.pointerId) return;
    gesture.x = event.clientX;
    gesture.y = event.clientY;
    if (Math.hypot(gesture.x - gesture.startX, gesture.y - gesture.startY) >= 5) {
      draggingId.value = gesture.entryId;
    }
    if (draggingId.value) event.preventDefault();
    updateInsertion();
  }

  function releasePointer(event: PointerEvent) {
    if (!gesture || event.pointerId !== gesture.pointerId) return;
    gesture.x = event.clientX;
    gesture.y = event.clientY;
    updateInsertion();
    const ids = options.entryIds();
    const from = ids.indexOf(gesture.entryId);
    const target = insertion.value;
    const targetIndex = target ? ids.indexOf(target.entryId) : -1;
    const slot = targetIndex + (target?.side === "after" ? 1 : 0);
    const to = slot > from ? slot - 1 : slot;
    const commit = options.enabled() && !!draggingId.value && targetIndex >= 0 && from >= 0;
    cancel();
    if (commit) options.move(from, to);
  }

  function cancelPointer(event: PointerEvent) {
    if (event.pointerId === gesture?.pointerId) cancel();
  }

  function keyDown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      cancel();
    }
  }

  function start(event: PointerEvent, entryId: string) {
    if (
      !options.enabled() ||
      options.entryIds().length < 2 ||
      !options.entryIds().includes(entryId)
    )
      return;
    if (event.button !== 0 || !event.isPrimary || !(event.currentTarget instanceof HTMLElement))
      return;
    cancel();
    event.preventDefault();
    const handle = event.currentTarget;
    handle.focus({ preventScroll: true });
    gesture = {
      entryId,
      pointerId: event.pointerId,
      handle,
      startX: event.clientX,
      startY: event.clientY,
      x: event.clientX,
      y: event.clientY,
    };
    handle.setPointerCapture(event.pointerId);
    handle.addEventListener("lostpointercapture", cancel);
    window.addEventListener("pointermove", movePointer, { passive: false });
    window.addEventListener("pointerup", releasePointer);
    window.addEventListener("pointercancel", cancelPointer);
    window.addEventListener("keydown", keyDown);
    window.addEventListener("blur", cancel);
    previousFrame = performance.now();
    animationFrame = requestAnimationFrame(scrollFrame);
  }

  function itemClasses(entryId: string) {
    const target = insertion.value;
    const isTarget = target?.entryId === entryId;
    return {
      "reorder-dragging": draggingId.value === entryId,
      "reorder-before": isTarget && target.side === "before",
      "reorder-after": isTarget && target.side === "after",
    };
  }

  watch(() => [...options.entryIds()], cancel, { flush: "sync" });
  watch(options.enabled, cancel, { flush: "sync" });
  onBeforeUnmount(cancel);

  return { start, itemClasses, cancel };
}
