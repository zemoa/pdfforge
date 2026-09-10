<script setup lang="ts">
type IconName =
  | "add"
  | "chevronLeft"
  | "chevronRight"
  | "chevronUp"
  | "chevronDown"
  | "document"
  | "folder"
  | "grid"
  | "merge"
  | "more"
  | "redact"
  | "settings"
  | "split"
  | "theme"
  | "trash";

const props = withDefaults(
  defineProps<{
    name: IconName;
    size?: number;
    strokeWidth?: number;
  }>(),
  { size: 16, strokeWidth: 1.75 },
);

const paths: Record<IconName, readonly string[]> = {
  add: ["M12 5v14", "M5 12h14"],
  chevronLeft: ["m15 18-6-6 6-6"],
  chevronRight: ["m9 18 6-6-6-6"],
  chevronUp: ["m6 15 6-6 6 6"],
  chevronDown: ["m6 9 6 6 6-6"],
  document: [
    "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z",
    "M14 2v6h6",
    "M8 13h8",
    "M8 17h6",
  ],
  folder: [
    "M3 6a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z",
    "M3 10h18",
  ],
  grid: ["M4 4h6v6H4z", "M14 4h6v6h-6z", "M4 14h6v6H4z", "M14 14h6v6h-6z"],
  merge: ["m8 3 4 4-4 4", "M4 7h8a4 4 0 0 1 4 4v2", "m16 21-4-4 4-4", "M20 17h-8a4 4 0 0 1-4-4v-2"],
  more: ["M5 12h.01", "M12 12h.01", "M19 12h.01"],
  redact: ["M4 4h16v16H4z", "m8 16 8-8", "M8 8h.01", "M16 16h.01"],
  settings: [
    "M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Z",
    "M19.4 15a1.7 1.7 0 0 0 .34 1.87l.06.06-2.12 2.12-.06-.06a1.7 1.7 0 0 0-1.87-.34 1.7 1.7 0 0 0-1.03 1.55v.1h-3v-.1A1.7 1.7 0 0 0 10.69 18.65a1.7 1.7 0 0 0-1.87.34l-.06.06-2.12-2.12.06-.06A1.7 1.7 0 0 0 7.04 15a1.7 1.7 0 0 0-1.55-1.03h-.1v-3h.1A1.7 1.7 0 0 0 7.04 9.94a1.7 1.7 0 0 0-.34-1.87l-.06-.06L8.76 5.9l.06.06a1.7 1.7 0 0 0 1.87.34 1.7 1.7 0 0 0 1.03-1.55v-.1h3v.1a1.7 1.7 0 0 0 1.03 1.55 1.7 1.7 0 0 0 1.87-.34l.06-.06 2.12 2.12-.06.06a1.7 1.7 0 0 0-.34 1.87 1.7 1.7 0 0 0 1.55 1.03h.1v3h-.1A1.7 1.7 0 0 0 19.4 15Z",
  ],
  split: ["M7 3v6", "M17 15v6", "M3 7h8", "M13 17h8", "m8 17 4-5 4-5"],
  theme: [
    "M12 3v2",
    "M12 19v2",
    "m5.64 5.64 1.42 1.42",
    "m16.94 16.94 1.42 1.42",
    "M3 12h2",
    "M19 12h2",
    "m5.64 18.36 1.42-1.42",
    "m16.94 7.06 1.42-1.42",
    "M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z",
  ],
  trash: ["M4 7h16", "M10 11v6", "M14 11v6", "M9 7V4h6v3", "M6 7l1 14h10l1-14"],
};
</script>

<template>
  <svg
    class="line-icon"
    :width="props.size"
    :height="props.size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    :stroke-width="props.strokeWidth"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <path v-for="path in paths[props.name]" :key="path" :d="path" />
  </svg>
</template>
